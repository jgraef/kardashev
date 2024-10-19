use crate::ecs::world::World;

use super::{
    schedule::Schedule,
    Resources,
};

pub struct RegisterPluginContext<'a> {
    pub world: &'a mut World,
    pub resources: &'a mut Resources,
    pub startup_schedule: &'a mut Schedule,
    pub schedule: &'a mut Schedule,
}

pub trait Plugin: 'static {
    fn register(self, context: RegisterPluginContext);
}
