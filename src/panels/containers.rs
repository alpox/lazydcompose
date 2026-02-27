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
    cli::{Container, Project, docker_action_tty, docker_project_action},
    effect::Effect,
    event::Message,
    model::{ContextId, Model, PendingOperation, Prompt, ResourceId},
    ui::colors::Colorize,
    util::args,
};

impl From<&Container> for ListItem<'_> {
    fn from(value: &Container) -> Self {
        ListItem::new(Line::styled(
            format!("Container: {}", value.names),
            Color::Cyan,
        ))
    }
}

fn docker_action<F, R>(model: &Model, op: PendingOperation, f: F) -> Effect<Message>
where
    F: FnOnce(Project, Container) -> R + Send + 'static,
    R: IntoIterator<Item = String> + Send,
{
    match (model.selected_project(), model.selected_container()) {
        (Some(project), Some(container)) => {
            let resource_id = ResourceId::Container(container.id.clone());
            Effect::Batch(vec![
                Effect::Dispatch(Message::InitPending(resource_id, op)),
                Effect::perform(async move {
                    let args: Vec<_> = f(project.clone(), container.clone()).into_iter().collect();
                    let result = docker_project_action(project, args).await;
                    Some(Message::from(result))
                }),
            ])
        }
        _ => Effect::None,
    }
}

pub fn handle_key(model: &mut Model, key: KeyEvent) -> Effect<Message> {
    match BINDINGS.resolve(&key, model) {
        Some(KeyAction::MoveUp) => {
            model.select_previous_container();
            Effect::None
        }
        Some(KeyAction::MoveDown) => {
            model.select_next_container();
            Effect::None
        }
        Some(KeyAction::DockerContainerStart) => {
            docker_action(model, PendingOperation::Starting, |_, container| {
                args(["container", "start", container.names.as_str()])
            })
        }
        Some(KeyAction::DockerContainerStop) => {
            if let Some(container) = &model.selected_container() {
                model.prompt(Prompt::new(
                    "Confirm",
                    format!(
                        "Are you sure that you want to stop the container '{}'?",
                        container.names
                    ),
                    docker_action(model, PendingOperation::Stopping, |_, container| {
                        args(["container", "stop", container.names.as_str()])
                    }),
                ))
            }

            Effect::None
        }
        Some(KeyAction::DockerContainerRestart) => {
            docker_action(model, PendingOperation::Restarting, |_, container| {
                args(["container", "restart", container.names.as_str()])
            })
        }
        Some(KeyAction::DockerFollowLogs) => {
            match (model.selected_project(), model.selected_container()) {
                (Some(project), Some(container)) => Effect::perform_blocking(async move {
                    let _ = docker_action_tty(
                        project,
                        ["logs", "--follow", "--since=60m", container.id.as_str()],
                    )
                    .await;
                    None
                }),
                _ => Effect::None,
            }
        }
        Some(KeyAction::DockerConsole) => {
            match (model.selected_project(), model.selected_container()) {
                (Some(project), Some(container)) => Effect::perform_blocking(async move {
                    let _ = docker_action_tty(
                        project,
                        ["exec", "-it", container.id.as_str(), "/bin/sh"],
                    )
                    .await;
                    None
                }),
                _ => Effect::None,
            }
        }
        Some(KeyAction::Deselect) => {
            model.active_context = ContextId::Projects;
            model.active_container_index = None;
            Effect::None
        }
        _ => Effect::None,
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
            let prefix = if model.has_pending_action(&ResourceId::Container(container.id.clone())) {
                "⟳ "
            } else {
                ""
            };
            Row::new(vec![
                Cell::from(prefix.to_string() + container.title()),
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
