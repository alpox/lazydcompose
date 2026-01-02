use crate::{
    cmd::{BoxedCmd, map_cmd},
    panels::{containers::ContainersPanel, projects::ProjectsPanel},
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
    pub running_state: RunningState,
    pub projects_panel: ProjectsPanel,
    pub containers_panel: ContainersPanel,
    pub active_panel: PanelId,
}

impl Default for Model {
    fn default() -> Self {
        Self {
            running_state: RunningState::Running,
            active_panel: PanelId::default(),
            projects_panel: ProjectsPanel::default(),
            containers_panel: ContainersPanel::default(),
        }
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

pub struct ChildAction<Msg, OutMsg>(
    pub Action<Msg>,
    pub Option<OutMsg>
);

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
            None => self.0
        }
    }

    pub fn into_inner(self) -> Action<Msg> {
        self.0
    }
}
