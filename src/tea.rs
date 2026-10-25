use std::time::Duration;

use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    widgets::{Paragraph, Widget},
};

use crate::{
    events::{
        general::ClearNotes,
        project::{RefreshProjectInfo, RefreshProjects},
    },
    model::{Model, Overlay, ViewId},
    panels::{
        project,
        projects::{self},
    },
    subs::Subscription,
    system::Message,
    ui::{
        bindings::Bindings,
        notes::Notes,
        prompt::{self},
    },
};

pub fn subscriptions(model: &Model) -> Subscription<Message> {
    let mut subscriptions = vec![Subscription::Interval(
        Duration::from_secs(1),
        RefreshProjects.into(),
    )];

    if model.active_view == ViewId::Info
        && let Some(project) = model.selected_project()
    {
        subscriptions.push(Subscription::Interval(
            Duration::from_secs(1),
            RefreshProjectInfo {
                project_name: project.name,
            }
            .into(),
        ))
    }

    if !model.notes.is_empty() {
        subscriptions.push(Subscription::Interval(
            Duration::from_secs(1),
            ClearNotes.into(),
        ))
    }

    Subscription::Batch(subscriptions)
}

struct AppLayout {
    pub main: Rect,
    pub hints: Rect,
}

fn layout(area: Rect) -> AppLayout {
    let [main, hints] = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Fill(1), Constraint::Length(1)])
        .areas(area);

    AppLayout { main, hints }
}

pub fn view(model: &mut Model, frame: &mut Frame) {
    let layout = layout(frame.area());

    match model.active_view {
        ViewId::Info if model.active_project_index.is_some() => project::view(
            model,
            model.active_project_index.unwrap(),
            true,
            frame,
            layout.main,
        ),
        _ => projects::view(model, frame, layout.main),
    };

    Notes::new(model.notes.clone()).render(frame.area(), frame.buffer_mut());

    match &model.overlay {
        Some(Overlay::Bindings { selected: _ }) => {
            Bindings::new(model).render(frame.area(), frame.buffer_mut());
        }
        Some(Overlay::Prompt(prompt)) => {
            prompt::Prompt::new(&prompt.title, &prompt.text)
                .render(frame.area(), frame.buffer_mut());
        }
        None => {}
    }

    Paragraph::new("?: Keybindings").render(layout.hints, frame.buffer_mut());
}
