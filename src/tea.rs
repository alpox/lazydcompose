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
    cmd::{Cmd, DockerComposeLsCommand},
    event::Message,
    model::{Model, RunningState},
    subs::Subscription,
};
use std::{cmp::min, time::Duration};

type BoxedCmd = Box<dyn Cmd<Model, Message>>;

pub fn update(model: &mut Model, msg: Message) -> (Option<Message>, Option<BoxedCmd>) {
    match msg {
        Message::Increment => model.counter = model.counter.saturating_add(1),
        Message::Decrement => model.counter = model.counter.saturating_sub(1),
        Message::Quit => model.running_state = RunningState::Done,
        Message::RefreshProjects => return (None, Some(Box::new(DockerComposeLsCommand {}))),
        Message::Projects(Ok(projects)) => {
            model.projects_panel.list_state.select(
                match (model.projects_panel.list_state.selected(), projects.len()) {
                    (Some(idx), pl) => Some(min(idx, pl - 1)),
                    (_, 0) => None,
                    _ => Some(0),
                },
            );
            model.projects_panel.projects = projects;
        }
        Message::Projects(Err(_)) => {}
        Message::Tick => {}
        Message::KeyPress(key) => return map_key_to_message(key),
        Message::Up => model.projects_panel.list_state.select_previous(),
        Message::Down => model.projects_panel.list_state.select_next(),
    }

    (None, None)
}

fn map_key_to_message(key: KeyEvent) -> (Option<Message>, Option<BoxedCmd>) {
    match key.code {
        KeyCode::Esc | KeyCode::Char('q') => (Some(Message::Quit), None),
        KeyCode::Char('c' | 'C') if key.modifiers == KeyModifiers::CONTROL => {
            (Some(Message::Quit), None)
        }
        KeyCode::Right => (Some(Message::Increment), None),
        KeyCode::Left => (Some(Message::Decrement), None),
        KeyCode::Up | KeyCode::Char('k') => (Some(Message::Up), None),
        KeyCode::Down | KeyCode::Char('j') => (Some(Message::Down), None),
        // Other handlers you could add here.
        _ => (None, None),
    }
}

pub fn subscriptions(_model: &Model) -> Subscription<Message> {
    Subscription::Interval(Duration::from_secs(2), Message::RefreshProjects)
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
