pub mod collider;
mod pipeline;

use core::f32;

use hecs::Entity;
use parry3d::{
    query::Ray,
    shape::Shape,
};

use crate::{
    collide::{
        collider::{
            AnyShape,
            ScaleUniformly,
        },
        pipeline::get_pipeline,
    },
    ecs::{
        plugin::{
            Plugin,
            RegisterPluginContext,
        },
        resource::Resources,
        system::SystemContext,
    },
};

pub fn cast_ray<S: Shape + ScaleUniformly + Send + Sync + 'static, T: Send + Sync + 'static>(
    resources: &mut Resources,
    world: &hecs::World,
    ray: &Ray,
    max_time_of_impact: Option<f32>,
    starts_in_solid: bool,
) -> Option<(Entity, f32)> {
    get_pipeline::<T>(resources).cast_ray::<S>(world, ray, max_time_of_impact, starts_in_solid)
}

pub fn sync_colliders<S: Shape + Send + Sync + 'static, T: Send + Sync + 'static>(
    system_context: &mut SystemContext,
) {
    get_pipeline::<T>(system_context.resources).sync::<S>(system_context.world);
}

// todo: impl Debug
#[derive(Default)]
pub struct ColliderPlugin {
    add_sync_systems: Vec<fn(&mut RegisterPluginContext)>,
}

impl ColliderPlugin {
    pub fn with_collider<S: Shape + Send + Sync + 'static, T: Send + Sync + 'static>(
        mut self,
    ) -> Self {
        self.add_sync_systems
            .push(|context| context.schedule.add_system(sync_colliders::<S, T>));
        self
    }
}

impl Plugin for ColliderPlugin {
    fn register(self, mut context: RegisterPluginContext) {
        context.schedule.add_system(sync_colliders::<AnyShape, ()>);

        for add_system in self.add_sync_systems {
            add_system(&mut context);
        }
    }
}
