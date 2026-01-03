use std::time::Duration;

use crossterm::event::KeyEvent;
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
};

use crate::{
    bindings::{BINDINGS, KeyAction},
    cmd::DockerComposeLsCommand,
    event::Message,
    model::{Action, Model, PanelId, RunningState},
    panels::{
        containers::{self, refresh_containers},
        projects::{self},
    },
    subs::Subscription,
    ui::table::TableStateExt,
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
        Message::RefreshProjects => {
            Action::Cmd(Box::new(DockerComposeLsCommand(Message::Projects)))
        }
        Message::Projects(Ok(projects)) => {
            model.projects_table_state.fit(projects.len());
            model.projects = projects;
            refresh_containers(model)
        }
        Message::Projects(Err(_)) => Action::None,
        Message::RefreshContainers => refresh_containers(model),
        Message::Containers(Ok(containers)) => {
            model.containers_table_state.fit(containers.len());
            model.containers = containers;
            Action::None
        }
        Message::Containers(Err(_)) => Action::None,
        Message::DockerComposeStart(_) => {
            refresh_containers(model)
        },
        Message::DockerComposeStop(_) => {
            refresh_containers(model)
        },
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

pub fn subscriptions(_model: &Model) -> Subscription<Message> {
    Subscription::Batch(vec![Subscription::Interval(
        Duration::from_secs(1),
        Message::RefreshProjects,
    )])
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
}
