use std::{cmp::min, time::Duration};

use crossterm::event::{KeyCode, KeyEvent};
use ratatui::widgets::ListState;

use crate::{
    cli::Project,
    cmd::DockerComposeLsCommand,
    model::{PanelId, UpdateResult}, subs::Subscription,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Message {
    Up,
    Down,
    RefreshProjects,
    Projects(Result<Vec<Project>, ()>),
    KeyPress(KeyEvent),
}

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

pub fn update(model: &mut ProjectsPanel, msg: Message) -> UpdateResult<Message> {
    match msg {
        Message::RefreshProjects => UpdateResult::Cmd(Box::new(DockerComposeLsCommand(Message::Projects))),
        Message::Projects(Ok(projects)) => {
            model
                .list_state
                .select(match (model.list_state.selected(), projects.len()) {
                    (Some(idx), pl) => Some(min(idx, pl - 1)),
                    (_, 0) => None,
                    _ => Some(0),
                });
            model.projects = projects;
            UpdateResult::None
        }
        Message::Projects(Err(_)) => UpdateResult::None,
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

pub fn map_key_to_message(_model: &ProjectsPanel, key: KeyEvent) -> UpdateResult<Message> {
    match key.code {
        KeyCode::Up | KeyCode::Char('k') => UpdateResult::Msg(Message::Up),
        KeyCode::Down | KeyCode::Char('j') => UpdateResult::Msg(Message::Down),
        _ => UpdateResult::None
    }
}

pub fn subscriptions(_model: &ProjectsPanel) -> Subscription<Message> {
    Subscription::Interval(Duration::from_secs(2), Message::RefreshProjects)
}
