use std::task::Poll;

use crate::ecs::{
    system::{
        DynSystem,
        System,
        SystemContext,
    },
    Error,
};

#[derive(Debug, Default)]
pub struct Schedule {
    systems: Vec<DynSystem>,
}

impl Schedule {
    pub fn add_system(&mut self, system: impl System) {
        self.systems.push(system.dyn_system());
    }
}

impl System for Schedule {
    type Error = Error;

    fn label(&self) -> &'static str {
        "schedule"
    }

    fn poll_system(
        &mut self,
        task_context: &mut std::task::Context<'_>,
        system_context: &mut SystemContext<'_>,
    ) -> Poll<Result<(), Self::Error>> {
        let mut i = 0;

        while i < self.systems.len() {
            let system = &mut self.systems[i];

            match system.poll_system(task_context, system_context) {
                Poll::Ready(Ok(())) => {
                    let system = self.systems.remove(i);
                    tracing::debug!(label = %system.label(), "system done");
                }
                Poll::Ready(Err(error)) => {
                    return Poll::Ready(Err(Error::System {
                        system: system.label(),
                        error,
                    }))
                }
                Poll::Pending => {
                    i += 1;
                }
            }
        }

        if self.systems.is_empty() {
            Poll::Ready(Ok(()))
        }
        else {
            Poll::Pending
        }
    }
}
