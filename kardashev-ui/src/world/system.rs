use std::{
    future::Future,
    pin::Pin,
};

use super::Resources;
use crate::error::Error;

pub struct RunSystemContext<'c> {
    pub command_buffer: hecs::CommandBuffer,
    pub world: &'c mut hecs::World,
    pub resources: &'c mut Resources,
    // todo: sender to send commands to world server
}

impl<'c> RunSystemContext<'c> {
    pub(super) fn apply_buffered(&mut self) {
        self.command_buffer.run_on(self.world);
    }
}

pub trait System: 'static {
    fn label(&self) -> &'static str;

    fn run<'s: 'c, 'c: 'd, 'd>(
        &'s mut self,
        context: &'d mut RunSystemContext<'c>,
    ) -> impl Future<Output = Result<(), Error>> + 'd;
}

pub trait OneshotSystem: 'static {
    fn label(&self) -> &'static str;

    fn run<'c: 'd, 'd>(
        self,
        context: &'d mut RunSystemContext<'c>,
    ) -> impl Future<Output = Result<(), Error>> + 'd;
}

pub(super) trait DynSystem {
    fn label(&self) -> &'static str;

    fn run<'s: 'c, 'c: 'd, 'd>(
        &'s mut self,
        context: &'d mut RunSystemContext<'c>,
    ) -> Pin<Box<dyn Future<Output = Result<(), Error>> + 'd>>;
}

impl<T: System> DynSystem for T {
    fn label(&self) -> &'static str {
        T::label(self)
    }

    fn run<'s: 'c, 'c: 'd, 'd>(
        &'s mut self,
        context: &'d mut RunSystemContext<'c>,
    ) -> Pin<Box<dyn Future<Output = Result<(), Error>> + 'd>> {
        Box::pin(T::run(self, context))
    }
}

pub(super) trait DynOneshotSystem {
    fn label(&self) -> &'static str;

    fn run<'c: 'd, 'd>(
        self: Box<Self>,
        context: &'d mut RunSystemContext<'c>,
    ) -> Pin<Box<dyn Future<Output = Result<(), Error>> + 'd>>;
}

impl<T: OneshotSystem> DynOneshotSystem for T {
    fn label(&self) -> &'static str {
        T::label(self)
    }

    fn run<'c: 'd, 'd>(
        self: Box<Self>,
        context: &'d mut RunSystemContext<'c>,
    ) -> Pin<Box<dyn Future<Output = Result<(), Error>> + 'd>> {
        Box::pin(T::run(*self, context))
    }
}

pub struct NullSystem;

impl System for NullSystem {
    fn label(&self) -> &'static str {
        "null"
    }

    async fn run<'s: 'c, 'c: 'd, 'd>(
        &'s mut self,
        _context: &'d mut RunSystemContext<'c>,
    ) -> Result<(), Error> {
        Ok(())
    }
}
