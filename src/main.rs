use crate::{log::initialize_logging, app::App};

pub mod app;
pub mod event;
pub mod cli;
pub mod subs;
pub mod log;
pub mod cmd;
pub mod tea;
pub mod model;

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    initialize_logging()?;
    let terminal = ratatui::init();
    let result = App::new().run(terminal).await;
    ratatui::restore();
    result
}
