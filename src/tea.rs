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
    model::{PanelId, RunningState, UpdateResult},
    panels::projects::{self, ProjectsPanel},
    subs::Subscription,
};
use std::{cmp::min, time::Duration};

pub type BoxedCmd = Box<dyn Cmd<Model, Message>>;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Model {
    pub running_state: RunningState,
    pub counter: u8,
    pub projects_panel: ProjectsPanel,
    pub active_panel: PanelId,
}

impl Default for Model {
    fn default() -> Self {
        Self {
            running_state: RunningState::Running,
            counter: 0,
            active_panel: PanelId::default(),
            projects_panel: ProjectsPanel::default(),
        }
    }
}

pub fn update(model: &mut Model, msg: Message) -> UpdateResult {
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
        Message::RefreshProjects => UpdateResult::Cmd(Box::new(DockerComposeLsCommand {})),
        Message::Projects(Ok(projects)) => {
            model.projects_panel.list_state.select(
                match (model.projects_panel.list_state.selected(), projects.len()) {
                    (Some(idx), pl) => Some(min(idx, pl - 1)),
                    (_, 0) => None,
                    _ => Some(0),
                },
            );
            model.projects_panel.projects = projects;
            UpdateResult::None
        }
        Message::Projects(Err(_)) => UpdateResult::None,
        Message::Tick => UpdateResult::None,
        Message::KeyPress(key) => map_key_to_message(key),

        msg => match model.active_panel {
            PanelId::Projects => projects::update(&mut model.projects_panel, msg),
            _ => UpdateResult::None,
        },
    }
}

fn map_key_to_message(key: KeyEvent) -> UpdateResult {
    match key.code {
        KeyCode::Esc | KeyCode::Char('q') => UpdateResult::Msg(Message::Quit),
        KeyCode::Char('c' | 'C') if key.modifiers == KeyModifiers::CONTROL => {
            UpdateResult::Msg(Message::Quit)
        }
        KeyCode::Right => UpdateResult::Msg(Message::Increment),
        KeyCode::Left => UpdateResult::Msg(Message::Decrement),
        KeyCode::Up | KeyCode::Char('k') => UpdateResult::Msg(Message::Up),
        KeyCode::Down | KeyCode::Char('j') => UpdateResult::Msg(Message::Down),
        // Other handlers you could add here.
        _ => UpdateResult::None,
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
