use std::fmt::Display;

use color_eyre::eyre::OptionExt;
use crossterm::event::KeyEvent;
use tokio::sync::mpsc;

use crate::{
    cli::Project,
    model::{PendingOperation, ResourceId},
    util::ResultExt,
};

/// Application events.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Message {
    Tick,
    Quit,
    ClearNotes,
    KeyPress(KeyEvent),
    RefreshProjects,
    Resize,
    InitPending(ResourceId, PendingOperation),
    Projects(Result<Vec<Project>, String>),
    ActionResult(Result<String, String>),
}

impl<Err: Display> From<Result<String, Err>> for Message {
    fn from(value: Result<String, Err>) -> Self {
        Message::ActionResult(value.stringify_err())
    }
}

/// Terminal event handler.
#[derive(Debug)]
pub struct EventHandler<Msg> {
    /// Event sender channel.
    sender: mpsc::UnboundedSender<Msg>,
    /// Event receiver channel.
    receiver: mpsc::UnboundedReceiver<Msg>,
}

impl<Msg> Default for EventHandler<Msg> {
    fn default() -> Self {
        Self::new()
    }
}

impl<Msg> EventHandler<Msg> {
    /// Constructs a new instance of [`EventHandler`] and spawns a new thread to handle events.
    pub fn new() -> Self {
        let (sender, receiver) = mpsc::unbounded_channel();
        Self { sender, receiver }
    }

    pub async fn recv(&mut self) -> Option<Msg> {
        self.receiver.recv().await
    }

    pub async fn next(&mut self) -> color_eyre::Result<Msg> {
        self.receiver
            .recv()
            .await
            .ok_or_eyre("Failed to receive event")
    }

    pub fn send(&mut self, app_event: Msg) {
        let _ = self.sender.send(app_event);
    }

    pub fn sender(&self) -> mpsc::UnboundedSender<Msg> {
        self.sender.clone()
    }
}
