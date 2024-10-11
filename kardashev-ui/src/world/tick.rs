use std::{
    marker::PhantomData,
    num::NonZeroU64,
    task::{
        ready,
        Poll,
    },
};

use serde::{
    Deserialize,
    Serialize,
};

use crate::{
    utils::futures::Interval,
    world::{
        resource::ResourceNotFound,
        system::{
            System,
            SystemContext,
        },
    },
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Tick(NonZeroU64);

impl Default for Tick {
    fn default() -> Self {
        Tick(unsafe { NonZeroU64::new_unchecked(1) })
    }
}

impl Tick {
    pub fn next(self) -> Self {
        Tick(NonZeroU64::new(self.0.get() + 1).expect("tick number overflow"))
    }
}

/// Runs a system once every tick. The tick rate is given by a resource `T`.
#[derive(Debug)]
pub struct EachTick<T, S> {
    system: S,
    previously_seen_tick: Option<Tick>,
    state: EachTickState,
    _tick_resource: PhantomData<T>,
}

#[derive(Clone, Copy, Debug)]
enum EachTickState {
    PollTick,
    PollSystem { tick: Tick },
}

impl<T, S> EachTick<T, S> {
    pub fn new(system: S) -> Self {
        Self {
            system,
            previously_seen_tick: None,
            state: EachTickState::PollTick,
            _tick_resource: PhantomData,
        }
    }
}

impl<T: TickRate, S: System> System for EachTick<T, S> {
    type Error = TickSystemError<S::Error>;

    fn label(&self) -> &'static str {
        self.system.label()
    }

    fn poll_system(
        &mut self,
        task_context: &mut std::task::Context<'_>,
        system_context: &mut SystemContext<'_>,
    ) -> Poll<Result<(), Self::Error>> {
        loop {
            match self.state {
                EachTickState::PollTick => {
                    let tick_resource = system_context.resources.try_get_mut::<T>()?;
                    let tick = ready!(tick_resource.poll_for(task_context, |current_tick| {
                        self.previously_seen_tick
                            .map_or(true, |previously_seen_tick| {
                                current_tick > previously_seen_tick
                            })
                    }));
                    self.state = EachTickState::PollSystem { tick };
                }
                EachTickState::PollSystem { tick } => {
                    ready!(self.system.poll_system(task_context, system_context))
                        .map_err(TickSystemError::System)?;
                    self.previously_seen_tick = Some(tick);
                    self.state = EachTickState::PollTick;
                }
            }
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum TickSystemError<E> {
    #[error("tick system error: inner system error")]
    System(#[source] E),
    #[error("tick system error: resource not found")]
    ResourceNotFound(#[from] ResourceNotFound),
}

pub trait TickRate: 'static {
    fn current(&self) -> Tick;
    fn poll_for<F: Fn(Tick) -> bool>(
        &mut self,
        task_context: &mut std::task::Context<'_>,
        f: F,
    ) -> Poll<Tick>;
}

#[derive(Debug)]
pub struct FixedTick {
    pub interval: Interval,
    pub current_tick: Tick,
}

impl FixedTick {
    pub fn new(interval: Interval) -> Self {
        Self {
            interval,
            current_tick: Tick::default(),
        }
    }
}

impl TickRate for FixedTick {
    fn current(&self) -> Tick {
        self.current_tick
    }

    fn poll_for<F: Fn(Tick) -> bool>(
        &mut self,
        task_context: &mut std::task::Context<'_>,
        f: F,
    ) -> Poll<Tick> {
        while !f(self.current_tick) {
            ready!(self.interval.poll_tick(task_context));
            self.current_tick = self.current_tick.next();
        }

        Poll::Ready(self.current_tick)
    }
}
