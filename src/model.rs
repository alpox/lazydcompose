use ratatui::widgets::ListState;

use crate::cli::Project;

const PROJECT_PANEL: &str = "projects";

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub enum RunningState {
    #[default]
    Running,
    Done,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProjectsPanel {
    pub name: String,
    pub projects: Vec<Project>,
    pub list_state: ListState,
}

impl Default for ProjectsPanel {
    fn default() -> Self {
        Self {
            name: PROJECT_PANEL.to_string(),
            list_state: ListState::default(),
            projects: vec![],
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Model {
    pub running_state: RunningState,
    pub counter: u8,
    pub projects_panel: ProjectsPanel,
    pub current_panel: String,
}

impl Default for Model {
    fn default() -> Self {
        Self {
            running_state: RunningState::Running,
            counter: 0,
            current_panel: PROJECT_PANEL.to_string(),
            projects_panel: ProjectsPanel::default(),
        }
    }
}

