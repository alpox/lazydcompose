use std::time::Duration;

use crate::{
    events::{
        general::ClearNotes,
        project::{RefreshProjectInfo, RefreshProjects},
    },
    model::{Model, ViewId},
    subs::Subscription,
    system::Message,
};

pub mod bindings;
pub mod container;
pub mod general;
pub mod project;
pub mod prompt;

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
                project_name: project.name.clone(),
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
