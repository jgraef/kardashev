use std::{
    any::type_name,
    convert::Infallible,
    fmt::Debug,
};

use super::Resources;
use crate::ecs::server::Tick;

pub struct SystemContext<'c> {
    pub world: &'c mut hecs::World,
    pub resources: &'c mut Resources,
    pub command_buffer: &'c mut hecs::CommandBuffer,
    pub tick: Tick,
}

impl<'c> SystemContext<'c> {
    pub(super) fn apply_buffered(&mut self) {
        self.command_buffer.run_on(self.world);
    }
}

pub trait System: Sized + 'static {
    type Error: std::error::Error + Send + Sync + 'static;

    fn label(&self) -> &'static str {
        type_name::<Self>()
    }

    fn poll_system(&mut self, system_context: &mut SystemContext<'_>) -> Result<(), Self::Error>;

    fn dyn_system(self) -> DynSystem {
        DynSystem::new(self)
    }
}

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

    fn poll_system(&mut self, system_context: &mut SystemContext<'_>) -> Result<(), Self::Error> {
        self.inner
            .poll_system(system_context)
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
        system_context: &mut SystemContext<'_>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>>;
}

impl<T: System> DynSystemTrait for T {
    fn label(&self) -> &'static str {
        T::label(self)
    }

    fn poll_system(
        &mut self,
        system_context: &mut SystemContext<'_>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
        <T as System>::poll_system(self, system_context)
            .map_err(|error| Box::new(error) as Box<dyn std::error::Error + Send + Sync + 'static>)
    }
}

#[derive(Debug, thiserror::Error)]
#[error("{error}")]
pub struct DynSystemError {
    error: Box<dyn std::error::Error + Send + Sync + 'static>,
}

impl DynSystemError {
    pub fn custom(error: impl std::error::Error + Send + Sync + 'static) -> Self {
        Self {
            error: Box::new(error),
        }
    }
}

impl<F> System for F
where
    F: FnMut(&mut SystemContext<'_>) + Send + Sync + 'static,
{
    type Error = Infallible;

    fn poll_system(&mut self, system_context: &mut SystemContext<'_>) -> Result<(), Self::Error> {
        self(system_context);
        Ok(())
    }
}
