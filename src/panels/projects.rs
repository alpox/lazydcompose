use crossterm::event::KeyEvent;
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Rect},
    style::{Color, Style},
    widgets::{Block, BorderType, Cell, Row, Table},
};

use crate::{
    bindings::{BINDINGS, KeyAction},
    cmd::DockerAction,
    event::Message,
    model::{Action, Model, PanelId},
    ui::{colors::Colorize, table::TableStateExt},
};

pub fn handle_key(model: &mut Model, key: KeyEvent) -> Action<Message> {
    match BINDINGS.get(&key) {
        Some(KeyAction::MoveUp) => {
            model.projects_table_state.select_previous();
            Action::None
        }
        Some(KeyAction::MoveDown) => {
            model.projects_table_state.select_next();
            model.projects_table_state.fit(model.projects.len());
            Action::None
        }
        Some(KeyAction::DockerComposeStop) => {
            if let Some(project) = model.selected_project() {
                Action::Cmd(Box::new(DockerAction {
                    project,
                    args: vec!["compose".to_string(), "stop".to_string()],
                    msg_fn: Some(Message::DockerComposeStop),
                }))
            } else {
                Action::None
            }
        }
        Some(KeyAction::DockerComposeStart) => {
            if let Some(project) = model.selected_project() {
                Action::Cmd(Box::new(DockerAction {
                    project,
                    args: vec!["compose".to_string(), "start".to_string()],
                    msg_fn: Some(Message::DockerComposeStart),
                }))
            } else {
                Action::None
            }
        }
        Some(KeyAction::DockerComposeUp) => {
            if let Some(project) = model.selected_project() {
                Action::Cmd(Box::new(DockerAction {
                    project,
                    args: vec!["compose".to_string(), "up".to_string(), "-d".to_string()],
                    msg_fn: Some(Message::DockerComposeStop),
                }))
            } else {
                Action::None
            }
        }
        Some(KeyAction::DockerComposeDown) => {
            if let Some(project) = model.selected_project() {
                Action::Cmd(Box::new(DockerAction {
                    project,
                    args: vec!["compose".to_string(), "down".to_string()],
                    msg_fn: Some(Message::DockerComposeStart),
                }))
            } else {
                Action::None
            }
        }
        Some(KeyAction::DockerComposeRestart) => {
            if let Some(project) = model.selected_project() {
                Action::Cmd(Box::new(DockerAction {
                    project,
                    args: vec!["compose".to_string(), "restart".to_string()],
                    msg_fn: Some(Message::DockerComposeStart),
                }))
            } else {
                Action::None
            }
        }
        _ => Action::None,
    }
}

pub fn view(model: &mut Model, frame: &mut Frame, area: Rect) {
    // let mut offset = 0;

    // for project in model.projects {
    //     let block = Block::new()
    //         .borders(Borders::ALL)
    //         .title(project.name.clone())
    //         .border_style(Color::Cyan);
    // }

    let block = Block::bordered()
        .title("[1] projects")
        .title_alignment(Alignment::Center)
        .border_type(BorderType::Rounded)
        .border_style(if model.active_panel == PanelId::Projects {
            Color::Green
        } else {
            Color::DarkGray
        });

    let max_name_len = model
        .projects
        .iter()
        .map(|project| project.name.len())
        .max()
        .unwrap_or(0) as u16;

    let rows: Vec<_> = model
        .projects
        .iter()
        .map(|project| {
            Row::new(vec![
                Cell::from(project.name.clone()),
                Cell::from(
                    project
                        .kind
                        .as_compose()
                        .map(|c| c.status.clone())
                        .unwrap_or("".to_string()),
                ),
            ])
            .style(project.colorize())
        })
        .collect();

    let table = Table::new(
        rows,
        [Constraint::Length(max_name_len + 1), Constraint::Fill(1)],
    )
    .block(block)
    .row_highlight_style(Style::new().bg(Color::Rgb(40, 40, 60)))
    .highlight_symbol(">");

    frame.render_stateful_widget(table, area, &mut model.projects_table_state);
}
