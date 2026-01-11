use std::sync::{Arc, atomic::AtomicBool};

use signal_hook::{consts::SIGINT, flag};

use crate::{log::initialize_logging, app::App};

pub mod app;
pub mod app_mode;
pub mod event;
pub mod cli;
pub mod sub_manager;
pub mod subs;
pub mod log;
pub mod cmd;
pub mod tea;
pub mod model;
pub mod panels;
pub mod bindings;
pub mod ui;
pub mod util;

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    let sigint_flag = Arc::new(AtomicBool::new(false));
    flag::register(SIGINT, Arc::clone(&sigint_flag))?;

    color_eyre::install()?;
    initialize_logging()?;
    let terminal = ratatui::init();
    let result = App::new(sigint_flag).run(terminal).await;
    ratatui::restore();
    result
}
