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

    fn poll_system(&mut self, system_context: &mut SystemContext<'_>) -> Result<(), Self::Error> {
        for system in &mut self.systems {
            system.poll_system(system_context).map_err(|error| {
                Error::System {
                    system: system.label(),
                    error,
                }
            })?;
        }

        Ok(())
    }
}
