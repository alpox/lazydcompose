use crossterm::event::KeyEvent;

use crate::{effect::Effect, model::RunningState, system::Ctx, system::Handler, system::Message};

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
pub struct KeyPress {
    key: KeyEvent,
}
