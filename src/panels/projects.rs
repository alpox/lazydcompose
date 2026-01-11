use crossterm::event::KeyEvent;
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::Color,
    widgets::{Block, BorderType, Borders},
};

use crate::{
    bindings::{BINDINGS, KeyAction},
    cli::Project,
    cmd::DockerAction,
    event::Message,
    model::{Action, Model, PendingOperation, ResourceId},
    panels::containers::{self},
    util::wrap_around_optional,
};

fn docker_compose_action<F, R>(model: &mut Model, op: PendingOperation, f: F) -> Action<Message>
where
    F: FnOnce(Project) -> R,
    R: Into<Vec<String>>,
{
    match model.selected_project() {
        Some(project) => {
            let resource_id = ResourceId::Project(project.name.clone());
            model.init_pending_action(resource_id.clone(), op);
            Action::Cmd(Box::new(DockerAction {
                project: project.clone(),
                msg_fn: Some(Message::action_result_constructor(resource_id)),
                args: f(project).into(),
            }))
        }
        _ => Action::None,
    }
}

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
            docker_compose_action(model, PendingOperation::Stopping, |_| {
                vec!["compose".to_string(), "stop".to_string()]
            })
        }
        Some(KeyAction::DockerComposeStart) => {
            docker_compose_action(model, PendingOperation::Starting, |_| {
                vec!["compose".to_string(), "start".to_string()]
            })
        }
        Some(KeyAction::DockerComposeUp) => {
            docker_compose_action(model, PendingOperation::Starting, |_| {
                vec!["compose".to_string(), "up".to_string(), "-d".to_string()]
            })
        }
        Some(KeyAction::DockerComposeDown) => {
            docker_compose_action(model, PendingOperation::Starting, |_| {
                vec!["compose".to_string(), "down".to_string()]
            })
        }
        Some(KeyAction::DockerComposeRestart) => {
            docker_compose_action(model, PendingOperation::Starting, |_| {
                vec!["compose".to_string(), "restart".to_string()]
            })
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
