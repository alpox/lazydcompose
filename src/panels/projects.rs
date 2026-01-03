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
    cli::Project,
    event::Message,
    model::{Action, Model, PanelId},
    panels::containers::refresh_containers,
    ui::list::ListStateExt,
};

impl From<&Project> for ListItem<'_> {
    fn from(value: &Project) -> Self {
        ListItem::new(Line::styled(
            format!("Project: {}", value.name),
            Color::Cyan,
        ))
    }
}

pub fn handle_key(model: &mut Model, key: KeyEvent) -> Action<Message> {
    match BINDINGS.get(&key) {
        Some(KeyAction::MoveUp) => {
            model.projects_list_state.select_previous();
            refresh_containers(model)
        }
        Some(KeyAction::MoveDown) => {
            model.projects_list_state.select_next();
            model.projects_list_state.fit(model.projects.len());
            refresh_containers(model)
        }
        _ => Action::None,
    }
}

pub fn view(model: &mut Model, frame: &mut Frame, area: Rect) {
    let block = Block::bordered()
        .title("[1] projects")
        .title_alignment(Alignment::Center)
        .border_type(BorderType::Rounded)
        .border_style(if model.active_panel == PanelId::Projects {
            Color::Green
        } else {
            Color::DarkGray
        });

    let items = model.projects.iter().map(ListItem::from);

    let list = List::new(items)
        .block(block)
        .highlight_style(Style::new().bg(Color::DarkGray))
        .highlight_symbol(">")
        .highlight_spacing(HighlightSpacing::Always);

    frame.render_stateful_widget(list, area, &mut model.projects_list_state);
}
