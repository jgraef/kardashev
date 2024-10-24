use hecs::Entity;
use nalgebra::{
    Point3,
    Similarity3,
    Translation3,
    Unit,
    UnitQuaternion,
    Vector3,
};

use crate::ecs::{
    server::Tick,
    system::SystemContext,
};

#[derive(Clone, Debug, Default)]
pub struct Transform {
    pub model_matrix: Similarity3<f32>,
}

impl Transform {
    pub fn from_position(position: Point3<f32>) -> Self {
        Self {
            model_matrix: Similarity3::from_parts(
                Translation3::from(position.coords),
                UnitQuaternion::identity(),
                1.0,
            ),
        }
    }

    pub fn look_at(eye: Point3<f32>, look_at: Point3<f32>, up: Vector3<f32>) -> Self {
        // according to `Unit:face_towards` this needs to be `look_at - eye`, but our Z
        // axis is reversed, so it needs to be this way.
        let dir = &eye - &look_at;

        let quaternion = Unit::face_towards(&dir, &up);
        Transform {
            model_matrix: Similarity3::from_parts(Translation3::from(eye.coords), quaternion, 1.0),
        }
    }

    pub fn with_rotation(mut self, rotation: UnitQuaternion<f32>) -> Self {
        self.model_matrix.append_rotation_wrt_center_mut(&rotation);
        self
    }

    pub fn with_scaling(mut self, scaling: f32) -> Self {
        self.model_matrix = self.model_matrix.append_scaling(scaling);
        self
    }
}

#[derive(Clone, Debug)]
pub struct GlobalTransform {
    pub model_matrix: Similarity3<f32>,
    pub tick_last_updated: Tick,
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

pub fn local_to_global_transform_system(system_context: &mut SystemContext) {
    type TransformView<'a> = (&'a Transform, Option<&'a mut GlobalTransform>);
    type HierarchyView<'a> = &'a Parent;

    fn local_to_global(
        entity: Entity,
        transform_view: &mut hecs::ViewBorrow<TransformView>,
        hierarchy_view: &hecs::ViewBorrow<HierarchyView>,
        tick: Tick,
        command_buffer: &mut hecs::CommandBuffer,
    ) -> Similarity3<f32> {
        let parent_global = if let Some(parent) = hierarchy_view.get(entity) {
            local_to_global(
                parent.entity,
                transform_view,
                hierarchy_view,
                tick,
                command_buffer,
            )
        }
        else {
            Default::default()
        };

        let (local, global) = transform_view.get_mut(entity).unwrap();

        if let Some(global) = global {
            if global.tick_last_updated < tick {
                global.model_matrix = local.model_matrix * parent_global;
                global.tick_last_updated = tick;
            }
            global.model_matrix
        }
        else {
            let model_matrix = local.model_matrix * parent_global;
            let global = GlobalTransform {
                model_matrix,
                tick_last_updated: tick,
            };
            command_buffer.insert(entity, (global.clone(),));
            model_matrix
        }
    }

    let mut entities_query = system_context.world.query::<()>().with::<&Transform>();
    let mut transform_view = system_context.world.view::<TransformView>();
    let hierarchy_view = system_context.world.view::<HierarchyView>();

    for (entity, _) in entities_query.iter() {
        local_to_global(
            entity,
            &mut transform_view,
            &hierarchy_view,
            system_context.tick,
            &mut system_context.command_buffer,
        );
    }
}
