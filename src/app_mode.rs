use tokio::sync::watch;

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub enum AppMode {
    #[default]
    Tui,
    Tty,
}

pub struct AppModeManager {
    tx: watch::Sender<AppMode>,
    rx: watch::Receiver<AppMode>,
}

impl Default for AppModeManager {
    fn default() -> Self {
        Self::new()
    }
}

impl AppModeManager {
    pub fn new() -> Self {
        let (tx, rx) = watch::channel(AppMode::default());
        Self { tx, rx }
    }

    pub fn set(&self, mode: AppMode) {
        let _ = self.tx.send(mode);
    }

    pub fn get(&self) -> AppMode {
        *self.rx.borrow()
    }

    pub fn subscribe(&self) -> AppModeSubscriber {
        AppModeSubscriber {
            rx: self.rx.clone(),
        }
    }
}

pub struct AppModeSubscriber {
    rx: watch::Receiver<AppMode>,
}

impl AppModeSubscriber {
    pub fn get(&self) -> AppMode {
        *self.rx.borrow()
    }

    pub async fn changed(&mut self) -> Option<AppMode> {
        self.rx.changed().await.ok()?;
        Some(*self.rx.borrow())
    }

    pub async fn wait_for(&mut self, target: AppMode) -> bool {
        loop {
            if self.get() == target {
                return true;
            }

            if self.changed().await.is_none() {
                return false;
            }
        }
    }
}
