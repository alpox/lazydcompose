use std::time::Duration;

use crossterm::event::KeyEvent;
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    widgets::Widget,
};

use crate::{
    bindings::{BINDINGS, KeyAction},
    cmd::DockerComposeLsCommand,
    event::Message,
    model::{Action, Model, Note, PanelId, RunningState},
    panels::{
        containers::{self, refresh_containers},
        projects::{self},
    },
    subs::Subscription,
    ui::{notes::Notes, table::TableStateExt},
};

const PANEL_ORDER: [PanelId; 2] = [PanelId::Projects, PanelId::Containers];

pub fn quit(model: &mut Model) -> Action<Message> {
    model.running_state = RunningState::Done;
    Action::None
}

pub fn move_panel_selection(model: &mut Model, offset: isize) {
    let current_index = PANEL_ORDER.iter().position(|&id| id == model.active_panel);
    match current_index {
        Some(idx) => {
            let new_idx = idx
                .checked_add_signed(offset)
                .unwrap_or(0)
                .min(PANEL_ORDER.len().saturating_sub(1));
            model.active_panel = PANEL_ORDER[new_idx]
        }
        None => model.active_panel = PANEL_ORDER[0],
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
            Action::Cmd(Box::new(DockerComposeLsCommand(Message::Projects)))
        }
        Message::Projects(Ok(projects)) => {
            model.projects_table_state.fit(projects.len());
            model.projects = projects;
            refresh_containers(model)
        }
        Message::Projects(Err(err)) => note_err(model, err),
        Message::RefreshContainers => refresh_containers(model),
        Message::Containers(Ok(containers)) => {
            model.containers_table_state.fit(containers.len());
            model.containers = containers;
            Action::None
        }
        Message::Containers(Err(err)) => note_err(model, err),
        Message::DockerComposeStart(Ok(_)) => refresh_containers(model),
        Message::DockerComposeStart(Err(err)) => note_err(model, err),
        Message::DockerComposeStop(Ok(_)) => refresh_containers(model),
        Message::DockerComposeStop(Err(err)) => note_err(model, err),
        Message::DockerComposeUp(Ok(_)) => refresh_containers(model),
        Message::DockerComposeUp(Err(err)) => note_err(model, err),
        Message::DockerComposeDown(Ok(_)) => refresh_containers(model),
        Message::DockerComposeDown(Err(err)) => note_err(model, err),
        Message::DockerComposeRestart(Ok(_)) => refresh_containers(model),
        Message::DockerComposeRestart(Err(err)) => note_err(model, err),
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
    match BINDINGS.get(&key) {
        Some(KeyAction::Quit) => quit(model),
        Some(KeyAction::NextPanel) => {
            move_panel_selection(model, 1);
            Action::None
        }
        Some(KeyAction::PrevPanel) => {
            move_panel_selection(model, -1);
            Action::None
        }
        Some(KeyAction::SelectPanel(num)) => {
            model.active_panel = PANEL_ORDER
                .get(num - 1)
                .cloned()
                .unwrap_or(model.active_panel);
            Action::None
        }
        _ => match model.active_panel {
            PanelId::Projects => projects::handle_key(model, key),
            PanelId::Containers => containers::handle_key(model, key),
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
    pub containers: Rect,
    pub main: Rect,
}

fn layout(area: Rect) -> AppLayout {
    let horizontal = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(30), Constraint::Percentage(70)])
        .split(area);

    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(33),
            Constraint::Percentage(34),
            Constraint::Percentage(33),
        ])
        .split(horizontal[0]);

    AppLayout {
        projects: vertical[0],
        containers: vertical[1],
        main: horizontal[1],
    }
}

pub fn view(model: &mut Model, frame: &mut Frame) {
    let layout = layout(frame.area());

    projects::view(model, frame, layout.projects);
    containers::view(model, frame, layout.containers);

    Notes::new(model.notes.clone()).render(frame.area(), frame.buffer_mut());
}
