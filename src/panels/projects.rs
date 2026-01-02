use std::{cmp::min, time::Duration};

use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    Frame,
    layout::{Alignment, Rect},
    style::{Color, Style},
    text::Line,
    widgets::{Block, BorderType, HighlightSpacing, List, ListItem, ListState},
};

use crate::{
    cli::Project,
    cmd::DockerComposeLsCommand,
    model::{Action, ChildAction, PanelId},
    subs::Subscription,
};

impl From<&Project> for ListItem<'_> {
    fn from(value: &Project) -> Self {
        ListItem::new(Line::styled(
            format!("Project: {}", value.name),
            Color::Cyan,
        ))
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Message {
    RefreshProjects,
    Projects(Result<Vec<Project>, ()>),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum OutMessage {
    ProjectChanged(Project),
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

impl ProjectsPanel {
    pub fn selected_project(&self) -> Option<Project> {
        self.list_state
            .selected()
            .and_then(|index| self.projects.get(index))
            .cloned()
    }
}

pub fn update(model: &mut ProjectsPanel, msg: Message) -> ChildAction<Message, OutMessage> {
    match msg {
        Message::RefreshProjects => {
            let cmd = Box::new(DockerComposeLsCommand(Message::Projects));
            ChildAction::new(Action::Cmd(cmd))
        }
        Message::Projects(Ok(projects)) => {
            model
                .list_state
                .select(match (model.list_state.selected(), projects.len()) {
                    (Some(idx), pl) => Some(min(idx, pl - 1)),
                    (_, 0) => None,
                    _ => Some(0),
                });
            model.projects = projects;
            match model.selected_project() {
                Some(project) => ChildAction::out(OutMessage::ProjectChanged(project)),
                None => ChildAction::none(),
            }
        }
        Message::Projects(Err(_)) => ChildAction::none(),
    }
}

pub fn handle_key(model: &mut ProjectsPanel, key: KeyEvent) -> ChildAction<Message, OutMessage> {
    match key.code {
        KeyCode::Up | KeyCode::Char('k') => {
            model.list_state.select_previous();
            ChildAction::none()
        }
        KeyCode::Down | KeyCode::Char('j') => {
            model.list_state.select_next();
            ChildAction::none()
        }
        _ => ChildAction::none(),
    }
}

pub fn subscriptions(_model: &ProjectsPanel) -> Subscription<Message> {
    Subscription::Interval(Duration::from_secs(2), Message::RefreshProjects)
}

pub fn view(model: &mut ProjectsPanel, frame: &mut Frame, area: Rect) {
    let block = Block::bordered()
        .title("lazydcompose")
        .title_alignment(Alignment::Center)
        .border_type(BorderType::Rounded);

    let items = model.projects.iter().map(ListItem::from);

    let list = List::new(items)
        .block(block)
        .highlight_style(Style::new().bg(Color::DarkGray))
        .highlight_symbol(">")
        .highlight_spacing(HighlightSpacing::Always);

    frame.render_stateful_widget(list, area, &mut model.list_state);
}
