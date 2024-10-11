use std::task::Poll;

use crate::world::{
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
        while !self.systems.is_empty() {
            let mut i = 0;
            let mut pending = false;

            while i < self.systems.len() {
                let system = &mut self.systems[i];

                match system.poll_system(task_context, system_context) {
                    Poll::Ready(Ok(())) => {
                        self.systems.remove(i);
                    }
                    Poll::Ready(Err(error)) => {
                        return Poll::Ready(Err(Error::System {
                            system: system.label(),
                            error,
                        }))
                    }
                    Poll::Pending => {
                        pending = true;
                        i += 1;
                    }
                }
            }

            if pending {
                return Poll::Pending;
            }
        }

        Poll::Ready(Ok(()))
    }
}
