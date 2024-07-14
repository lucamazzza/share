mod app;
mod io;
mod inputs;

use eyre::Result;
use log::LevelFilter;
use share::io::handler::IOAsyncHandler;
use share::io::IOEvent;
use share::app::{Chat, send_message, send_response};
use share::app::history::History;
use share::app::action::Action;
use share::app::action::Actions;
use share::app::state::{Message, MessageType, State};
use share::start_ui;

use futures::StreamExt;
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
use std::{collections::HashMap, process};
use std::error::{Error};
use std::fs::metadata;
use tokio::{io::AsyncBufReadExt, signal::ctrl_c, sync::mpsc};
use tui::{
    backend::Backend,
    Frame,
    layout::{Constraint, Direction, Layout},
    widgets::{Block, Borders},
};
use std::sync::Arc;

/*


#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    println!("Username: ");
    let username = stdin
        .next_line()
        .await
        .expect("a valid username")
        .unwrap_or(String::from("anonymous"))
        .trim()
        .to_owned();
    let (sync_io_tx, mut sync_io_rx) = mpsc::channel::<IOEvent>(100);
    let mut behaviour = Chat {
        dns: Mdns::new(MdnsConfig::default())
            .await
            .expect("unable to create mdns"),
        messager: Floodsub::new(peer_id),
        state: State {
            history: History::new(),
            usernames: HashMap::from([(peer_id.to_string(), username)]),
            counter_tick: 0,
        },
        peer_id: peer_id.to_string(),
        responder: response_sender,
        actions: Action::iterator().collect(),
        io_tx: sync_io_tx.clone(),
    };
    let topic = Topic::new("sylo");
    behaviour.messager.subscribe(topic.clone());
    let mut swarm = Swarm::new(transport, behaviour, peer_id);
    swarm.listen_on("/ip4/0.0.0.0/tcp/0".parse()?)?;
    tui_logger::init_logger(LevelFilter::Debug).unwrap();
    tui_logger::set_default_level(LevelFilter::Debug);
    tokio::spawn(async move {
        let mut handler = IOAsyncHandler::new(app);
        while let Some(io_event) = sync_io_rx.recv().await {
            handler.handle_io_event(io_event).await;
        }
    });

    start_ui(&app_ui).await?;
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

 */
#[tokio::main]
async fn main() -> Result<()>{
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
    let (response_sender,
        mut response_rcv) = mpsc::unbounded_channel();
    println!("Username: ");
    let mut stdin = tokio::io::BufReader::new(tokio::io::stdin()).lines();
    let username = stdin
        .next_line()
        .await
        .expect("a valid username")
        .unwrap_or(String::from("anonymous"))
        .trim()
        .to_owned();
    let (sync_io_tx,
        mut sync_io_rx) = mpsc::channel::<IOEvent>(100);
    let mut behaviour = Chat {
        dns: Mdns::new(MdnsConfig::default())
            .await
            .expect("unable to create mdns"),
        messager: Floodsub::new(peer_id),
        state: State {
            history: History::new(),
            usernames: HashMap::from([(peer_id.to_string(), username)]),
            counter_tick: 0,
        },
        peer_id: peer_id.to_string(),
        responder: response_sender,
        actions: Actions::default(),
        io_tx: sync_io_tx.clone(),
    };
    let mut topic = Topic::new("sylo");
    behaviour.messager.subscribe(topic.clone());
    let mut swarm = Swarm::new(transport, behaviour, peer_id);
    swarm.listen_on("/ip4/0.0.0.0/tcp/0".parse()?)?;
    tui_logger::init_logger(LevelFilter::Debug).unwrap();
    tui_logger::set_default_level(LevelFilter::Debug);
    // TODO: FIND HOW TO MOVE VAL
    let app = Arc::new(tokio::sync::Mutex::new(behaviour));
    let app_ui = Arc::clone(&app);
    tokio::spawn(async move {
        let mut handler = IOAsyncHandler::new(app);
        while let Some(io_event) = sync_io_rx.recv().await {
            handler.handle_io_event(io_event).await;
        }
    });
    start_ui(&app_ui, peer_id, &mut swarm, &mut topic).await?;
    Ok(())
}