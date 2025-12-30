use crate::{
    cmd::{Cmd},
    event::{EventHandler, Message},
    model::{Model, RunningState},
    subs::{Sub},
    tea,
};
use ratatui::{
    DefaultTerminal,
    crossterm::event::{KeyCode, KeyEvent, KeyModifiers},
};
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
        let model = Model {
            running_state: RunningState::Running,
            counter: 0,
            projects: vec![],
        };

        Self {
            model,
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
            terminal.draw(|frame| frame.render_widget(&self.model, frame.area()))?;

            let msg = self.events.next().await?;
            self.handle_message(msg);
        }

        Ok(())
    }

    /// Handle a message using pure update logic
    fn handle_message(&mut self, msg: Message) {
        let mut current_msg = Some(msg);

        while current_msg.is_some() {
            let (next_msg, next_cmd) = tea::update(&mut self.model, current_msg.unwrap());
            current_msg = next_msg;

            if let Some(cmd) = next_cmd {
                self.run_cmd(cmd);
            }

            self.update_subscriptions();
        }
    }

    /// Update subscriptions based on model
    fn update_subscriptions(&mut self) {
        let sub = tea::subscriptions(&self.model);

        self.sub.update(sub);
    }

    fn run_cmd(&self, cmd: Box<dyn Cmd<Model, Message>>) {
        let sender = self.events.sender();
        let model = self.model.clone();

        tokio::spawn(async move {
            let msg = cmd.exec(&model).await;
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

    /// Handles the key events and updates the state of [`App`].
    pub fn handle_key_events(&mut self, key_event: KeyEvent) -> color_eyre::Result<()> {
        match key_event.code {
            KeyCode::Esc | KeyCode::Char('q') => self.events.send(Message::Quit),
            KeyCode::Char('c' | 'C') if key_event.modifiers == KeyModifiers::CONTROL => {
                self.events.send(Message::Quit)
            }
            KeyCode::Right => self.events.send(Message::Increment),
            KeyCode::Left => self.events.send(Message::Decrement),
            // Other handlers you could add here.
            _ => {}
        }
        Ok(())
    }
}
