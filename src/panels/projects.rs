use ratatui::widgets::ListState;

use crate::{
    cli::Project,
    event::Message,
    model::{PanelId, UpdateResult},
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProjectsPanel {
    pub id: PanelId,
    pub projects: Vec<Project>,
    pub list_state: ListState,
}

impl Default for ProjectsPanel {
    fn default() -> Self {
        Self {
            id: PanelId::Projects,
            list_state: ListState::default(),
            projects: vec![],
        }
    }
}

pub fn update(model: &mut ProjectsPanel, msg: Message) -> UpdateResult {
    match msg {
        Message::Up => {
            model.list_state.select_previous();
            UpdateResult::None
        }
        Message::Down => {
            model.list_state.select_next();
            UpdateResult::None
        }
        _ => UpdateResult::None,
    }
}
