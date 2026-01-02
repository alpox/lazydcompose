use std::{cmp::min, time::Duration};

use crossterm::event::KeyEvent;
use ratatui::{
    Frame,
    layout::{Alignment, Rect},
    style::{Color, Style},
    text::Line,
    widgets::{Block, BorderType, HighlightSpacing, List, ListItem, ListState},
};

use crate::{
    bindings::{Bindings, KeyAction},
    cli::Container,
    cmd::DockerContainerListCommand,
    model::{Action, ChildAction, PanelId},
    subs::Subscription,
};

impl From<&Container> for ListItem<'_> {
    fn from(value: &Container) -> Self {
        ListItem::new(Line::styled(
            format!("Container: {}", value.names),
            Color::Cyan,
        ))
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Message {
    RefreshContainers,
    Containers(Result<Vec<Container>, String>),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ContainersPanel {
    pub id: PanelId,
    pub containers: Vec<Container>,
    pub list_state: ListState,
}

impl Default for ContainersPanel {
    fn default() -> Self {
        Self {
            id: PanelId::Containers,
            list_state: ListState::default(),
            containers: vec![],
        }
    }
}

pub fn update(model: &mut ContainersPanel, msg: Message) -> ChildAction<Message, ()> {
    match msg {
        Message::RefreshContainers => {
            let cmd = Action::Cmd(Box::new(DockerContainerListCommand {
                msg_fn: |r| match r {
                    Ok(containers) => Message::Containers(Ok(containers)),
                    Err(err) => Message::Containers(Err(err.to_string())),
                },
                args: vec!["-a".to_string()],
            }));
            ChildAction::new(cmd)
        }
        Message::Containers(Ok(containers)) => {
            model
                .list_state
                .select(match (model.list_state.selected(), containers.len()) {
                    (Some(idx), pl) => Some(min(idx, pl - 1)),
                    (_, 0) => None,
                    _ => Some(0),
                });
            model.containers = containers;
            ChildAction::none()
        }
        Message::Containers(Err(_)) => ChildAction::none(),
    }
}

pub fn handle_key(model: &mut ContainersPanel, key: KeyEvent) -> ChildAction<Message, ()> {
    match Bindings.get(&key) {
        Some(KeyAction::MoveUp) => {
            model.list_state.select_previous();
            ChildAction::none()
        }
        Some(KeyAction::MoveDown) => {
            model.list_state.select_next();
            ChildAction::none()
        }
        _ => ChildAction::none(),
    }
}

pub fn subscriptions(_model: &ContainersPanel) -> Subscription<Message> {
    Subscription::Interval(Duration::from_secs(2), Message::RefreshContainers)
}

pub fn view(model: &mut ContainersPanel, frame: &mut Frame, area: Rect, is_active: bool) {
    let block = Block::bordered()
        .title("[2] containers")
        .title_alignment(Alignment::Center)
        .border_type(BorderType::Rounded)
        .border_style(if is_active {
            Color::Green
        } else {
            Color::DarkGray
        });

    let items = model.containers.iter().map(ListItem::from);

    let list = List::new(items)
        .block(block)
        .highlight_style(Style::new().bg(Color::DarkGray))
        .highlight_symbol(">")
        .highlight_spacing(HighlightSpacing::Always);

    frame.render_stateful_widget(list, area, &mut model.list_state);
}
