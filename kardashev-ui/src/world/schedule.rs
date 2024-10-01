use futures::StreamExt;
use tracing::Instrument;

use super::{
    system::{
        DynOneshotSystem,
        DynSystem,
        OneshotSystem,
    },
    RunSystemContext,
    System,
};
use crate::error::Error;

pub struct Scheduler {
    pub(super) startup_systems: Vec<Box<dyn DynOneshotSystem>>,
    pub(super) update_systems: TimedSystems,
    pub(super) render_systems: TimedSystems,
}

impl Default for Scheduler {
    fn default() -> Self {
        Self {
            startup_systems: vec![],
            update_systems: TimedSystems::new(60),
            render_systems: TimedSystems::new(60),
        }
    }
}

impl Scheduler {
    pub fn set_fps(&mut self, fps: u32) {
        self.render_systems.set_ups(fps);
    }

    pub fn set_ups(&mut self, ups: u32) {
        self.update_systems.set_ups(ups);
    }

    pub fn add_startup_system(&mut self, system: impl OneshotSystem) {
        self.startup_systems.push(Box::new(system));
    }

    pub fn add_update_system(&mut self, system: impl System) {
        self.update_systems.add_system(system);
    }

    pub fn add_render_system(&mut self, system: impl System) {
        self.render_systems.add_system(system);
    }
}

pub(super) struct TimedSystems {
    interval: Interval,
    systems: Vec<Box<dyn DynSystem>>,
}

impl TimedSystems {
    pub fn new(ups: u32) -> Self {
        Self {
            interval: Interval::new(ups),
            systems: vec![],
        }
    }

    pub fn set_ups(&mut self, ups: u32) {
        self.interval = Interval::new(ups);
    }

    pub fn add_system(&mut self, system: impl System) {
        self.systems.push(Box::new(system));
    }

    pub async fn wait(&mut self) {
        self.interval.tick().await;
    }

    pub async fn run<'s: 'c, 'c: 'd, 'd>(
        &'s mut self,
        context: &'d mut RunSystemContext<'c>,
    ) -> Result<(), Error> {
        for system in &mut self.systems {
            let span = tracing::debug_span!("system", label = system.label());
            system.run(context).instrument(span).await?;
        }

        Ok(())
    }
}

struct Interval {
    inner: gloo_timers::future::IntervalStream,
}

impl Interval {
    pub fn new(ups: u32) -> Self {
        Self {
            inner: gloo_timers::future::IntervalStream::new(1000 / ups),
        }
    }

    pub async fn tick(&mut self) {
        self.inner.next().await;
    }
}
