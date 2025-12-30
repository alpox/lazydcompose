use crate::cli::Project;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub enum RunningState {
    #[default]
    Running,
    Done,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Model {
    pub running_state: RunningState,
    pub counter: u8,
    pub projects: Vec<Project>
}
