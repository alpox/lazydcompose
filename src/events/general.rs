use crate::{
    effect::Effect,
    model::RunningState,
    system::{Ctx, Handler, Message},
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
        Effect::Batch(self.0.into_iter().map(Effect::Dispatch).collect())
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
pub struct ClearNotes;

impl Handler for ClearNotes {
    fn handle(self, ctx: &mut Ctx) -> Effect<Message> {
        ctx.db.notes = ctx
            .db
            .notes
            .iter()
            .filter(|note| !note.finished())
            .cloned()
            .collect();
        Effect::None
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Resize;

impl Handler for Resize {
    fn handle(self, _ctx: &mut Ctx) -> Effect<Message> {
        Effect::None
    }
}
