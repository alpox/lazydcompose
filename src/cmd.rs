use std::{fmt::Display, ops::Deref, process::ExitStatus};

use async_trait::async_trait;

use crate::cli::{
    Project, docker_action_tty, docker_get_projects, docker_project_action
};

pub trait ResultExt<T> {
    fn stringify_err(self) -> Result<T, String>;
}

impl<T, E: Display> ResultExt<T> for Result<T, E> {
    fn stringify_err(self) -> Result<T, String> {
        self.map_err(|e| e.to_string())
    }
}

#[async_trait]
pub trait Cmd: Send + Sync {
    type Msg;
    async fn exec(&self) -> Option<Self::Msg>;
}

pub type BoxedCmd<Msg> = Box<dyn Cmd<Msg = Msg>>;

#[async_trait]
impl<Msg> Cmd for BoxedCmd<Msg> {
    type Msg = Msg;
    async fn exec(&self) -> Option<Msg> {
        return self.deref().exec().await;
    }
}

pub struct MappedCmd<C, F>
where
    C: Cmd,
{
    original: C,
    map_fn: F,
}

#[async_trait]
impl<C, NewMsg, F> Cmd for MappedCmd<C, F>
where
    C: Cmd + Sync,
    NewMsg: Send,
    F: Fn(C::Msg) -> NewMsg + Send + Sync,
{
    type Msg = NewMsg;
    async fn exec(&self) -> Option<NewMsg> {
        let result = self.original.exec().await?;
        Some((self.map_fn)(result))
    }
}

pub fn map_cmd<C, NewMsg, F>(cmd: C, f: F) -> MappedCmd<C, F>
where
    C: Cmd,
    F: Fn(C::Msg) -> NewMsg + Send + Sync + 'static,
{
    MappedCmd {
        original: cmd,
        map_fn: f,
    }
}

pub struct DockerGetProjectsCommand<Msg>(pub fn(Result<Vec<Project>, String>) -> Msg);

#[async_trait]
impl<Msg> Cmd for DockerGetProjectsCommand<Msg> {
    type Msg = Msg;
    async fn exec(&self) -> Option<Msg> {
        Some(self.0(docker_get_projects().await.stringify_err()))
    }
}

pub struct DockerAction<Msg> {
    pub project: Project,
    pub args: Vec<String>,
    pub msg_fn: Option<fn(Result<String, String>) -> Msg>,
}

#[async_trait]
impl<Msg> Cmd for DockerAction<Msg> {
    type Msg = Msg;
    async fn exec(&self) -> Option<Msg> {
        let args = self.args.iter().map(String::as_str);

        let result = docker_project_action(self.project.clone(), args)
            .await
            .stringify_err();

        self.msg_fn.map(|f| f(result))
    }
}

pub struct DockerActionTTY<Msg> {
    pub project: Project,
    pub args: Vec<String>,
    pub msg_fn: Option<fn(Result<ExitStatus, String>) -> Msg>,
}

#[async_trait]
impl<Msg> Cmd for DockerActionTTY<Msg> {
    type Msg = Msg;
    async fn exec(&self) -> Option<Msg> {
        let args = self.args.iter().map(String::as_str);

        let result = docker_action_tty(self.project.clone(), args)
            .await
            .stringify_err();

        self.msg_fn.map(|f| f(result))
    }
}
