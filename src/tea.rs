use std::time::Duration;

use color_eyre::eyre::Context;
use crossterm::event::KeyEvent;
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    widgets::{Paragraph, Widget},
};

use crate::{
    bindings::{KeyAction, bindings},
    cli::{cli_action, docker_get_projects},
    effect::Effect,
    event::Message,
    inspect::ContainerInspect,
    model::{ContextId, Model, Note, OverlayContextId, RunningState, ViewId},
    panels::{
        containers::{self},
        project,
        projects::{self},
    },
    subs::Subscription,
    ui::{
        bindings::{self, Bindings},
        notes::Notes,
        prompt::{self},
    },
    util::ResultExt,
};

pub fn quit(model: &mut Model) -> Effect<Message> {
    model.running_state = RunningState::Done;
    Effect::None
}

fn note_styled(model: &mut Model, text: impl Into<String>, style: Style) -> Effect<Message> {
    model.notes.push(Note::new(text.into()).style(style));
    Effect::None
}

fn note_err(model: &mut Model, text: impl Into<String>) -> Effect<Message> {
    note_styled(model, text, Style::new().fg(Color::Red))
}

// fn note(model: &mut Model, text: impl Into<String>) -> Effect<Message> {
//     note_styled(model, text, Default::default())
// }

pub fn update(model: &mut Model, msg: Message) -> Effect<Message> {
    match msg {
        Message::Quit => quit(model),
        Message::Tick => Effect::None,
        Message::KeyPress(key) => handle_key(model, key),
        Message::InitPending(resource_id, op) => {
            model.init_pending_action(resource_id, op);
            Effect::None
        }
        Message::RefreshProjects => Effect::perform(async move {
            let result = docker_get_projects().await;
            Some(Message::Projects(result.stringify_err()))
        }),
        Message::RefreshProjectInfo(project_name) => {
            let ids: Vec<String> = model
                .projects
                .iter()
                .find(|p| p.name == project_name)
                .map(|p| p.containers.iter().map(|c| c.id.clone()).collect())
                .unwrap_or_default();

            if ids.is_empty() {
                return Effect::None;
            }

            Effect::perform(async move {
                let mut args = vec!["inspect".to_string()];
                args.extend(ids);
                let arg_refs: Vec<&str> = args.iter().map(String::as_str).collect();

                let result = cli_action("docker", arg_refs)
                    .await
                    .and_then(|out| {
                        serde_json::from_str::<Vec<ContainerInspect>>(&out)
                            .wrap_err("parse docker inspect")
                    })
                    .map(|inspects| (project_name, inspects))
                    .stringify_err();

                Some(Message::ProjectInfo(result))
            })
        }
        Message::ProjectInfo(Ok((_, inspects))) => {
            for inspect in inspects {
                let key = inspect.id[..12].to_string();
                model.inspects.insert(key, inspect);
            }

            Effect::None
        }
        Message::ProjectInfo(Err(err)) => note_err(model, err),
        Message::Projects(Ok(projects)) => {
            model.projects = projects;
            if !model.projects.is_empty() && model.active_project_index.is_none() {
                model.active_project_index = Some(0);
            }
            model.update_pending_actions();
            Effect::None
        }
        Message::Projects(Err(err)) => note_err(model, err),
        Message::ActionResult(Ok(_)) => {
            model.update_pending_actions();
            Effect::None
        }
        Message::ActionResult(Err(err)) => {
            model.update_pending_actions();
            note_err(model, err)
        }
        Message::ClearNotes => {
            model.notes = model
                .notes
                .iter()
                .filter(|note| !note.finished())
                .cloned()
                .collect();
            Effect::None
        }
        Message::Resize => Effect::None,
    }
}

fn handle_key(model: &mut Model, key: KeyEvent) -> Effect<Message> {
    if model.active_overlay_context.is_some() {
        return match model.active_overlay_context {
            Some(OverlayContextId::BindingsPopup) => bindings::handle_key(model, key),
            Some(OverlayContextId::Prompt) => prompt::handle_key(model, key),
            None => Effect::None,
        };
    }

    match bindings().resolve(&key, model) {
        Some(KeyAction::Quit) => quit(model),
        Some(KeyAction::ShowBindings) => {
            model.active_overlay_context = Some(OverlayContextId::BindingsPopup);
            model.selected_action_index = Some(0);
            Effect::None
        }
        _ => match model.active_context {
            ContextId::Projects => project::handle_key(model, key),
            ContextId::Containers => containers::handle_key(model, key),
        },
    }
}

pub fn subscriptions(model: &Model) -> Subscription<Message> {
    let mut subscriptions = vec![Subscription::Interval(
        Duration::from_secs(1),
        Message::RefreshProjects,
    )];

    if model.active_view == ViewId::Info
        && let Some(project) = model.selected_project()
    {
        subscriptions.push(Subscription::Interval(
            Duration::from_secs(1),
            Message::RefreshProjectInfo(project.name),
        ))
    }

    if !model.notes.is_empty() {
        subscriptions.push(Subscription::Interval(
            Duration::from_secs(1),
            Message::ClearNotes,
        ))
    }

    Subscription::Batch(subscriptions)
}

struct AppLayout {
    pub main: Rect,
    pub hints: Rect,
}

fn layout(area: Rect) -> AppLayout {
    let [main, hints] = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Fill(1), Constraint::Length(1)])
        .areas(area);

    AppLayout { main, hints }
}

pub fn view(model: &mut Model, frame: &mut Frame) {
    let layout = layout(frame.area());

    match model.active_view {
        ViewId::Info if model.active_project_index.is_some() => project::view(
            model,
            model.active_project_index.unwrap(),
            true,
            frame,
            layout.main,
        ),
        _ => projects::view(model, frame, layout.main),
    };

    Notes::new(model.notes.clone()).render(frame.area(), frame.buffer_mut());

    match model.active_overlay_context {
        Some(OverlayContextId::BindingsPopup) => {
            Bindings::new(model).render(frame.area(), frame.buffer_mut());
        }
        Some(OverlayContextId::Prompt) => {
            if let Some(prompt) = &model.prompt {
                prompt::Prompt::new(&prompt.title, &prompt.text)
                    .render(frame.area(), frame.buffer_mut());
            }
        }
        None => {}
    }

    Paragraph::new("?: Keybindings").render(layout.hints, frame.buffer_mut());
}
