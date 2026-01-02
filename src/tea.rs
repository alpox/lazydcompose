use crossterm::event::KeyEvent;
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
};

use crate::{
    bindings::{Bindings, KeyAction},
    event::Message,
    model::{Action, Model, PanelId, RunningState},
    panels::{
        containers::{self},
        projects::{self},
    },
    subs::Subscription,
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

pub fn update(model: &mut Model, msg: Message) -> Action<Message> {
    match msg {
        Message::Quit => quit(model),
        Message::Tick => Action::None,
        Message::KeyPress(key) => handle_key(model, key),
        Message::ProjectsPanel(msg) => projects::update(&mut model.projects_panel, msg)
            .map_msg(Message::ProjectsPanel)
            .handle_out(|m| match m {
                projects::OutMessage::ProjectChanged(_) => containers::update(
                    &mut model.containers_panel,
                    containers::Message::RefreshContainers,
                )
                .map_msg(Message::ContainersPanel)
                .into_inner(),
            }),
        Message::ContainersPanel(msg) => containers::update(&mut model.containers_panel, msg)
            .map_msg(Message::ContainersPanel)
            .into_inner(),
    }
}

fn handle_key(model: &mut Model, key: KeyEvent) -> Action<Message> {
    match Bindings.get(&key) {
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
            model.active_panel = PANEL_ORDER.get(num - 1).cloned().unwrap_or(model.active_panel);
            Action::None
        }
        _ => match model.active_panel {
            PanelId::Projects => projects::handle_key(&mut model.projects_panel, key)
                .map_msg(Message::ProjectsPanel)
                .into_inner(),
            PanelId::Containers => containers::handle_key(&mut model.containers_panel, key)
                .map_msg(Message::ContainersPanel)
                .into_inner(),
            _ => Action::None,
        },
    }
}

pub fn subscriptions(model: &Model) -> Subscription<Message> {
    Subscription::Batch(vec![
        projects::subscriptions(&model.projects_panel).map(Message::ProjectsPanel),
        containers::subscriptions(&model.containers_panel).map(Message::ContainersPanel),
    ])
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

    projects::view(
        &mut model.projects_panel,
        frame,
        layout.projects,
        model.active_panel == PanelId::Projects,
    );
    containers::view(
        &mut model.containers_panel,
        frame,
        layout.containers,
        model.active_panel == PanelId::Containers,
    );
}
