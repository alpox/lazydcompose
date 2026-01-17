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
        prompt,
    },
    util::ResultExt,
};

const PANEL_ORDER: [ContextId; 2] = [ContextId::Projects, ContextId::Containers];

pub fn quit(model: &mut Model) -> Effect<Message> {
    model.running_state = RunningState::Done;
    Effect::None
}

pub fn move_panel_selection(model: &mut Model, offset: isize) {
    let current_index = PANEL_ORDER
        .iter()
        .position(|&id| id == model.active_context);
    match current_index {
        Some(idx) => {
            let new_idx = idx
                .checked_add_signed(offset)
                .unwrap_or(0)
                .min(PANEL_ORDER.len().saturating_sub(1));
            model.active_context = PANEL_ORDER[new_idx]
        }
        None => model.active_context = PANEL_ORDER[0],
    }
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
        Message::Prompt(prompt) => {
            model.prompt = Some(*prompt);
            Effect::None
        }
    }
}

fn handle_key(model: &mut Model, key: KeyEvent) -> Effect<Message> {
    if model.active_overlay_context.is_some() {
        return match model.active_overlay_context {
            Some(OverlayContextId::BindingsPopup) => bindings::handle_key(model, key),
            None => Effect::None,
        };
    }

    match BINDINGS.get_for_context(&key, model.active_context) {
        Some(KeyAction::Quit) => quit(model),
        Some(KeyAction::ShowBindings) => {
            model.active_overlay_context = Some(OverlayContextId::BindingsPopup);
            Effect::None
        }
        _ => match model.active_context {
            ContextId::Projects => projects::handle_key(model, key),
            ContextId::Containers => containers::handle_key(model, key),
            _ => Effect::None,
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

    if let Some(prompt) = model.prompt.clone() {
        prompt::Prompt::new(prompt).render(frame.area(), frame.buffer_mut());
    }

    projects::view(model, frame, layout.projects);

    Notes::new(model.notes.clone()).render(frame.area(), frame.buffer_mut());

    match model.active_overlay_context {
        Some(OverlayContextId::BindingsPopup) => {
            Bindings::new(model.active_context).render(frame.area(), frame.buffer_mut());
        }
        None => {}
    }

    Paragraph::new("?: Keybindings").render(layout.hints, frame.buffer_mut());
}
