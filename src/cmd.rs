use async_trait::async_trait;

use crate::{cli::docker_compose_ls, event::Message};

#[async_trait]
pub trait Cmd<Model, Msg>: Send {
    async fn exec(&self, model: &Model) -> Msg;
}

pub struct DockerComposeLsCommand;

#[async_trait]
impl<Model> Cmd<Model, Message> for DockerComposeLsCommand {
    async fn exec(&self, _model: &Model) -> Message {
        if let Ok(projects) = docker_compose_ls().await {
            Message::Projects(Ok(projects))
        } else {
            Message::Projects(Err(()))
        }
    }
}

