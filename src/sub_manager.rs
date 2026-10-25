use std::{pin::Pin, sync::Arc};

use futures::{Stream, StreamExt};
use tokio::sync::{
    Mutex,
    mpsc::{self, UnboundedReceiver, UnboundedSender, unbounded_channel},
};
use tokio_stream::{StreamMap, wrappers::ReceiverStream};

use crate::subs::Subscription;

type MsgStream<Msg> = Pin<Box<dyn Stream<Item = Msg> + Send>>;

pub struct SubscriptionManager<Msg: Send> {
    subs_sender: UnboundedSender<Vec<Subscription<Msg>>>,
    subs_receiver: Arc<Mutex<UnboundedReceiver<Vec<Subscription<Msg>>>>>,
    current_sub: Subscription<Msg>,
}

impl<Msg> Default for SubscriptionManager<Msg>
where
    Msg: PartialEq + Clone + Send,
{
    fn default() -> Self {
        let (subs_sender, subs_receiver) = unbounded_channel::<Vec<Subscription<Msg>>>();

        Self {
            subs_sender,
            subs_receiver: Arc::new(Mutex::new(subs_receiver)),
            current_sub: Subscription::None,
        }
    }
}

impl<Msg> SubscriptionManager<Msg>
where
    Msg: PartialEq + Clone + Send + 'static + std::fmt::Debug,
{
    pub fn new() -> Self {
        Self::default()
    }

    pub fn stream(&self) -> MsgStream<Msg> {
        let (sender, receiver) = mpsc::channel::<Msg>(1);
        let subs_receiver = self.subs_receiver.clone();

        tokio::spawn(async move {
            let mut subs_receiver = subs_receiver.lock().await;
            let mut running: Vec<(usize, Subscription<Msg>)> = Vec::new();
            let mut streams = StreamMap::new();
            let mut next_id = 0;

            loop {
                tokio::select! {
                    Some(subs) = subs_receiver.recv() => {
                        running.retain(|(id, sub)| {
                            let keep = subs.contains(sub);
                            if !keep {
                                streams.remove(id);
                            }

                            keep
                        });

                        for sub in subs {
                            if running.iter().any(|(_, running_sub)| *running_sub == sub) {
                                continue;
                            }
                            streams.insert(next_id, sub.build_stream());
                            running.push((next_id, sub));
                            next_id += 1;
                        }
                    }

                    Some((_, msg)) = streams.next(), if !streams.is_empty() => {
                        if sender.send(msg).await.is_err() {
                            break;
                        }
                    }

                    else => break
                }
            }
        });

        Box::pin(ReceiverStream::new(receiver))
    }

    pub fn update(&mut self, sub: Subscription<Msg>) {
        if self.current_sub != sub {
            let _ = self.subs_sender.send(sub.clone().leaves());
            self.current_sub = sub;
        }
    }
}
