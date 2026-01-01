use crate::cmd::{BoxedCmd, map_cmd};

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

pub enum UpdateResult<Msg> {
    None,
    Msg(Msg),
    Cmd(BoxedCmd<Msg>),
    MsgAndCmd(Msg, BoxedCmd<Msg>),
}

impl<Msg> UpdateResult<Msg>
{
    pub fn map<F, NewMsg>(self, f: F) -> UpdateResult<NewMsg>
    where
        Msg: Send + 'static,
        NewMsg: Send + 'static,
        F: Fn(Msg) -> NewMsg + Clone + Send + Sync + 'static,
    {
        match self {
            UpdateResult::None => UpdateResult::None,
            UpdateResult::Msg(msg) => UpdateResult::Msg(f(msg)),
            UpdateResult::Cmd(cmd) => UpdateResult::Cmd(map_cmd(cmd, f)),
            UpdateResult::MsgAndCmd(msg, cmd) => UpdateResult::MsgAndCmd(f(msg), map_cmd(cmd, f)),
        }
    }
}
