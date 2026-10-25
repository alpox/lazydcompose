use std::time::Instant;

use crossterm::event::KeyEvent;

use crate::{
    bindings::bindings,
    effect::Effect,
    model::Overlay,
    system::{Ctx, Handler, Message},
};

#[derive(Clone, Debug, PartialEq)]
pub struct PressKey {
    pub key: KeyEvent,
}

impl Handler for PressKey {
    fn handle(self, ctx: &mut Ctx) -> Effect<Message> {
        if let Some(resolved_event) = bindings().resolve_event(&self.key, ctx.db) {
            resolved_event.handle(&mut Ctx {
                db: ctx.db,
                now: Instant::now(),
            })
        } else {
            Effect::None
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ShowBindings;

impl Handler for ShowBindings {
    fn handle(self, ctx: &mut Ctx) -> Effect<Message> {
        ctx.db.overlay = Some(Overlay::Bindings { selected: 0 });
        Effect::None
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct SelectNextBinding;

impl Handler for SelectNextBinding {
    fn handle(self, ctx: &mut Ctx) -> Effect<Message> {
        let bindings = [bindings().bindings_for(ctx.db), bindings().global()].concat();
        let count = bindings.iter().len();

        if let Some(Overlay::Bindings { selected }) = &mut ctx.db.overlay {
            *selected = (*selected + 1) % count;
        }
        Effect::None
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct SelectPreviousBinding;

impl Handler for SelectPreviousBinding {
    fn handle(self, ctx: &mut Ctx) -> Effect<Message> {
        let bindings = [bindings().bindings_for(ctx.db), bindings().global()].concat();
        let count = bindings.iter().len();

        if let Some(Overlay::Bindings { selected }) = &mut ctx.db.overlay {
            *selected = (*selected + count - 1) % count;
        }
        Effect::None
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct SelectBinding;

impl Handler for SelectBinding {
    fn handle(self, ctx: &mut Ctx) -> Effect<Message> {
        let Some(Overlay::Bindings { selected }) = ctx.db.overlay else {
            return Effect::None;
        };

        ctx.db.overlay = None;

        let bindings = [bindings().bindings_for(ctx.db), bindings().global()].concat();

        let Some(binding) = bindings.get(selected) else {
            return Effect::None;
        };

        if let Some(event) = &binding.event {
            return event(ctx.db).map_or(Effect::None, Effect::Dispatch);
        }

        match binding.keys.first() {
            Some(key) => Effect::Dispatch(PressKey { key: key.into() }.into()),
            None => Effect::None,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct CloseBindings;

impl Handler for CloseBindings {
    fn handle(self, ctx: &mut Ctx) -> Effect<Message> {
        ctx.db.overlay = None;
        Effect::None
    }
}
