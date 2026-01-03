use ratatui::widgets::ListState;

use crate::{
    cli::{Container, Project},
    cmd::{BoxedCmd, map_cmd},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum PanelId {
    #[default]
    Projects,
    Containers,
    Logs,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum PanelSize {
    #[default]
    Normal,
    Expanded,
    Collapsed,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub enum RunningState {
    #[default]
    Running,
    Done,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Model {
    // Application data
    pub running_state: RunningState,
    pub projects: Vec<Project>,
    pub containers: Vec<Container>,

    // UI state
    pub active_panel: PanelId,
    pub projects_list_state: ListState,
    pub containers_list_state: ListState,
}

impl Default for Model {
    fn default() -> Self {
        Self {
            running_state: RunningState::Running,
            projects: vec![],
            containers: vec![],

            active_panel: PanelId::default(),
            projects_list_state: ListState::default(),
            containers_list_state: ListState::default(),
        }
    }
}

impl Model {
    pub fn selected_project(&self) -> Option<Project> {
        self.projects_list_state
            .selected()
            .and_then(|index| self.projects.get(index))
            .cloned()
    }

    pub fn selected_container(&self) -> Option<Container> {
        self.containers_list_state
            .selected()
            .and_then(|index| self.containers.get(index))
            .cloned()
    }
}

pub enum Action<Msg> {
    None,
    Cmd(BoxedCmd<Msg>),
}

impl<Msg> Action<Msg> {
    pub fn map<F, NewMsg>(self, f: F) -> Action<NewMsg>
    where
        Msg: Send + 'static,
        NewMsg: Send + 'static,
        F: Fn(Msg) -> NewMsg + Clone + Send + Sync + 'static,
    {
        match self {
            Action::None => Action::None,
            Action::Cmd(cmd) => Action::Cmd(map_cmd(cmd, f)),
        }
    }
}

pub struct ChildAction<Msg, OutMsg>(pub Action<Msg>, pub Option<OutMsg>);

pub trait ChildActionAdaptor<Model, Msg> {
    fn adapt(self, model: &mut Model) -> Action<Msg>;
}

impl<Msg, OutMsg> ChildAction<Msg, OutMsg> {
    pub fn none() -> Self {
        ChildAction(Action::None, None)
    }

    pub fn out(out: OutMsg) -> Self {
        ChildAction(Action::None, Some(out))
    }

    pub fn new(result: Action<Msg>) -> Self {
        ChildAction(result, None)
    }

    pub fn map_msg<F, NewMsg>(self, f: F) -> ChildAction<NewMsg, OutMsg>
    where
        Msg: Send + 'static,
        NewMsg: Send + 'static,
        F: Fn(Msg) -> NewMsg + Clone + Send + Sync + 'static,
    {
        ChildAction(self.0.map(f), self.1)
    }

    pub fn handle_out<F>(self, handler: F) -> Action<Msg>
    where
        F: FnOnce(OutMsg) -> Action<Msg>,
    {
        match self.1 {
            Some(out) => handler(out),
            None => self.0,
        }
    }

    pub fn into_inner(self) -> Action<Msg> {
        self.0
    }
}
