use crate::{
    cmd::Cmd,
    event::{EventHandler, Message},
    model::{RunningState, UpdateResult},
    sub::Sub,
    tea::{self, Model},
};
use ratatui::DefaultTerminal;
use tokio_stream::StreamExt;

/// Application.
pub struct App {
    /// Counter.
    pub model: Model,
    /// Event handler.
    pub events: EventHandler,
    sub: Sub<Message>,
}

impl Default for App {
    fn default() -> Self {
        Self {
            model: Model::default(),
            events: EventHandler::new(),
            sub: Sub::new(),
        }
    }
}

impl App {
    /// Constructs a new instance of [`App`].
    pub fn new() -> Self {
        Self::default()
    }

    /// Run the application's main loop.
    pub async fn run(mut self, mut terminal: DefaultTerminal) -> color_eyre::Result<()> {
        self.update_subscriptions();
        self.process_subscriptions();
        self.process_keyboard_events();

        while self.model.running_state != RunningState::Done {
            let _ = terminal.draw(|frame| tea::view(&mut self.model, frame));

            let msg = self.events.next().await?;
            self.handle_message(msg);
        }

        Ok(())
    }

    /// Handle a message using pure update logic
    fn handle_message(&mut self, msg: Message) {
        let mut current_msg = Some(msg);

        while current_msg.is_some() {
            match tea::update(&mut self.model, current_msg.unwrap()) {
                UpdateResult::None => current_msg = None,
                UpdateResult::Msg(msg) => current_msg = Some(msg),
                UpdateResult::Cmd(cmd) => {
                    current_msg = None;
                    self.run_cmd(cmd)
                }
                UpdateResult::MsgAndCmd(msg, cmd) => {
                    current_msg = Some(msg);
                    self.run_cmd(cmd);
                }
            }

            self.update_subscriptions();
        }
    }

    /// Update subscriptions based on model
    fn update_subscriptions(&mut self) {
        let sub = tea::subscriptions(&self.model);

        self.sub.update(sub);
    }

    fn run_cmd(&self, cmd: Box<dyn Cmd<Message>>) {
        let sender = self.events.sender();

        tokio::spawn(async move {
            let msg = cmd.exec().await;
            let _ = sender.send(msg);
        });
    }

    fn process_subscriptions(&self) {
        let sender = self.events.sender();
        let mut sub_stream = self.sub.stream();

        tokio::spawn(async move {
            loop {
                while let Some(msg) = sub_stream.next().await {
                    let _ = sender.send(msg);
                }
            }
        });
    }

    fn process_keyboard_events(&self) {
        let sender = self.events.sender();
        tokio::spawn(async move {
            let mut reader = crossterm::event::EventStream::new();
            while let Some(Ok(evt)) = reader.next().await {
                if let crossterm::event::Event::Key(key) = evt {
                    let _ = sender.send(Message::KeyPress(key));
                }
            }
        });
    }
}
