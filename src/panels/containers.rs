use crossterm::event::KeyEvent;
use ratatui::{
    Frame,
    layout::{Constraint, Rect},
    style::{Color, Style},
    text::Line,
    widgets::{Cell, ListItem, Row, Table, TableState},
};

use crate::{
    bindings::{BINDINGS, KeyAction},
    cli::Container,
    cmd::{DockerAction, DockerActionTTY},
    event::Message,
    model::{Action, ContextId, Model},
    ui::colors::Colorize,
};

impl From<&Container> for ListItem<'_> {
    fn from(value: &Container) -> Self {
        ListItem::new(Line::styled(
            format!("Container: {}", value.names),
            Color::Cyan,
        ))
    }
}

pub fn handle_key(model: &mut Model, key: KeyEvent) -> Action<Message> {
    match BINDINGS.get_for_context(&key, model.active_context) {
        Some(KeyAction::MoveUp) => {
            model.select_previous_container();
            Action::None
        }
        Some(KeyAction::MoveDown) => {
            model.select_next_container();
            Action::None
        }
        Some(KeyAction::DockerContainerStart) => {
            match (model.selected_project(), model.selected_container()) {
                (Some(project), Some(container)) => Action::Cmd(Box::new(DockerAction {
                    project,
                    args: vec![
                        "container".to_string(),
                        "start".to_string(),
                        container.names,
                    ],
                    msg_fn: None,
                })),
                _ => Action::None,
            }
        }
        Some(KeyAction::DockerContainerStop) => {
            match (model.selected_project(), model.selected_container()) {
                (Some(project), Some(container)) => Action::Cmd(Box::new(DockerAction {
                    project,
                    args: vec!["container".to_string(), "stop".to_string(), container.names],
                    msg_fn: None,
                })),
                _ => Action::None,
            }
        }
        Some(KeyAction::DockerContainerRestart) => {
            match (model.selected_project(), model.selected_container()) {
                (Some(project), Some(container)) => Action::Cmd(Box::new(DockerAction {
                    project,
                    args: vec![
                        "container".to_string(),
                        "restart".to_string(),
                        container.names,
                    ],
                    msg_fn: None,
                })),
                _ => Action::None,
            }
        }
        Some(KeyAction::DockerFollowLogs) => {
            match (model.selected_project(), model.selected_container()) {
                (Some(project), Some(container)) => {
                    Action::BlockingCmd(Box::new(DockerActionTTY {
                        project,
                        args: vec![
                            "logs".to_string(),
                            "--follow".to_string(),
                            "--since=60m".to_string(),
                            container.id,
                        ],
                        msg_fn: None,
                    }))
                }
                _ => Action::None,
            }
        }
        Some(KeyAction::DockerConsole) => {
            match (model.selected_project(), model.selected_container()) {
                (Some(project), Some(container)) => {
                    Action::BlockingCmd(Box::new(DockerActionTTY {
                        project,
                        args: vec![
                            "exec".to_string(),
                            "-it".to_string(),
                            container.id,
                            "/bin/sh".to_string(),
                        ],
                        msg_fn: None,
                    }))
                }
                _ => Action::None,
            }
        }
        Some(KeyAction::Deselect) => {
            model.active_context = ContextId::Projects;
            model.active_container_index = None;
            Action::None
        }
        _ => Action::None,
    }
}

pub fn view(model: &mut Model, project_index: usize, frame: &mut Frame, area: Rect) {
    let project = &model.projects[project_index];

    let max_name_len = project
        .containers
        .iter()
        .map(|container| container.title().len())
        .max()
        .unwrap_or(0) as u16;

    let rows: Vec<_> = project
        .containers
        .iter()
        .map(|container| {
            Row::new(vec![
                Cell::from(container.title()),
                Cell::from(container.status.clone()),
            ])
            .style(container.colorize())
        })
        .collect();

    let table = Table::new(
        rows,
        [Constraint::Length(max_name_len + 1), Constraint::Fill(1)],
    )
    .row_highlight_style(Style::new().bg(Color::Rgb(50, 60, 90)).bold())
    .highlight_symbol("▶ ");

    let mut table_state = TableState::new();

    let is_active = model.active_context == ContextId::Containers
        && model
            .active_project_index
            .map(|active| active == project_index)
            .unwrap_or(false);

    if is_active {
        table_state
            .select(model.project_container_index(project_index, model.active_container_index));
    }

    frame.render_stateful_widget(table, area, &mut table_state);
}
