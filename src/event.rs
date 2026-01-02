use color_eyre::eyre::{OptionExt};
use crossterm::event::{KeyEvent};
use tokio::sync::mpsc;

use crate::{panels::{containers, projects}};

/// Application events.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Message {
    Tick,
    Quit,
    KeyPress(KeyEvent),
    ProjectsPanel(projects::Message),
    ContainersPanel(containers::Message),
}

/// Terminal event handler.
#[derive(Debug)]
pub struct EventHandler {
    /// Event sender channel.
    sender: mpsc::UnboundedSender<Message>,
    /// Event receiver channel.
    receiver: mpsc::UnboundedReceiver<Message>,
}

impl Default for EventHandler {
    fn default() -> Self {
        Self::new()
    }
}

impl EventHandler {
    /// Constructs a new instance of [`EventHandler`] and spawns a new thread to handle events.
    pub fn new() -> Self {
        let (sender, receiver) = mpsc::unbounded_channel();
        Self { sender, receiver }
    }

    pub async fn next(&mut self) -> color_eyre::Result<Message> {
        self.receiver
            .recv()
            .await
            .ok_or_eyre("Failed to receive event")
    }

    pub fn send(&mut self, app_event: Message) {
        let _ = self.sender.send(app_event);
    }

    pub fn sender(&self) -> mpsc::UnboundedSender<Message> {
        self.sender.clone()
    }
}
