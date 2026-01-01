use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    Frame,
    layout::Alignment,
    style::{Color, Style},
    text::Line,
    widgets::{Block, BorderType, HighlightSpacing, List, ListItem},
};

use crate::{
    cli::Project,
    event::Message,
    model::{Model, PanelId, RunningState, UpdateResult},
    panels::projects::{self},
    subs::Subscription,
};

pub fn update(model: &mut Model, msg: Message) -> UpdateResult<Message> {
    match msg {
        Message::Increment => {
            model.counter = model.counter.saturating_add(1);
            UpdateResult::None
        }
        Message::Decrement => {
            model.counter = model.counter.saturating_sub(1);
            UpdateResult::None
        }
        Message::Quit => {
            model.running_state = RunningState::Done;
            UpdateResult::None
        }
        Message::Tick => UpdateResult::None,
        Message::KeyPress(key) => map_key_to_message(model, key),

        Message::ProjectsPanel(msg) => match model.active_panel {
            PanelId::Projects => {
                projects::update(&mut model.projects_panel, msg).map(Message::ProjectsPanel)
            }
            _ => UpdateResult::None,
        },
    }
}

fn map_key_to_message(model: &Model, key: KeyEvent) -> UpdateResult<Message> {
    match key.code {
        KeyCode::Esc | KeyCode::Char('q') => UpdateResult::Msg(Message::Quit),
        KeyCode::Char('c' | 'C') if key.modifiers == KeyModifiers::CONTROL => {
            UpdateResult::Msg(Message::Quit)
        }
        KeyCode::Right => UpdateResult::Msg(Message::Increment),
        KeyCode::Left => UpdateResult::Msg(Message::Decrement),

        _ => match model.active_panel {
            PanelId::Projects => {
                projects::map_key_to_message(&model.projects_panel, key).map(Message::ProjectsPanel)
            }
            _ => UpdateResult::None,
        },
    }
}

pub fn subscriptions(model: &Model) -> Subscription<Message> {
    Subscription::Batch(vec![
        projects::subscriptions(&model.projects_panel).map(Message::ProjectsPanel),
    ])
}

pub fn view(model: &mut Model, frame: &mut Frame) {
    let block = Block::bordered()
        .title("lazydcompose")
        .title_alignment(Alignment::Center)
        .border_type(BorderType::Rounded);

    let items = model.projects_panel.projects.iter().map(ListItem::from);

    let list = List::new(items)
        .block(block)
        .highlight_style(Style::new().bg(Color::DarkGray))
        .highlight_symbol(">")
        .highlight_spacing(HighlightSpacing::Always);

    frame.render_stateful_widget(list, frame.area(), &mut model.projects_panel.list_state);
}

impl From<&Project> for ListItem<'_> {
    fn from(value: &Project) -> Self {
        ListItem::new(Line::styled(
            format!("Project: {}", value.name),
            Color::Cyan,
        ))
    }
}
