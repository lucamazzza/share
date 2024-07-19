use std::sync::Arc;
use eyre::Result;
use log::{error, info};
use super::IOEvent;
use crate::app::Chat;

pub struct IOAsyncHandler {
    app: Arc<tokio::sync::Mutex<Chat>>,
}

impl IOAsyncHandler {
    pub fn new(app: Arc<tokio::sync::Mutex<Chat>>) -> Self {
        Self { app }
    }

    pub async fn handle_io_event(&mut self, io_event: IOEvent) {
        let result = match io_event {
            IOEvent::Initialize => self.do_initialize().await,
            IOEvent::Send => self.do_send().await,
        };
        if let Err(err) = result {
            error!("Something wrong happened: {}", err);
        }
        let mut app = self.app.lock().await;
    }

    async fn do_initialize(&mut self) -> Result<()> {
        info!("Initializing...");
        let mut app = self.app.lock().await;
        info!("Done!");
        Ok(())
    }

    async fn do_send(&mut self) -> Result<()> {
        info!("Sending...");
        let mut app = self.app.lock().await;
        info!("Done!");
        Ok(())
    }
}
