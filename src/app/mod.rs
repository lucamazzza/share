pub(crate) mod history;
pub(crate) mod state;
pub(crate) mod ui;

use futures::StreamExt;
use crate::app::history::History;
use libp2p::{
    core::upgrade,
    floodsub::{Floodsub, FloodsubEvent, Topic},
    identity::{self},
    mdns::{Mdns, MdnsConfig, MdnsEvent},
    mplex,
    NetworkBehaviour,
    noise::{Keypair, NoiseConfig, X25519Spec},
    PeerId,
    Swarm, swarm::NetworkBehaviourEventProcess, tcp::TcpConfig, Transport,
};
use log::{error, info};
use crate::app::state::{Message, MessageType, State};
use std::{collections::HashMap, process};
use std::error::Error;
use std::fs::metadata;
use tokio::{io::AsyncBufReadExt, signal::ctrl_c, sync::mpsc};
use tui::{
    backend::Backend,
    Frame,
    layout::{Constraint, Direction, Layout},
    widgets::{Block, Borders},
};

#[derive(NetworkBehaviour)]
#[behaviour(event_process = true)]
pub(crate) struct Chat {
    pub(crate) dns: Mdns,
    pub(crate) messager: Floodsub,
    #[behaviour(ignore)]
    pub(crate) state: State,
    #[behaviour(ignore)]
    pub(crate) peer_id: String,
    #[behaviour(ignore)]
    pub(crate) responder: mpsc::UnboundedSender<Message>,
}

impl NetworkBehaviourEventProcess<MdnsEvent> for Chat {
    fn inject_event(&mut self, event: MdnsEvent) {
        match event {
            MdnsEvent::Discovered(list) => {
                for (peer, addr) in list {
                    info!("Discovered {}@{}", peer, addr);
                    self.messager.add_node_to_partial_view(peer);
                }
            }
            MdnsEvent::Expired(list) => {
                for (peer, addr) in list {
                    info!("Expired {}@{}", peer, addr);
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

pub fn send_response(msg: Message, responder: mpsc::UnboundedSender<Message>) {
    tokio::spawn(async move {
        if let Err(e) = responder.send(msg) {
            error!("Error sending response: {:?}", e);
        }
    });
}

pub fn send_message(msg: &Message, swarm: &mut Swarm<Chat>, topic: &Topic) {
    let bytes = bincode::serialize(msg).unwrap();
    swarm
        .behaviour_mut()
        .messager
        .publish(topic.clone(), bytes);
}