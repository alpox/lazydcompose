use std::{
    io::stdout,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    thread::sleep,
    time::Duration,
};

use crate::{
    app_mode::{AppMode, AppModeManager},
    cmd::Cmd,
    event::{EventHandler, Message},
    model::{Action, Model, RunningState},
    sub_manager::SubscriptionManager,
    tea::{self},
};
use crossterm::{
    cursor::Show,
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use futures::executor::block_on;
use ratatui::DefaultTerminal;
use tokio::task::spawn_blocking;
use tokio_stream::StreamExt;

/// Application.
pub struct App {
    /// Counter.
    pub model: Model,
    /// Event handler.
    pub events: EventHandler<Message>,
    sub: SubscriptionManager<Message>,
    sigint_flag: Arc<AtomicBool>,
    app_mode: AppModeManager,
}

impl Default for App {
    fn default() -> Self {
        Self {
            model: Model::default(),
            events: EventHandler::new(),
            sub: SubscriptionManager::new(),
            sigint_flag: Arc::default(),
            app_mode: AppModeManager::new(),
        }
    }
}

impl App {
    /// Constructs a new instance of [`App`].
    pub fn new(sigint_flag: Arc<AtomicBool>) -> Self {
        Self {
            sigint_flag,
            ..Self::default()
        }
    }

    pub async fn run(mut self, mut terminal: DefaultTerminal) -> color_eyre::Result<()> {
        self.update_subscriptions();
        self.process_subscriptions();
        self.process_keyboard_events();

        let _ = terminal.draw(|frame| tea::view(&mut self.model, frame));

        while self.model.running_state != RunningState::Done {
            if self.sigint_flag.swap(false, Ordering::Relaxed) {
                break;
            }
            let msg = self.events.next().await?;

            if self.handle_message(msg)? {
                terminal.clear()?;
            }

            let _ = terminal.draw(|frame| tea::view(&mut self.model, frame));
        }

        Ok(())
    }

    fn handle_message(&mut self, msg: Message) -> color_eyre::Result<bool> {
        let mut must_redraw = false;

        match tea::update(&mut self.model, msg) {
            Action::None => {}
            Action::Cmd(cmd) => self.run_cmd(cmd),
            Action::BlockingCmd(cmd) => must_redraw = self.run_cmd_blocking(cmd)?,
        }

        self.update_subscriptions();

        Ok(must_redraw)
    }

    fn update_subscriptions(&mut self) {
        let sub = tea::subscriptions(&self.model);

        self.sub.update(sub);
    }

    fn run_cmd(&self, cmd: impl Cmd<Msg = Message> + 'static) {
        let sender = self.events.sender();

        tokio::spawn(async move {
            if let Some(msg) = cmd.exec().await {
                let _ = sender.send(msg);
            }
        });
    }

    fn run_cmd_blocking(&self, cmd: impl Cmd<Msg = Message> + 'static) -> color_eyre::Result<bool> {
        let sender = self.events.sender();

        self.app_mode.set(AppMode::Tty);
        // Wait for keyboard input swap
        sleep(Duration::from_millis(20));

        execute!(stdout(), LeaveAlternateScreen, Show)?;
        disable_raw_mode()?;

        let handle = spawn_blocking(move || {
            block_on(async move {
                if let Some(msg) = cmd.exec().await {
                    let _ = sender.send(msg);
                }
            })
        });

        block_on(handle)?;

        self.sigint_flag.store(false, Ordering::Relaxed);

        enable_raw_mode()?;
        execute!(stdout(), EnterAlternateScreen)?;

        self.app_mode.set(AppMode::Tui);

        Ok(true)
    }

    fn process_subscriptions(&self) {
        let sender = self.events.sender();
        let mut sub_stream = self.sub.stream();

        tokio::spawn(async move {
            while let Some(msg) = sub_stream.next().await {
                let _ = sender.send(msg);
            }
        });
    }

    fn process_keyboard_events(&self) {
        let sender = self.events.sender();
        let mut app_mode_sub = self.app_mode.subscribe();

        tokio::spawn(async move {
            loop {
                if !app_mode_sub.wait_for(AppMode::Tui).await {
                    break;
                }

                let mut reader = crossterm::event::EventStream::new();
                loop {
                    tokio::select! {
                        Some(AppMode::Tty) = async { app_mode_sub.changed().await } => {
                            break;
                        },
                        Some(Ok(evt)) = async { reader.next().await } => {
                            match evt {
                                crossterm::event::Event::Key(key) => {
                                    let _ = sender.send(Message::KeyPress(key));
                                },
                                crossterm::event::Event::Resize(_, _) => {
                                    let _ = sender.send(Message::Resize);
                                }
                                _ => {}
                            }
                        }
                    }
                }
            }
        });
    }
}
