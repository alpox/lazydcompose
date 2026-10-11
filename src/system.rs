use std::{any::Any, fmt::Debug, time::Instant};

use crate::{effect::Effect, model::Model};

pub struct Ctx<'a> {
    pub db: &'a mut Model,
    pub now: Instant,
}

pub trait Handler: Clone + PartialEq + Debug + Send + Sync + 'static {
    fn handle(self, ctx: &mut Ctx) -> Effect<Message>;
}

pub type Message = Box<dyn Event>;

impl<T> From<T> for Message
where
    T: Handler,
{
    fn from(value: T) -> Self {
        Box::new(value)
    }
}

impl Clone for Message {
    fn clone(&self) -> Self {
        (**self).clone_box()
    }
}

pub trait Event: Any + Debug + Send + Sync {
    fn handle(self: Box<Self>, ctx: &mut Ctx) -> Effect<Message>;
    fn clone_box(&self) -> Message;
    fn eq_dyn(&self, other: &dyn Event) -> bool;
}

impl<T> Event for T
where
    T: Handler,
{
    fn handle(self: Box<Self>, ctx: &mut Ctx) -> Effect<Message> {
        Handler::handle(*self, ctx)
    }

    fn clone_box(&self) -> Message {
        Box::new(self.clone())
    }

    fn eq_dyn(&self, other: &dyn Event) -> bool {
        let other: &dyn Any = other;
        other.downcast_ref::<T>() == Some(self)
    }
}

impl PartialEq for dyn Event {
    fn eq(&self, other: &Self) -> bool {
        self.eq_dyn(other)
    }
}

impl Eq for dyn Event {}
