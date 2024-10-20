use hecs::Entity;
use nalgebra::{
    Point3,
    Similarity3,
    Unit,
    Vector3,
};

use crate::{
    ecs::{
        server::Tick,
        system::{
            System,
            SystemContext,
        },
    },
    graphics::Error,
};

#[derive(Clone, Debug, Default)]
pub struct Transform {
    pub model_matrix: Similarity3<f32>,
}

impl Transform {
    pub fn look_at(eye: Point3<f32>, look_at: Point3<f32>) -> Self {
        let unit = Unit::face_towards(&(&eye - &look_at), &Vector3::z());
        let (axis, angle) = unit.axis_angle().unwrap();
        Transform {
            model_matrix: Similarity3::new(eye.coords, *axis * angle, 1.0),
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct GlobalTransform {
    pub model_matrix: Similarity3<f32>,
    pub tick_last_updated: Option<Tick>,
}

impl GlobalTransform {
    pub fn as_homogeneous_matrix_array(&self) -> [f32; 16] {
        self.model_matrix
            .to_homogeneous()
            .as_slice()
            .try_into()
            .expect("convert model matrix to array")
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Parent {
    pub entity: Entity,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct LocalToGlobalTransformSystem;

impl System for LocalToGlobalTransformSystem {
    type Error = Error;

    fn label(&self) -> &'static str {
        "local-to-global-transform"
    }

    fn poll_system(&mut self, system_context: &mut SystemContext<'_>) -> Result<(), Self::Error> {
        fn local_to_global(
            entity: Entity,
            local: &Transform,
            global: Option<&mut GlobalTransform>,
            parent: Option<&Parent>,
            tick: Tick,
            world: &hecs::World,
            mut command_buffer: &mut hecs::CommandBuffer,
        ) {
            let mut new_global = None;
            let global = global.unwrap_or_else(|| {
                new_global = Some(GlobalTransform::default());
                new_global.as_mut().unwrap()
            });

            if global
                .tick_last_updated
                .map_or(false, |tick_last_updated| tick_last_updated >= tick)
            {
                return;
            }

            if let Some(parent) = parent {
                let mut parent_query = world
                    .query_one::<(&Transform, Option<&mut GlobalTransform>, Option<&Parent>)>(
                        parent.entity,
                    )
                    .unwrap();

                global.tick_last_updated = Some(tick);

                if let Some((parent_local, mut parent_global, parent_parent)) = parent_query.get() {
                    local_to_global(
                        parent.entity,
                        parent_local,
                        parent_global.as_deref_mut(),
                        parent_parent,
                        tick,
                        world,
                        &mut command_buffer,
                    );

                    global.model_matrix =
                        local.model_matrix * parent_global.as_ref().unwrap().model_matrix;
                }
            }
            else {
                global.model_matrix = local.model_matrix;
                global.tick_last_updated = Some(tick);
            }

            if let Some(global) = new_global {
                command_buffer.insert(entity, (global,));
            }
        }

        let mut query =
            system_context
                .world
                .query::<(&Transform, Option<&mut GlobalTransform>, Option<&Parent>)>();

        for (entity, (local, global, parent)) in query.iter() {
            local_to_global(
                entity,
                local,
                global,
                parent,
                system_context.tick,
                &system_context.world,
                &mut system_context.command_buffer,
            );
        }

        Ok(())
    }
}
