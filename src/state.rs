use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use crate::history::History;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum MessageType {
    Message,
    State,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Message {
    pub message_type: MessageType,
    pub data: Vec<u8>,
    pub addressee: Option<String>,
    pub source: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct State {
    pub history: History<Message>,
    pub usernames: HashMap<String, String>,
}

impl State {
    pub fn merge(&mut self, mut other: State) {
        for (peer_id, username) in other.usernames.drain() {
            if !self.usernames.contains_key(&peer_id) {
                println!("{} joined", &username);
                self.usernames.insert(peer_id, username);
            }
        }
        if self.history.get_count() < 1 && other.history.get_count() > 0 {
            for message in other.history.get_all() {
                println!(
                    "{}: {}",
                    self.get_username(&message.source),
                    String::from_utf8_lossy(&message.data)
                );
                self.history.push((*message).to_owned());
            }
        }
    }
    
    pub fn get_username(&self, usr: &String) -> String {
        self.usernames
            .get(usr)
            .unwrap_or(&String::from("n/d"))
            .to_string()
    }
}