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
};

fn docker_compose_action<F, R>(model: &mut Model, op: PendingOperation, f: F) -> Action<Message>
where
    F: FnOnce(Project) -> R,
    R: Into<Vec<String>>,
{
    match model.selected_project() {
        Some(project) => {
            for container in &project.containers {
                model.init_pending_action(ResourceId::Container(container.id.clone()), op.clone());
            }
            Action::Cmd(Box::new(DockerAction {
                project: project.clone(),
                msg_fn: Some(Box::new(Message::ActionResult)),
                args: f(project).into(),
            }))
        }
        _ => Action::None,
    }
}

pub fn handle_key(model: &mut Model, key: KeyEvent) -> Action<Message> {
    match BINDINGS.get_for_context(&key, model.active_context) {
        Some(KeyAction::MoveUp) => {
            model.select_previous_project();
            Action::None
        }
        Some(KeyAction::MoveDown) => {
            model.select_next_project();
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
    let min_size = 4_u16;
    let project_height = |project: &Project| (project.containers.len() as u16 + 2).min(min_size);
    let project_index_height =
        |idx: usize| model.projects.get(idx).map(project_height).unwrap_or(2);
    let mut offsets = model
        .projects
        .iter()
        .scan(0, |sum, project| {
            *sum += project_height(project);
            Some(*sum)
        })
        .collect::<Vec<_>>();

    // First project offset
    offsets.insert(0, 0);

    // Min block size of 4 per project (2 rows of containers)
    let min_total_height = *offsets.last().unwrap_or(&0);
    let needs_scroll = area.height < min_total_height;

    let active_idx = model.active_project_index.unwrap_or(0);
    let active_y = offsets.get(active_idx).unwrap_or(&0);
    let scroll_offset = (active_y + project_index_height(active_idx)).saturating_sub(area.height);

    let first_visible_index = offsets
        .iter()
        .position(|&offset| scroll_offset <= offset)
        .unwrap_or(0);
    let first_visible_offset = offsets.get(first_visible_index).copied().unwrap_or(0);

    let visible_projects: Vec<_> = model.projects.iter().skip(first_visible_index).collect();

    let layout = if needs_scroll {
        visible_projects
            .iter()
            .enumerate()
            .map(|(i, _)| Rect {
                height: project_index_height(first_visible_index + i),
                y: area.y + offsets[i + first_visible_index] - first_visible_offset,
                ..area
            })
            .filter(|rect| (rect.y + rect.height) <= (area.y + area.height))
            .collect::<Vec<_>>()
    } else {
        let constraints = visible_projects
            .iter()
            .map(|project| Constraint::Length(project.containers.len() as u16 + 2));

        Layout::default()
            .direction(Direction::Vertical)
            .constraints(constraints)
            .split(area)
            .to_vec()
    };

    for (i, area) in layout.iter().enumerate() {
        let project = &model.projects[i + first_visible_index];

        let block = Block::new()
            .borders(Borders::ALL)
            .border_type(
                if let Some(idx) = model.active_project_index
                    && (idx - first_visible_index) == i
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

        containers::view(model, i + first_visible_index, frame, inner);
    }
}
