use crossterm::event::KeyEvent;
use ratatui::{
    Frame,
    layout::{Alignment, Rect},
    style::{Color, Style},
    text::Line,
    widgets::{Block, BorderType, HighlightSpacing, List, ListItem},
};

use crate::{
    bindings::{BINDINGS, KeyAction},
    cli::Container,
    cmd::DockerContainerListCommand,
    event::Message,
    model::{Action, Model, PanelId},
    ui::list::ListStateExt,
};

impl From<&Container> for ListItem<'_> {
    fn from(value: &Container) -> Self {
        ListItem::new(Line::styled(
            format!("Container: {}", value.names),
            Color::Cyan,
        ))
    }
}

pub fn refresh_containers(model: &mut Model) -> Action<Message> {
    let mut args: Vec<String> = vec!["-a".to_string()];

    if let Some(project) = model.selected_project() {
        let label_filter = format!("label=com.docker.compose.project={}", project.name);
        args.push("--filter".to_string());
        args.push(label_filter);
    }

    Action::Cmd(Box::new(DockerContainerListCommand {
        msg_fn: Message::Containers,
        args,
    }))
}

pub fn handle_key(model: &mut Model, key: KeyEvent) -> Action<Message> {
    match BINDINGS.get(&key) {
        Some(KeyAction::MoveUp) => {
            model.containers_list_state.select_previous();
            Action::None
        }
        Some(KeyAction::MoveDown) => {
            model.containers_list_state.select_next();
            model.containers_list_state.fit(model.containers.len());
            Action::None
        }
        _ => Action::None,
    }
}

pub fn view(model: &mut Model, frame: &mut Frame, area: Rect) {
    let block = Block::bordered()
        .title("[2] containers")
        .title_alignment(Alignment::Center)
        .border_type(BorderType::Rounded)
        .border_style(if model.active_panel == PanelId::Containers {
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

    frame.render_stateful_widget(list, area, &mut model.containers_list_state);
}
