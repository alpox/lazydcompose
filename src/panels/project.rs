use crossterm::event::KeyEvent;
use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    widgets::{Block, BorderType, Borders},
};

use crate::{
    bindings::{KeyAction, bindings},
    cli::{Project, docker_project_action},
    effect::Effect,
    event::Message,
    model::{Model, PendingOperation, Prompt, ResourceId, ViewId},
    panels::{
        containers::{self},
        info,
    },
    ui::colors::Colorize,
    util::args,
};

fn docker_compose_action<F, R>(model: &Model, op: PendingOperation, f: F) -> Effect<Message>
where
    F: FnOnce(Project) -> R + Send + 'static,
    R: IntoIterator<Item = String> + Send,
{
    match model.selected_project() {
        Some(project) => {
            let mut effects: Vec<_> = project
                .containers
                .iter()
                .map(|container| {
                    Effect::Dispatch(Message::InitPending(
                        ResourceId::Container(container.id.clone()),
                        op.clone(),
                    ))
                })
                .collect();

            let async_effect = Effect::perform(async move {
                let args: Vec<_> = f(project.clone()).into_iter().collect();
                let result = docker_project_action(project, args).await;
                Some(Message::from(result))
            });

            effects.push(async_effect);

            Effect::Batch(effects)
        }
        _ => Effect::None,
    }
}

pub fn handle_key(model: &mut Model, key: KeyEvent) -> Effect<Message> {
    match bindings().resolve(&key, model) {
        Some(KeyAction::Info) => {
            model.active_view = ViewId::Info;
            model.info_scroll = 0;
            match model.selected_project() {
                Some(p) => Effect::Dispatch(Message::RefreshProjectInfo(p.name)),
                None => Effect::None,
            }
        }
        Some(KeyAction::QuitInfo) => {
            model.active_view = ViewId::Main;
            model.info_scroll = 0;
            Effect::None
        }
        Some(KeyAction::ScrollDown) => {
            model.info_scroll = model.info_scroll.saturating_add(1);
            Effect::None
        }
        Some(KeyAction::ScrollUp) => {
            model.info_scroll = model.info_scroll.saturating_sub(1);
            Effect::None
        }
        Some(KeyAction::MoveUp) => {
            model.select_previous_project();
            Effect::None
        }
        Some(KeyAction::MoveDown) => {
            model.select_next_project();
            Effect::None
        }
        Some(KeyAction::DockerComposeStop) => {
            if let Some(project) = &model.selected_project() {
                model.prompt(Prompt::new(
                    "Confirm",
                    format!(
                        "Are you sure that you want to stop the project '{}'?",
                        project.name
                    ),
                    docker_compose_action(model, PendingOperation::Stopping, |_| {
                        args(["compose", "stop"])
                    }),
                ))
            }

            Effect::None
        }
        Some(KeyAction::DockerComposeStart) => {
            docker_compose_action(model, PendingOperation::Starting, |_| {
                args(["compose", "start"])
            })
        }
        Some(KeyAction::DockerComposeUp) => {
            docker_compose_action(model, PendingOperation::Starting, |_| {
                args(["compose", "up", "-d"])
            })
        }
        Some(KeyAction::DockerComposeDown) => {
            if let Some(project) = &model.selected_project() {
                model.prompt(Prompt::new(
                    "Confirm",
                    format!(
                        "Are you sure that you want to down the project '{}'?",
                        project.name
                    ),
                    docker_compose_action(model, PendingOperation::Starting, |_| {
                        args(["compose", "down"])
                    }),
                ))
            }

            Effect::None
        }
        Some(KeyAction::DockerComposeRestart) => {
            docker_compose_action(model, PendingOperation::Starting, |_| {
                args(["compose", "restart"])
            })
        }
        Some(KeyAction::Select) => {
            model.select_project(model.active_project_index);
            Effect::None
        }
        _ => Effect::None,
    }
}

pub fn view(
    model: &mut Model,
    project_index: usize,
    is_active: bool,
    frame: &mut Frame,
    area: Rect,
) {
    let project = &model.projects[project_index];
    let rows = project.containers.len();

    let block = Block::new()
        .borders(Borders::ALL)
        .border_type(if is_active {
            BorderType::Double
        } else {
            BorderType::Rounded
        })
        .title(project.name.clone())
        .border_style(project.colorize());

    let [projects, info] = Layout::default()
        .direction(ratatui::layout::Direction::Vertical)
        .constraints([Constraint::Length(rows as u16 + 2), Constraint::Fill(1)])
        .areas(area);

    let inner = block.inner(area);
    frame.render_widget(block, projects);

    containers::view(model, project_index, frame, inner);
    info::view(model, frame, info)
}
