use core::f32;
use std::marker::PhantomData;

use hecs::{
    Entity,
    ViewBorrow,
    World,
};
use nalgebra::{
    Isometry3,
    Vector3,
};
use parry3d::{
    partitioning::{
        IndexedData,
        Qbvh,
        QbvhUpdateWorkspace,
    },
    query::{
        details::{
            NormalConstraints,
            RayCompositeShapeToiBestFirstVisitor,
        },
        Ray,
    },
    shape::{
        Shape,
        TypedSimdCompositeShape,
    },
};

use crate::{
    collide::collider::{
        Collider,
        ScaleUniformly,
    },
    ecs::resource::Resources,
    graphics::transform::GlobalTransform,
};

// todo: impl Debug
pub(super) struct QueryPipeline<Tag> {
    qbvh: Qbvh<Handle>,
    workspace: QbvhUpdateWorkspace,
    _tag: PhantomData<fn() -> Tag>,
}

impl<Tag> Default for QueryPipeline<Tag> {
    fn default() -> Self {
        Self {
            qbvh: Qbvh::new(),
            workspace: QbvhUpdateWorkspace::default(),
            _tag: PhantomData,
        }
    }
}

impl<T: Send + Sync + 'static> QueryPipeline<T> {
    /// Synchronizes the QBVH with the ECS world
    pub fn sync<S: Shape + Send + Sync + 'static>(&mut self, world: &mut World) {
        // note: using the crate decorum for Eq on floats might be a good idea :/
        //
        // todo: a better system for tracking changes would be nice.

        let query_changed = world.query_mut::<(
            &mut SyncedData<S, T>,
            Option<&Collider<S, T>>,
            Option<&GlobalTransform>,
        )>();

        for (entity, (synced, collider, transform)) in query_changed {
            if let (Some(collider), Some(transform)) = (collider, transform) {
                // check for changes by comparing the collider and transform stored in the
                // SyncedData component

                /*let mut changed = false;

                if !synced.collider.eq(&collider) {
                    synced.collider = collider.clone();
                    changed = true;
                }

                if !synced.transform.model_matrix.eq(&transform.model_matrix) {
                    synced.transform = transform.clone();
                }

                if changed {
                    self.qbvh.pre_update_or_insert(Handle(entity));
                }*/
                todo!();
            }
            else {
                // either the collider or global transform was removed, so we remove this from
                // the qbvh

                self.qbvh.remove(Handle(entity));
            }
        }

        let query_added = world
            .query_mut::<(&Collider<S, T>, &GlobalTransform)>()
            .without::<&SyncedData<S, T>>();

        for (entity, (collider, transform)) in query_added {
            // entity with collider and transform that is not yet synced

            self.qbvh.pre_update_or_insert(Handle(entity));

            // self.command_buffer.insert_one(
            //     entity,
            //     SyncedData {
            //         collider: collider.clone(),
            //         transform: transform.clone(),
            //     },
            // );
        }

        let view = world.view_mut::<(&Collider<S, T>, &GlobalTransform)>();

        self.qbvh.refit(0.0, &mut self.workspace, |Handle(entity)| {
            let (collider, transform) = view
                .get(*entity)
                .unwrap_or_else(|| panic!("entity with collider not found: {entity:?}"));
            collider
                .shape()
                .compute_aabb(&transform.model_matrix.isometry)
                .scaled(&Vector3::repeat(transform.model_matrix.scaling()))
        });
        self.qbvh.rebalance(0.0, &mut self.workspace);
    }

    fn as_composite_shape<'world, S: Send + Sync + 'static>(
        &self,
        world: &'world hecs::World,
    ) -> QueryPipelineAsCompositeShape<'_, 'world, '_, S, T> {
        let view = world.view::<(&Collider<S, T>, &GlobalTransform)>();
        QueryPipelineAsCompositeShape {
            query_pipeline: self,
            view,
        }
    }

    pub fn cast_ray<S: Shape + ScaleUniformly + Send + Sync + 'static>(
        &self,
        world: &hecs::World,
        ray: &Ray,
        max_time_of_impact: Option<f32>,
        starts_in_solid: bool,
    ) -> Option<(Entity, f32)> {
        let pipeline_shape = self.as_composite_shape::<S>(world);
        let mut visitor = RayCompositeShapeToiBestFirstVisitor::new(
            &pipeline_shape,
            ray,
            max_time_of_impact.unwrap_or(f32::MAX),
            starts_in_solid,
        );

        self.qbvh
            .traverse_best_first(&mut visitor)
            .map(|(_, (handle, distance))| (handle.0, distance))
    }
}

#[derive(Copy, Clone, Debug)]
struct Handle(Entity);

impl IndexedData for Handle {
    fn default() -> Self {
        Self(Entity::DANGLING)
    }

    fn index(&self) -> usize {
        self.0.id().try_into().unwrap()
    }
}

#[derive(Debug)]
struct SyncedData<Shape, Tag> {
    collider: Collider<Shape, Tag>,
    transform: GlobalTransform,
}

struct QueryPipelineAsCompositeShape<
    'pipeline,
    'view,
    'query,
    Shape: Send + Sync + 'static,
    Tag: Send + Sync + 'static,
> {
    query_pipeline: &'pipeline QueryPipeline<Tag>,
    view: ViewBorrow<'view, (&'query Collider<Shape, Tag>, &'query GlobalTransform)>,
}

impl<
        'pipeline,
        'view,
        'query,
        S: Shape + ScaleUniformly + Send + Sync + 'static,
        T: Send + Sync + 'static,
    > TypedSimdCompositeShape for QueryPipelineAsCompositeShape<'pipeline, 'view, 'query, S, T>
{
    type PartShape = dyn parry3d::shape::Shape;
    type PartNormalConstraints = dyn NormalConstraints;
    type PartId = Handle;

    fn map_typed_part_at(
        &self,
        shape_id: Self::PartId,
        mut f: impl FnMut(
            Option<&Isometry3<f32>>,
            &Self::PartShape,
            Option<&Self::PartNormalConstraints>,
        ),
    ) {
        if let Some((collider, transform)) = self.view.get(shape_id.0) {
            // todo: we could filter here
            f(
                Some(&transform.model_matrix.isometry),
                &collider.get_scaled_shape(transform.model_matrix.scaling()),
                None,
            )
        }
    }

    fn map_untyped_part_at(
        &self,
        shape_id: Self::PartId,
        f: impl FnMut(Option<&Isometry3<f32>>, &Self::PartShape, Option<&dyn NormalConstraints>),
    ) {
        self.map_typed_part_at(shape_id, f);
    }

    fn typed_qbvh(&self) -> &Qbvh<Handle> {
        &self.query_pipeline.qbvh
    }
}

pub(super) fn get_pipeline<'a, T: 'static>(
    resources: &'a mut Resources,
) -> &'a mut QueryPipeline<T> {
    resources.get_mut_or_insert_default()
}
