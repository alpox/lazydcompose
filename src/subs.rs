use std::{pin::Pin, time::Duration};

use futures::{
    Stream, StreamExt,
    stream::{self, select_all},
};
use tokio_stream::wrappers::IntervalStream;

pub type MsgStream<Msg> = Pin<Box<dyn Stream<Item = Msg> + Send>>;

#[derive(PartialEq, Eq, Clone)]
pub enum Subscription<Msg> {
    None,
    Interval(Duration, Msg),
    Batch(Vec<Subscription<Msg>>),
}

impl<Msg> Subscription<Msg>
where
    Msg: Send + Sync + Clone + 'static,
{
    pub fn build_stream(&self) -> MsgStream<Msg> {
        match self {
            Subscription::None => Box::pin(stream::empty()),
            Subscription::Interval(period, msg) => {
                let mut interval = tokio::time::interval(*period);
                let message = msg.clone();

                interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);

                Box::pin(IntervalStream::new(interval).map(move |_| message.clone()))
            }
            Subscription::Batch(subs) => {
                let streams: Vec<_> = subs.iter().map(|sub| sub.build_stream()).collect();
                let combined_subs = select_all(streams);

                Box::pin(combined_subs)
            }
        }
    }
}

impl<Msg> Subscription<Msg> {
    pub fn map<F, NewMsg>(self, f: F) -> Subscription<NewMsg>
    where
        F: Fn(Msg) -> NewMsg + Clone,
    {
        match self {
            Subscription::None => Subscription::None,
            Subscription::Interval(d, msg) => Subscription::Interval(d, f(msg)),
            Subscription::Batch(subs) => {
                Subscription::Batch(subs.into_iter().map(|s| s.map(f.clone())).collect())
            }
        }
    }
}
