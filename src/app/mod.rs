use self::action::Actions;
use self::state::State;
use crate::app::action::Action;
use crate::app::state::{Message, MessageType};
use crate::inputs::key::Key;
use crate::io::IOEvent;

pub mod action;
pub mod state;
pub mod ui;
pub mod history;

use libp2p::{
    floodsub::{Floodsub, FloodsubEvent, Topic},
    mdns::{Mdns, MdnsEvent},
    NetworkBehaviour,
    PeerId,
    Swarm, swarm::NetworkBehaviourEventProcess,
};
use log::{debug, error, warn, info};
use tokio::{sync::mpsc};

#[derive(Debug, PartialEq, Eq)]
pub enum AppReturn {
    Exit,
    Continue,
}

#[derive(NetworkBehaviour)]
#[behaviour(event_process = true)]
pub struct Chat {
    pub(crate) dns: Mdns,
    pub messager: Floodsub,
    #[behaviour(ignore)]
    pub(crate) state: State,
    #[behaviour(ignore)]
    pub(crate) peer_id: String,
    #[behaviour(ignore)]
    pub(crate) responder: mpsc::UnboundedSender<Message>,
    #[behaviour(ignore)]
    pub(crate) actions: Actions,
    #[behaviour(ignore)]
    pub(crate) io_tx: mpsc::Sender<IOEvent>,
}

impl Chat {
    pub async fn do_send(&mut self, msg: String, peer_id: PeerId, swarm: &mut Swarm<Chat>, topic: &mut Topic) -> AppReturn {
        self.dispatch(IOEvent::Send).await;
        let message: Message = Message {
            message_type: MessageType::Message,
            data: msg.as_bytes().to_vec(),
            addressee: None,
            source: peer_id.to_string(),
        };
        send_message(&message, swarm, &topic);
        swarm
            .behaviour_mut()
            .state
            .history
            .push(message);
        AppReturn::Continue
    }

    pub async fn do_action(&mut self, key: Key) -> AppReturn {
        if let Some(action) = self.actions.find(key) {
            debug!("Run action [{:?}]", action);
            match action {
                Action::Quit => return AppReturn::Exit,
                Action::Send => {
                    self.dispatch(IOEvent::Send).await;
                    AppReturn::Continue
                }
            }
        } else {
            warn!("No action associated with key [{:?}]", key);
            AppReturn::Continue
        }
    }

    pub async fn update_on_tick(&mut self) -> AppReturn {
        // here we just increment a counter
        self.state.incr_tick();
        AppReturn::Continue
    }

    pub async fn dispatch(&mut self, action: IOEvent) {
        if let Err(e) = self.io_tx.send(action).await {
            error!("Error sending IO event: {}", e);
        }
    }

    pub fn actions(&self) -> &Actions {
        &self.actions
    }
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