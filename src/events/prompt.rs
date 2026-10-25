use crate::{
    effect::Effect,
    system::{Ctx, Handler, Message},
};

#[derive(Clone, Debug, PartialEq)]
pub struct ShowPrompt;

impl Handler for ShowPrompt {
    fn handle(self, _ctx: &mut Ctx) -> Effect<Message> {
        Effect::None
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ClosePrompt;

impl Handler for ClosePrompt {
    fn handle(self, ctx: &mut Ctx) -> Effect<Message> {
        ctx.db.overlay = None;
        Effect::None
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ConfirmPrompt;

impl Handler for ConfirmPrompt {
    fn handle(self, ctx: &mut Ctx) -> Effect<Message> {
        match ctx.db.take_prompt() {
            Some(prompt) => prompt.then.map(|msg| msg),
            None => Effect::None,
        }
    }
}
