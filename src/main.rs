use std::sync::{Arc, atomic::AtomicBool};

use color_eyre::eyre::OptionExt;
use config::Config;
use dirs::config_local_dir;
use signal_hook::{consts::SIGINT, flag};

use crate::{
    app::App,
    bindings::{BINDINGS, KeyBindings},
    log::initialize_logging,
    settings::Settings,
};

pub mod app;
pub mod app_mode;
pub mod bindings;
pub mod cli;
pub mod effect;
pub mod event;
pub mod log;
pub mod model;
pub mod panels;
pub mod settings;
pub mod sub_manager;
pub mod subs;
pub mod tea;
pub mod ui;
pub mod util;

fn read_settings() -> color_eyre::Result<Settings> {
    let mut dir = config_local_dir().ok_or_eyre("No configuration directory found")?;

    dir.push(env!("CARGO_PKG_NAME"));
    dir.push("config.toml");

    let config = Config::builder()
        .add_source(config::File::from(dir).required(false))
        .build()?;

    let settings = config.try_deserialize::<Settings>()?;

    Ok(settings)
}

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    let sigint_flag = Arc::new(AtomicBool::new(false));
    flag::register(SIGINT, Arc::clone(&sigint_flag))?;

    color_eyre::install()?;
    initialize_logging()?;

    let settings = match read_settings() {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Invalid config: {e:#}");
            std::process::exit(1)
        }
    };

    let mut bindings = KeyBindings::default();
    bindings.apply_config(settings);
    BINDINGS.set(bindings).ok().expect("BINDINGS already set");

    let terminal = ratatui::init();
    let result = App::new(sigint_flag).run(terminal).await;
    ratatui::restore();
    result
}
