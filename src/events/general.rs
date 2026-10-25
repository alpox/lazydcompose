use crossterm::event::KeyEvent;

use crate::{
    effect::Effect,
    event::{self},
    model::RunningState,
    system::{Ctx, Handler, Message},
    tea,
};

#[derive(Clone, Debug, PartialEq)]
pub struct Batch(pub Vec<Message>);

impl Batch {
    pub fn new(messages: impl IntoIterator<Item = Message>) -> Batch {
        Batch(messages.into_iter().collect())
    }
}

impl Handler for Batch {
    fn handle(self, _ctx: &mut Ctx) -> Effect<Message> {
        let effects: Vec<Effect<_>> = self
            .0
            .iter()
            .map(|event| Effect::Dispatch(event.clone()))
            .collect();

        Effect::Batch(effects)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct QuitEvent;

impl Handler for QuitEvent {
    fn handle(self, ctx: &mut Ctx) -> Effect<Message> {
        ctx.db.running_state = RunningState::Done;
        Effect::None
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Tick;

impl Handler for Tick {
    fn handle(self, _ctx: &mut Ctx) -> Effect<Message> {
        Effect::None
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Compat {
    pub message: event::Message,
}

impl Handler for Compat {
    fn handle(self, ctx: &mut Ctx) -> Effect<Message> {
        tea::update(ctx.db, self.message).map(Message::from)
    }
}

impl From<event::Message> for Message {
    fn from(message: event::Message) -> Self {
        match message {
            event::Message::Event(event) => event,
            message => Compat { message }.into(),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct KeyPress {
    key: KeyEvent,
}
