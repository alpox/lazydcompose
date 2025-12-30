use crate::{
    cmd::{Cmd, DockerComposeLsCommand},
    event::Message,
    model::{Model, RunningState},
    subs::{Subscription},
};
use std::time::Duration;

type BoxedCmd = Box<dyn Cmd<Model, Message>>;

pub fn update(model: &mut Model, msg: Message) -> (Option<Message>, Option<BoxedCmd>) {
        match msg {
            Message::Increment => model.counter = model.counter.saturating_add(1),
            Message::Decrement => model.counter = model.counter.saturating_sub(1),
            Message::Quit => model.running_state = RunningState::Done,
            Message::RefreshProjects => return (None, Some(Box::new(DockerComposeLsCommand {}))),
            Message::Projects(Ok(projects)) => model.projects = projects,
            Message::Projects(Err(_)) => {}
            Message::Tick => {}
        }

        (None, None)
}

pub fn subscriptions(_model: &Model) -> Subscription<Message> {
    Subscription::Interval(
        Duration::from_secs(2),
        Message::RefreshProjects,
    )
}
