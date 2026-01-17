use futures::future::BoxFuture;
use itertools::Itertools;
use std::fmt::Debug;
use tokio::sync::mpsc::UnboundedSender;

pub type Task<Msg> = BoxFuture<'static, Option<Msg>>;

pub enum Effect<Msg: Send> {
    None,
    Dispatch(Msg),
    Async(Task<Msg>),
    Batch(Vec<Effect<Msg>>),
    Blocking(Task<Msg>),
}

impl<Msg: Send + Debug> Debug for Effect<Msg> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::None => write!(f, "None"),
            Self::Dispatch(arg0) => f.debug_tuple("Dispatch").field(arg0).finish(),
            Self::Async(_) => f.debug_tuple("Async").finish(),
            Self::Batch(arg0) => f.debug_tuple("Batch").field(arg0).finish(),
            Self::Blocking(_) => f.debug_tuple("Blocking").finish(),
        }
    }
}

impl<Msg> Effect<Msg>
where
    Msg: Send + 'static,
{
    pub fn map<M, F>(self, f: F) -> Effect<M>
    where
        M: Send + 'static,
        F: Fn(Msg) -> M + Clone + Send + 'static,
    {
        match self {
            Self::None => Effect::None,
            Self::Dispatch(msg) => Effect::Dispatch(f(msg)),
            Self::Batch(effects) => {
                Effect::Batch(effects.into_iter().map(|eff| eff.map(f.clone())).collect())
            }
            Self::Async(fut) => Effect::Async(Box::pin(async move { fut.await.map(f) })),
            Self::Blocking(fut) => Effect::Blocking(Box::pin(async move { fut.await.map(f) })),
        }
    }

    pub fn process(self, sender: UnboundedSender<Msg>) -> Option<Task<Msg>> {
        match self {
            Self::None => None,
            Self::Dispatch(msg) => {
                let _ = sender.send(msg);
                None
            }
            Self::Batch(effects) => {
                let sorted_effects = effects.into_iter().sorted_by_key(|e| match e {
                    Effect::None => 0,
                    Effect::Batch(_) => 1,
                    Effect::Dispatch(_) => 2,
                    Effect::Async(_) => 3,
                    Effect::Blocking(_) => 4,
                });

                for effect in sorted_effects {
                    effect.process(sender.clone());
                }
                None
            }
            Self::Async(fut) => {
                tokio::spawn(async move {
                    if let Some(msg) = fut.await {
                        let _ = sender.send(msg);
                    }
                });
                None
            }
            Self::Blocking(fut) => Some(fut),
        }
    }

    pub fn dispatch(msg: Msg) -> Self {
        Self::Dispatch(msg)
    }

    pub fn batch(effects: Vec<Effect<Msg>>) -> Self {
        Self::Batch(effects)
    }

    pub fn perform<F>(fut: F) -> Self
    where
        F: Future<Output = Option<Msg>> + Send + 'static,
    {
        Self::Async(Box::pin(fut))
    }

    pub fn perform_blocking<F>(fut: F) -> Self
    where
        F: Future<Output = Option<Msg>> + Send + 'static,
    {
        Self::Blocking(Box::pin(fut))
    }
}

impl<T, Msg> From<T> for Effect<Msg>
where
    Msg: Send + 'static,
    T: Future<Output = Option<Msg>> + Send + 'static,
{
    fn from(value: T) -> Self {
        Effect::perform(value)
    }
}
