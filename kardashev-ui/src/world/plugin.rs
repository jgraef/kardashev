use super::{
    schedule::Scheduler,
    Resources,
};

pub struct RegisterPluginContext<'a> {
    pub world: &'a mut hecs::World,
    pub resources: &'a mut Resources,
    pub scheduler: &'a mut Scheduler,
}

pub trait Plugin: 'static {
    fn register(self, context: RegisterPluginContext);
}
