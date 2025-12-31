use std::{pin::Pin, sync::Arc, time::Duration};

use futures::{
    Stream, StreamExt,
    stream::{self, select_all},
};
use tokio::{sync::{
    Mutex,
    mpsc::{UnboundedReceiver, UnboundedSender, unbounded_channel},
}};
use tokio_stream::wrappers::{IntervalStream, UnboundedReceiverStream};

#[derive(Eq, PartialEq, Clone, Debug)]
pub enum Subscription<Msg> {
    None,
    Interval(Duration, Msg),
    Batch(Vec<Subscription<Msg>>),
}

type MsgStream<Msg> = Pin<Box<dyn Stream<Item = Msg> + Send>>;

pub struct Sub<Msg: Send> {
    stream_sender: UnboundedSender<MsgStream<Msg>>,
    stream_receiver: Arc<Mutex<UnboundedReceiver<MsgStream<Msg>>>>,
    current_sub: Subscription<Msg>,
}

impl<Msg> Default for Sub<Msg>
where
    Msg: PartialEq + Clone + Send,
{
    fn default() -> Self {
        let (stream_sender, stream_receiver) = unbounded_channel::<MsgStream<Msg>>();

        Self {
            stream_sender,
            stream_receiver: Arc::new(Mutex::new(stream_receiver)),
            current_sub: Subscription::None,
        }
    }
}

impl<Msg> Sub<Msg>
where
    Msg: Sync + PartialEq + Clone + Send + 'static + std::fmt::Debug,
{
    pub fn new() -> Self {
        Self::default()
    }

    pub fn stream(&self) -> MsgStream<Msg> {
        let (sender, receiver) = unbounded_channel::<Msg>();
        let stream_receiver = self.stream_receiver.clone();

        tokio::spawn(async move {
            let mut current_stream: Option<MsgStream<Msg>> = None;
            let mut locked_stream_receiver = stream_receiver.lock().await;

            loop {
                tokio::select! {
                    Some(new_stream) = locked_stream_receiver.recv() => {
                        current_stream = Some(new_stream)
                    }

                    Some(msg) = async {
                        match &mut current_stream {
                            Some(stream) => stream.next().await,
                            None => std::future::pending().await
                        }
                    } => {
                        if sender.send(msg).is_err() {
                            break
                        }
                    }
                }
            }
        });

        Box::pin(UnboundedReceiverStream::new(receiver))
    }

    pub fn build_stream(&self, sub: &Subscription<Msg>) -> Pin<Box<dyn Stream<Item = Msg> + Send>> {
        match sub {
            Subscription::None => Box::pin(stream::empty()),
            Subscription::Interval(period, msg) => {
                let interval = tokio::time::interval(*period);
                let message = msg.clone();

                Box::pin(IntervalStream::new(interval).map(move |_| message.clone()))
            }
            Subscription::Batch(subs) => {
                let streams: Vec<_> = subs.iter().map(|sub| self.build_stream(sub)).collect();
                let combined_subs = select_all(streams);

                Box::pin(combined_subs)
            }
        }
    }

    pub fn update(&mut self, sub: Subscription<Msg>) {
        if self.current_sub != sub {
            let stream = self.build_stream(&sub);
            let _ = self.stream_sender.send(stream);
            self.current_sub = sub;
        }
    }
}
