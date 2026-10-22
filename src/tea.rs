use std::time::{Duration, Instant};

use crossterm::event::KeyEvent;
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    widgets::{Paragraph, Widget},
};

use crate::{
    bindings::bindings, effect::Effect, event::Message, events::project::{RefreshProjectInfo, RefreshProjects}, model::{ContextId, Model, Note, Overlay, ViewId}, panels::{
        containers::{self},
        project,
        projects::{self},
    }, subs::Subscription, system::{self, Ctx}, ui::{
        bindings::Bindings,
        notes::Notes,
        prompt::{self},
    },
};

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
        Message::Tick => Effect::None,
        Message::KeyPress(key) => handle_key(model, key),
        Message::InitPending(resource_id, op) => {
            model.init_pending_action(resource_id, op);
            Effect::None
        }
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
        Message::Event(event) => handle_event(model, event),
    }
}

fn handle_event(model: &mut Model, event: system::Message) -> Effect<Message> {
    event
        .handle(&mut Ctx {
            db: model,
            now: Instant::now(),
        })
        .map(Message::Event)
}

fn handle_key(model: &mut Model, key: KeyEvent) -> Effect<Message> {
    if let Some(resolved_event) = bindings().resolve_event(&key, model) {
        return handle_event(model, resolved_event);
    }

    match model.active_context {
        ContextId::Projects => project::handle_key(model, key),
        ContextId::Containers => containers::handle_key(model, key),
    }
}

pub fn subscriptions(model: &Model) -> Subscription<Message> {
    let mut subscriptions = vec![Subscription::Interval(
        Duration::from_secs(1),
        Message::Event(RefreshProjects.into()),
    )];

    if model.active_view == ViewId::Info
        && let Some(project) = model.selected_project()
    {
        subscriptions.push(Subscription::Interval(
            Duration::from_secs(1),
            Message::Event(RefreshProjectInfo {
                project_name: project.name
            }.into())
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

    match &model.overlay {
        Some(Overlay::Bindings { selected: _ }) => {
            Bindings::new(model).render(frame.area(), frame.buffer_mut());
        }
        Some(Overlay::Prompt(prompt)) => {
            prompt::Prompt::new(&prompt.title, &prompt.text)
                .render(frame.area(), frame.buffer_mut());
        }
        None => {}
    }

    Paragraph::new("?: Keybindings").render(layout.hints, frame.buffer_mut());
}
