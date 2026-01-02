use async_trait::async_trait;
use color_eyre::eyre::Error;

use crate::cli::{Container, Project, docker_compose_ls, docker_container_list};

#[async_trait]
pub trait Cmd<Msg>: Send + Sync {
    async fn exec(&self) -> Msg;
}

pub type BoxedCmd<Msg> = Box<dyn Cmd<Msg>>;

pub struct MappedCmd<Msg, F> {
    original: Box<dyn Cmd<Msg> + Send + Sync>,
    map_fn: F,
}

#[async_trait]
impl<Msg, NewMsg, F> Cmd<NewMsg> for MappedCmd<Msg, F>
where
    Msg: Send + 'static,
    NewMsg: Send + 'static,
    F: Fn(Msg) -> NewMsg + Send + Sync,
{
    async fn exec(&self) -> NewMsg {
        let result = self.original.exec().await;
        (self.map_fn)(result)
    }
}

pub fn map_cmd<Msg, NewMsg, F>(cmd: BoxedCmd<Msg>, f: F) -> BoxedCmd<NewMsg>
where
    Msg: Send + 'static,
    NewMsg: Send + 'static,
    F: Fn(Msg) -> NewMsg + Send + Sync + 'static,
{
    Box::new(MappedCmd {
        original: cmd,
        map_fn: f,
    })
}

pub struct DockerComposeLsCommand<Msg>(pub fn(Result<Vec<Project>, ()>) -> Msg);

#[async_trait]
impl<Msg> Cmd<Msg> for DockerComposeLsCommand<Msg> {
    async fn exec(&self) -> Msg {
        match docker_compose_ls().await {
            Ok(projects) => self.0(Ok(projects)),
            Err(_) => self.0(Err(())),
        }
    }
}

pub struct DockerContainerListCommand<Msg> {
    pub args: Vec<String>,
    pub msg_fn: fn(Result<Vec<Container>, Error>) -> Msg,
}

#[async_trait]
impl<Msg> Cmd<Msg> for DockerContainerListCommand<Msg> {
    async fn exec(&self) -> Msg {
        let args = self.args.iter().map(String::as_str);
        match docker_container_list(args).await {
            Ok(containers) => (self.msg_fn)(Ok(containers)),
            Err(err) => (self.msg_fn)(Err(err)),
        }
    }
}
