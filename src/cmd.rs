use std::fmt::Display;

use async_trait::async_trait;

use crate::cli::{Container, Project, docker_compose_ls, docker_container_list};

pub trait ResultExt<T> {
    fn stringify_err(self) -> Result<T, String>;
}

impl<T, E: Display> ResultExt<T> for Result<T, E> {
    fn stringify_err(self) -> Result<T, String> {
        self.map_err(|e| e.to_string())
    }
}

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

pub struct DockerComposeLsCommand<Msg>(pub fn(Result<Vec<Project>, String>) -> Msg);

#[async_trait]
impl<Msg> Cmd<Msg> for DockerComposeLsCommand<Msg> {
    async fn exec(&self) -> Msg {
        self.0(docker_compose_ls().await.stringify_err())
    }
}

pub struct DockerContainerListCommand<Msg> {
    pub args: Vec<String>,
    pub msg_fn: fn(Result<Vec<Container>, String>) -> Msg,
}

#[async_trait]
impl<Msg> Cmd<Msg> for DockerContainerListCommand<Msg> {
    async fn exec(&self) -> Msg {
        let args = self.args.iter().map(String::as_str);
        (self.msg_fn)(docker_container_list(args).await.stringify_err())
    }
}
