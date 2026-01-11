use crossterm::event::KeyEvent;
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::Color,
    widgets::{Block, BorderType, Borders},
};

use crate::{
    bindings::{BINDINGS, KeyAction},
    cmd::DockerAction,
    event::Message,
    model::{Action, Model},
    panels::containers::{self},
    util::wrap_around_optional,
};

pub fn handle_key(model: &mut Model, key: KeyEvent) -> Action<Message> {
    match BINDINGS.get_for_context(&key, model.active_context) {
        Some(KeyAction::MoveUp) => {
            model.active_project_index = wrap_around_optional(
                model.active_project_index,
                -1,
                model.projects.len().saturating_sub(1),
            );
            Action::None
        }
        Some(KeyAction::MoveDown) => {
            model.active_project_index = wrap_around_optional(
                model.active_project_index,
                1,
                model.projects.len().saturating_sub(1),
            );
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
        Some(KeyAction::Select) => {
            model.select_project(model.active_project_index);
            Action::None
        }
        _ => Action::None,
    }
}

pub fn view(model: &mut Model, frame: &mut Frame, area: Rect) {
    // let mut offset = 0;
    //
    let constraints = model
        .projects
        .iter()
        .map(|project| Constraint::Length(project.containers.len() as u16 + 2));

    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints)
        .split(area);

    for (i, area) in layout.iter().enumerate() {
        let project = &model.projects[i];

        let block = Block::new()
            .borders(Borders::ALL)
            .border_type(
                if let Some(idx) = model.active_project_index
                    && idx == i
                {
                    BorderType::Double
                } else {
                    BorderType::Rounded
                },
            )
            .title(project.name.clone())
            .border_style(Color::Cyan);

        let inner = block.inner(*area);
        frame.render_widget(block, *area);

        containers::view(model, i, frame, inner);
    }

    // let block = Block::bordered()
    //     .title("[1] projects")
    //     .title_alignment(Alignment::Center)
    //     .border_type(BorderType::Rounded)
    //     .border_style(if model.active_panel == PanelId::Projects {
    //         Color::Green
    //     } else {
    //         Color::DarkGray
    //     });
    //
    // let max_name_len = model
    //     .projects
    //     .iter()
    //     .map(|project| project.name.len())
    //     .max()
    //     .unwrap_or(0) as u16;
    //
    // let rows: Vec<_> = model
    //     .projects
    //     .iter()
    //     .map(|project| {
    //         Row::new(vec![
    //             Cell::from(project.name.clone()),
    //             Cell::from(
    //                 project
    //                     .kind
    //                     .as_compose()
    //                     .map(|c| c.status.clone())
    //                     .unwrap_or("".to_string()),
    //             ),
    //         ])
    //         .style(project.colorize())
    //     })
    //     .collect();
    //
    // let table = Table::new(
    //     rows,
    //     [Constraint::Length(max_name_len + 1), Constraint::Fill(1)],
    // )
    // .block(block)
    // .row_highlight_style(Style::new().bg(Color::Rgb(40, 40, 60)))
    // .highlight_symbol(">");
    //
    // frame.render_stateful_widget(table, area, &mut model.projects_table_state);
}
