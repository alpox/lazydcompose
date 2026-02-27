use std::time::Duration;

use crossterm::event::KeyEvent;
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    widgets::{Paragraph, Widget},
};

use crate::{
    bindings::{BINDINGS, KeyAction},
    cli::docker_get_projects,
    effect::Effect,
    event::Message,
    model::{ContextId, Model, Note, OverlayContextId, RunningState},
    panels::{
        containers::{self},
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

    match BINDINGS.resolve(&key, model) {
        Some(KeyAction::Quit) => quit(model),
        Some(KeyAction::ShowBindings) => {
            model.active_overlay_context = Some(OverlayContextId::BindingsPopup);
            Effect::None
        }
        _ => match model.active_context {
            ContextId::Projects => projects::handle_key(model, key),
            ContextId::Containers => containers::handle_key(model, key),
        },
    }
}

pub fn subscriptions(model: &Model) -> Subscription<Message> {
    let mut subscriptions = vec![Subscription::Interval(
        Duration::from_secs(1),
        Message::RefreshProjects,
    )];

    if !model.notes.is_empty() {
        subscriptions.push(Subscription::Interval(
            Duration::from_secs(1),
            Message::ClearNotes,
        ))
    }

    Subscription::Batch(subscriptions)
}

struct AppLayout {
    pub projects: Rect,
    pub hints: Rect,
}

fn layout(area: Rect) -> AppLayout {
    let [projects, hints] = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Fill(1), Constraint::Length(1)])
        .areas(area);
    //
    // let horizontal = Layout::default()
    //     .direction(Direction::Horizontal)
    //     .constraints([Constraint::Percentage(30), Constraint::Percentage(70)])
    //     .split(full[0]);

    AppLayout { projects, hints }
}

pub fn view(model: &mut Model, frame: &mut Frame) {
    let layout = layout(frame.area());

    projects::view(model, frame, layout.projects);

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
