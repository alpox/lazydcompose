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
    cmd::DockerGetProjectsCommand,
    event::Message,
    model::{Action, ContextId, Model, Note, OverlayContextId, RunningState},
    panels::{
        containers::{self},
        projects::{self},
    },
    subs::Subscription,
    ui::{
        bindings::{self, Bindings},
        notes::Notes,
    },
};

const PANEL_ORDER: [ContextId; 2] = [ContextId::Projects, ContextId::Containers];

pub fn quit(model: &mut Model) -> Action<Message> {
    model.running_state = RunningState::Done;
    Action::None
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

fn note_styled(model: &mut Model, text: impl Into<String>, style: Style) -> Action<Message> {
    model.notes.push(Note::new(text.into()).style(style));
    Action::None
}

fn note_err(model: &mut Model, text: impl Into<String>) -> Action<Message> {
    note_styled(model, text, Style::new().fg(Color::Red))
}

fn note(model: &mut Model, text: impl Into<String>) -> Action<Message> {
    note_styled(model, text, Default::default())
}

pub fn update(model: &mut Model, msg: Message) -> Action<Message> {
    match msg {
        Message::Quit => quit(model),
        Message::Tick => Action::None,
        Message::KeyPress(key) => handle_key(model, key),
        Message::RefreshProjects => {
            Action::Cmd(Box::new(DockerGetProjectsCommand(Message::Projects)))
        }
        Message::Projects(Ok(projects)) => {
            model.projects = projects;
            if !model.projects.is_empty() && model.active_project_index.is_none() {
                model.active_project_index = Some(0);
            }
            Action::None
        }
        Message::Projects(Err(err)) => note_err(model, err),
        Message::ActionResult(Ok(_)) => Action::None,
        Message::ActionResult(Err(err)) => note_err(model, err),
        Message::ClearNotes => {
            model.notes = model
                .notes
                .iter()
                .filter(|note| !note.finished())
                .cloned()
                .collect();
            Action::None
        }
    }
}

fn handle_key(model: &mut Model, key: KeyEvent) -> Action<Message> {
    if model.active_overlay_context.is_some() {
        return match model.active_overlay_context {
            Some(OverlayContextId::BindingsPopup) => bindings::handle_key(model, key),
            None => Action::None,
        };
    }

    match BINDINGS.get_for_context(&key, model.active_context) {
        Some(KeyAction::Quit) => quit(model),
        Some(KeyAction::NextPanel) => {
            move_panel_selection(model, 1);
            Action::None
        }
        Some(KeyAction::PrevPanel) => {
            move_panel_selection(model, -1);
            Action::None
        }
        Some(KeyAction::ShowBindings) => {
            model.active_overlay_context = Some(OverlayContextId::BindingsPopup);
            Action::None
        }
        _ => match model.active_context {
            ContextId::Projects => projects::handle_key(model, key),
            ContextId::Containers => containers::handle_key(model, key),
            _ => Action::None,
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
    pub main: Rect,
    pub hints: Rect,
}

fn layout(area: Rect) -> AppLayout {
    let full = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Fill(1), Constraint::Length(1)])
        .split(area);

    let horizontal = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(30), Constraint::Percentage(70)])
        .split(full[0]);

    AppLayout {
        projects: horizontal[0],
        main: horizontal[1],
        hints: full[1],
    }
}

pub fn view(model: &mut Model, frame: &mut Frame) {
    let layout = layout(frame.area());

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
