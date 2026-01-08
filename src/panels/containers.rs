use crossterm::event::KeyEvent;
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Rect},
    style::{Color, Style},
    text::Line,
    widgets::{Block, BorderType, Cell, ListItem, Row, Table},
};

use crate::{
    bindings::{BINDINGS, KeyAction},
    cli::Container,
    cmd::{DockerActionTTY, DockerContainerListCommand},
    event::Message,
    model::{Action, Model, PanelId},
    ui::{colors::Colorize, table::TableStateExt},
};

impl From<&Container> for ListItem<'_> {
    fn from(value: &Container) -> Self {
        ListItem::new(Line::styled(
            format!("Container: {}", value.names),
            Color::Cyan,
        ))
    }
}

pub fn refresh_containers(model: &mut Model) -> Action<Message> {
    let mut args: Vec<String> = vec!["-a".to_string()];

    if let Some(project) = model.selected_project() {
        let label_filter = format!("label=com.docker.compose.project={}", project.name);
        args.push("--filter".to_string());
        args.push(label_filter);
    }

    Action::Cmd(Box::new(DockerContainerListCommand {
        msg_fn: Message::Containers,
        args,
    }))
}

pub fn handle_key(model: &mut Model, key: KeyEvent) -> Action<Message> {
    match BINDINGS.get(&key) {
        Some(KeyAction::MoveUp) => {
            model.containers_table_state.select_previous();
            Action::None
        }
        Some(KeyAction::MoveDown) => {
            model.containers_table_state.select_next();
            model.containers_table_state.fit(model.containers.len());
            Action::None
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
        _ => Action::None,
    }
}

pub fn view(model: &mut Model, frame: &mut Frame, area: Rect) {
    let block = Block::bordered()
        .title("[2] containers")
        .title_alignment(Alignment::Center)
        .border_type(BorderType::Rounded)
        .border_style(if model.active_panel == PanelId::Containers {
            Color::Green
        } else {
            Color::DarkGray
        });

    let max_name_len = model
        .containers
        .iter()
        .map(|container| container.names.len())
        .max()
        .unwrap_or(0) as u16;

    let rows: Vec<_> = model
        .containers
        .iter()
        .map(|container| {
            Row::new(vec![
                Cell::from(container.names.clone()),
                Cell::from(container.status.clone()),
            ])
            .style(container.colorize())
        })
        .collect();

    let table = Table::new(
        rows,
        [Constraint::Length(max_name_len + 1), Constraint::Fill(1)],
    )
    .block(block)
    .row_highlight_style(Style::new().bg(Color::Rgb(40, 40, 60)))
    .highlight_symbol(">");

    frame.render_stateful_widget(table, area, &mut model.containers_table_state);
}
