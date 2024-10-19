use std::{
    fmt::Debug,
    future::Future,
    task::Poll,
};

use super::Resources;
use crate::ecs::{tick::{
    EachTick,
    TickRate,
}, world::World};

pub struct SystemContext<'c> {
    pub world: &'c mut World,
    pub resources: &'c mut Resources,
    pub command_buffer: hecs::CommandBuffer,
    pub add_systems: Vec<DynSystem>,
}

impl<'c> SystemContext<'c> {
    pub(super) fn apply_buffered(&mut self) {
        self.command_buffer.run_on(self.world);
    }

    pub fn add_system(&mut self, system: impl System) {
        self.add_systems.push(system.dyn_system());
    }
}

pub trait System: Sized + 'static {
    type Error: std::error::Error + Send + Sync + 'static;

    fn label(&self) -> &'static str;

    fn poll_system(
        &mut self,
        task_context: &mut std::task::Context<'_>,
        system_context: &mut SystemContext<'_>,
    ) -> Poll<Result<(), Self::Error>>;

    fn dyn_system(self) -> DynSystem {
        DynSystem::new(self)
    }
}

pub trait SystemExt: System + Sized {
    fn run(
        &mut self,
        system_context: &mut SystemContext<'_>,
    ) -> impl Future<Output = Result<(), Self::Error>> {
        std::future::poll_fn(|task_context| self.poll_system(task_context, system_context))
    }

    fn each_tick<T: TickRate>(self) -> EachTick<T, Self> {
        EachTick::new(self)
    }
}

impl<S: System> SystemExt for S {}

pub struct DynSystem {
    inner: Box<dyn DynSystemTrait>,
}

impl DynSystem {
    pub fn new(system: impl System) -> Self {
        Self {
            inner: Box::new(system),
        }
    }
}

impl System for DynSystem {
    type Error = DynSystemError;

    fn label(&self) -> &'static str {
        self.inner.label()
    }

    fn poll_system(
        &mut self,
        task_context: &mut std::task::Context<'_>,
        system_context: &mut SystemContext<'_>,
    ) -> Poll<Result<(), Self::Error>> {
        self.inner
            .poll_system(task_context, system_context)
            .map_err(|error| DynSystemError { error })
    }

    fn dyn_system(self) -> DynSystem {
        self
    }
}

impl Debug for DynSystem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DynSystem")
            .field("label", &self.inner.label())
            .finish()
    }
}

trait DynSystemTrait {
    fn label(&self) -> &'static str;

    fn poll_system(
        &mut self,
        task_context: &mut std::task::Context<'_>,
        system_context: &mut SystemContext<'_>,
    ) -> Poll<Result<(), Box<dyn std::error::Error + Send + Sync + 'static>>>;
}

impl<T: System> DynSystemTrait for T {
    fn label(&self) -> &'static str {
        T::label(self)
    }

    fn poll_system(
        &mut self,
        task_context: &mut std::task::Context<'_>,
        system_context: &mut SystemContext<'_>,
    ) -> Poll<Result<(), Box<dyn std::error::Error + Send + Sync + 'static>>> {
        <T as System>::poll_system(self, task_context, system_context)
            .map_err(|error| Box::new(error) as Box<dyn std::error::Error + Send + Sync + 'static>)
    }
}

#[derive(Debug, thiserror::Error)]
#[error("{error}")]
pub struct DynSystemError {
    error: Box<dyn std::error::Error + Send + Sync + 'static>,
}
