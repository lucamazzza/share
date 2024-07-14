pub mod app;
pub mod inputs;
pub mod io;

use app::{Chat, AppReturn};
use inputs::event::Event as EventInput;
use inputs::InputEvent;
use crate::app::ui;

use eyre::Result;
use io::IOEvent;
use std::io::{stdout, Write};
use std::sync::Arc;
use std::time::Duration;
use tui::backend::CrosstermBackend;
use tui::Terminal;
use tui_input::backend::crossterm as backend;
use tui_input::backend::crossterm::EventHandler;
use tui_input::Input;
use crossterm::{
    execute,
    cursor::{Hide, Show},
    terminal::{
        disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
    },
    event::{read, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEvent},

};
use futures::StreamExt;
use libp2p::{PeerId, Swarm};
use libp2p::floodsub::Topic;
use crate::app::state::Message;

pub async fn start_ui(
    app: &Arc<tokio::sync::Mutex<Chat>>,
    peer_id: PeerId,
    swarm: &mut Swarm<Chat>,
    topic: &mut Topic) -> Result<()> {
    let stdout = stdout();
    let mut stdout_lock = stdout.lock();
    execute!(stdout_lock, Hide, EnterAlternateScreen, EnableMouseCapture)?;
    enable_raw_mode()?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;
    terminal.hide_cursor()?;
    let input: Input = "Write message here ".into();
    backend::write(&mut stdout_lock, input.value(), input.cursor(), (0, 0), 15)?;
    stdout_lock.flush()?;
    let tick_rate = Duration::from_millis(200);
    let mut events = EventInput::new(tick_rate);
    {
        let mut app = app.lock().await;
        app.dispatch(IOEvent::Initialize).await;
    }
    loop {
        let mut app = app.lock().await;
        terminal.draw(|rect| ui::draw(rect, &app))?;
        let result = match events.next().await {
            InputEvent::Input(key) => app.do_action(key).await,
            InputEvent::Send => app.do_send(input.to_string(), peer_id, swarm, topic).await,
            InputEvent::Tick => app.update_on_tick().await,
        };
        if result == AppReturn::Exit {
            events.close();
            break;
        }
    }
    terminal.clear()?;
    terminal.show_cursor()?;
    disable_raw_mode()?;
    Ok(())
}
