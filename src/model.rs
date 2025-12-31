use crate::{event::Message, tea::BoxedCmd};

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

pub enum UpdateResult {
    None,
    Msg(Message),
    Cmd(BoxedCmd),
    MsgAndCmd(Message, BoxedCmd)
}
