mod state;
mod history;

use futures::StreamExt;
use history::History;
use libp2p::{
    core::upgrade,
    floodsub::{Floodsub, FloodsubEvent, Topic},
    identity::{self},
    mdns::{Mdns, MdnsConfig, MdnsEvent},
    mplex,
    noise::{Keypair, NoiseConfig, X25519Spec},
    swarm::NetworkBehaviourEventProcess,
    tcp::TcpConfig,
    NetworkBehaviour, PeerId, Swarm, Transport,
};
use log::{error, info};
use state::{Message, MessageType, State};
use std::{collections::HashMap, process};
use std::fs::metadata;
use tokio::{io::AsyncBufReadExt, sync::mpsc, signal::ctrl_c};

#[derive(NetworkBehaviour)]
#[behaviour(event_process = true)]
struct Chat {
    dns: Mdns,
    messager: Floodsub,
    #[behaviour(ignore)]
    state: State,
    #[behaviour(ignore)]
    peer_id: String,
    #[behaviour(ignore)]
    responder: mpsc::UnboundedSender<Message>,
}

impl NetworkBehaviourEventProcess<MdnsEvent> for Chat {
    fn inject_event(&mut self, event: MdnsEvent) {
        match event {
            MdnsEvent::Discovered(list) => {
                for (peer, addr) in list {
                    info!("Discovered {} at {}", peer, addr);
                    self.messager.add_node_to_partial_view(peer);
                }
            }
            MdnsEvent::Expired(list) => {
                for (peer, addr) in list {
                    info!("Expired {} at {}", peer, addr);
                    self.messager.remove_node_from_partial_view(&peer);
                }
            }
        }
    }
}

impl NetworkBehaviourEventProcess<FloodsubEvent> for Chat {
    fn inject_event(&mut self, event: FloodsubEvent) {
        match event {
            FloodsubEvent::Message(msg) => {
                let deserialize = bincode::deserialize::<Message>(&msg.data);
                if let Ok(message) = deserialize {
                    if let Some(user) = &message.addressee {
                        if *user != self.peer_id.to_string() {
                            return;
                        }
                    }
                    match message.message_type {
                        MessageType::Message => {
                            let un: String = self.state.get_username(&msg.source.to_string());
                            println!("{}: {}", un, String::from_utf8_lossy(&message.data));
                            self.state.history.push(message);
                        }
                        MessageType::State => {
                            info!("History received!");
                            let data: State = bincode::deserialize(&message.data).unwrap();
                            self.state.merge(data);
                        }
                    }
                } else {
                    error!("Failed to decode message. Cause: {:?}", deserialize.unwrap_err());
                }
            }
            FloodsubEvent::Subscribed { peer_id, topic: _ } => {
                info!("Sending stage to {}", peer_id);
                let message: Message = Message {
                    message_type: MessageType::State,
                    data: bincode::serialize(&self.state).unwrap(),
                    addressee: Some(peer_id.to_string()),
                    source: self.peer_id.to_string(),
                };
                send_response(message, self.responder.clone());
            }
            FloodsubEvent::Unsubscribed { peer_id, topic: _ } => {
                let name = self
                    .state
                    .usernames
                    .remove(&peer_id.to_string())
                    .unwrap_or(String::from("Anon"));
                println!("{} has left the chat.", name);
            }
        }
    }
}

fn send_response(msg: Message, responder: mpsc::UnboundedSender<Message>) {
    tokio::spawn(async move {
        if let Err(e) = responder.send(msg) {
            error!("Error sending response: {:?}", e);
        }
    });
}

fn send_message(msg: &Message, swarm: &mut Swarm<Chat>, topic: &Topic) {
    let bytes = bincode::serialize(msg).unwrap();
    swarm
        .behaviour_mut()
        .messager
        .publish(topic.clone(), bytes);
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    pretty_env_logger::init();
    let id_keys = identity::Keypair::generate_ed25519();
    let peer_id = PeerId::from(id_keys.public());
    println!("Local peer id: {:?}", peer_id);
    let auth_keys = Keypair::<X25519Spec>::new()
        .into_authentic(&id_keys)
        .expect("unable to create auth keys");
    let transport = TcpConfig::new()
        .upgrade(upgrade::Version::V1)
        .authenticate(NoiseConfig::xx(auth_keys).into_authenticated())
        .multiplex(mplex::MplexConfig::new())
        .boxed();
    let (response_sender, mut response_rcv) = mpsc::unbounded_channel();
    let mut stdin = tokio::io::BufReader::new(tokio::io::stdin()).lines();
    println!("Username: ");
    let username = stdin
        .next_line()
        .await
        .expect("a valid username")
        .unwrap_or(String::from("anon"))
        .trim()
        .to_owned();
    let mut behaviour = Chat {
        dns: Mdns::new(MdnsConfig::default())
            .await
            .expect("unable to create mdns"),
        messager: Floodsub::new(peer_id),
        state: State {
            history: History::new(),
            usernames: HashMap::from([(peer_id.to_string(), username)]),
        },
        peer_id: peer_id.to_string(),
        responder: response_sender,
    };
    let topic = Topic::new("sylo");
    behaviour.messager.subscribe(topic.clone());
    let mut swarm = Swarm::new(transport, behaviour, peer_id);
    swarm.listen_on("/ip4/0.0.0.0/tcp/0".parse()?)?;
    loop {
        tokio::select! {
            line = stdin.next_line() => {
                if let Some(input_line) = line.expect("a valid line") {
                    let message: Message = Message {
                        message_type: MessageType::Message,
                        data: input_line.as_bytes().to_vec(),
                        addressee: None,
                        source: peer_id.to_string(),
                    };
                    send_message(&message, &mut swarm, &topic);
                    swarm
                        .behaviour_mut()
                        .state
                        .history
                        .push(message);
                }
            },
            event = swarm.select_next_some() => {
                info!("Swarm event: {:?}", event);
            },
            response = response_rcv.recv() => {
                if let Some(msg) = response {
                    send_message(&msg, &mut swarm, &topic);
                }
            },
            event = ctrl_c() => {
                if let Err(e) = event {
                    println!("Failed to register interrupt handler {}", e);
                }
                break;
            }
        }
    }
    swarm.behaviour_mut().messager.unsubscribe(topic);
    swarm.select_next_some().await;
    process::exit(0);
}