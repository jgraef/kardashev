use std::marker::PhantomData;

use nalgebra::Vector3;

use crate::ecs::Dirty;

#[derive(Clone, Debug)]
pub struct Collider<Shape = AnyShape, Tag = ()> {
    shape: Dirty<Shape>,
    _tag: PhantomData<fn() -> Tag>,
}

impl<Shape, Tag> Collider<Shape, Tag> {
    pub fn shape(&self) -> &Shape {
        self.shape.as_ref()
    }

    pub fn shape_mut(&mut self) -> &mut Shape {
        self.shape.as_mut()
    }
}

impl<Shape: ScaleUniformly, Tag> Collider<Shape, Tag> {
    pub(super) fn get_scaled_shape(&self, scale: f32) -> Shape {
        self.shape.as_ref().scale_uniformly(scale)
    }
}

#[derive(Clone, Debug)]
pub enum AnyShape {
    Ball(parry3d::shape::Ball),
    Cuboid(parry3d::shape::Cuboid),
    Capsule(parry3d::shape::Capsule),
    Segment(parry3d::shape::Segment),
    Triangle(parry3d::shape::Triangle),
    TriMesh(parry3d::shape::TriMesh),
    Polyline(parry3d::shape::Polyline),
    HalfSpace(parry3d::shape::HalfSpace),
    HeightField(parry3d::shape::HeightField),
    // note: we would need to define our own Compound type that contains `dyn Shape` (our trait)
    //Compound(parry3d::shape::Compound),
    ConvexPolyhedron(parry3d::shape::ConvexPolyhedron),
    Cylinder(parry3d::shape::Cylinder),
    Cone(parry3d::shape::Cone),
    RoundCuboid(parry3d::shape::RoundCuboid),
    RoundTriangle(parry3d::shape::RoundTriangle),
    RoundCylinder(parry3d::shape::RoundCylinder),
    RoundCone(parry3d::shape::RoundCone),
    RoundConvexPolyhedron(parry3d::shape::RoundConvexPolyhedron),
}

macro_rules! any_shape_dispatch {
    ($trait:path, $self:expr, $method:ident ($($arg:expr),*)) => {
        match $self {
            AnyShape::Ball(inner) => <_ as $trait>::$method(inner, $($arg,)*),
            AnyShape::Cuboid(inner) => <_ as $trait>::$method(inner, $($arg,)*),
            AnyShape::Capsule(inner) => <_ as $trait>::$method(inner, $($arg,)*),
            AnyShape::Segment(inner) => <_ as $trait>::$method(inner, $($arg,)*),
            AnyShape::Triangle(inner) => <_ as $trait>::$method(inner, $($arg,)*),
            AnyShape::TriMesh(inner) => <_ as $trait>::$method(inner, $($arg,)*),
            AnyShape::Polyline(inner) => <_ as $trait>::$method(inner, $($arg,)*),
            AnyShape::HalfSpace(inner) => <_ as $trait>::$method(inner, $($arg,)*),
            AnyShape::HeightField(inner) => <_ as $trait>::$method(inner, $($arg,)*),
            //AnyShape::Compound(inner) => <_ as $trait>::$method(inner, $($arg,)*),
            AnyShape::ConvexPolyhedron(inner) => <_ as $trait>::$method(inner, $($arg,)*),
            AnyShape::Cylinder(inner) => <_ as $trait>::$method(inner, $($arg,)*),
            AnyShape::Cone(inner) => <_ as $trait>::$method(inner, $($arg,)*),
            AnyShape::RoundCuboid(inner) => <_ as $trait>::$method(inner, $($arg,)*),
            AnyShape::RoundTriangle(inner) => <_ as $trait>::$method(inner, $($arg,)*),
            AnyShape::RoundCylinder(inner) => <_ as $trait>::$method(inner, $($arg,)*),
            AnyShape::RoundCone(inner) => <_ as $trait>::$method(inner, $($arg,)*),
            AnyShape::RoundConvexPolyhedron(inner) => <_ as $trait>::$method(inner, $($arg,)*),
        }
    };
}

impl parry3d::shape::Shape for AnyShape {
    fn compute_local_aabb(&self) -> parry3d::bounding_volume::Aabb {
        any_shape_dispatch!(parry3d::shape::Shape, self, compute_local_aabb())
    }

    fn compute_local_bounding_sphere(&self) -> parry3d::bounding_volume::BoundingSphere {
        any_shape_dispatch!(parry3d::shape::Shape, self, compute_local_bounding_sphere())
    }

    fn clone_dyn(&self) -> Box<dyn parry3d::shape::Shape> {
        any_shape_dispatch!(parry3d::shape::Shape, self, clone_dyn())
    }

    fn scale_dyn(
        &self,
        scale: &parry3d::math::Vector<f32>,
        num_subdivisions: u32,
    ) -> Option<Box<dyn parry3d::shape::Shape>> {
        any_shape_dispatch!(
            parry3d::shape::Shape,
            self,
            scale_dyn(scale, num_subdivisions)
        )
    }

    fn mass_properties(&self, density: f32) -> parry3d::mass_properties::MassProperties {
        any_shape_dispatch!(parry3d::shape::Shape, self, mass_properties(density))
    }

    fn shape_type(&self) -> parry3d::shape::ShapeType {
        any_shape_dispatch!(parry3d::shape::Shape, self, shape_type())
    }

    fn as_typed_shape(&self) -> parry3d::shape::TypedShape {
        any_shape_dispatch!(parry3d::shape::Shape, self, as_typed_shape())
    }

    fn ccd_thickness(&self) -> f32 {
        any_shape_dispatch!(parry3d::shape::Shape, self, ccd_thickness())
    }

    fn ccd_angular_thickness(&self) -> f32 {
        any_shape_dispatch!(parry3d::shape::Shape, self, ccd_angular_thickness())
    }

    fn compute_aabb(
        &self,
        position: &parry3d::math::Isometry<f32>,
    ) -> parry3d::bounding_volume::Aabb {
        any_shape_dispatch!(parry3d::shape::Shape, self, compute_aabb(position))
    }

    fn compute_bounding_sphere(
        &self,
        position: &parry3d::math::Isometry<f32>,
    ) -> parry3d::bounding_volume::BoundingSphere {
        any_shape_dispatch!(
            parry3d::shape::Shape,
            self,
            compute_bounding_sphere(position)
        )
    }

    fn is_convex(&self) -> bool {
        any_shape_dispatch!(parry3d::shape::Shape, self, is_convex())
    }

    fn as_support_map(&self) -> Option<&dyn parry3d::shape::SupportMap> {
        any_shape_dispatch!(parry3d::shape::Shape, self, as_support_map())
    }

    fn as_composite_shape(&self) -> Option<&dyn parry3d::shape::SimdCompositeShape> {
        any_shape_dispatch!(parry3d::shape::Shape, self, as_composite_shape())
    }

    fn as_polygonal_feature_map(&self) -> Option<(&dyn parry3d::shape::PolygonalFeatureMap, f32)> {
        any_shape_dispatch!(parry3d::shape::Shape, self, as_polygonal_feature_map())
    }

    fn feature_normal_at_point(
        &self,
        feature: parry3d::shape::FeatureId,
        point: &parry3d::math::Point<f32>,
    ) -> Option<nalgebra::Unit<parry3d::math::Vector<f32>>> {
        any_shape_dispatch!(
            parry3d::shape::Shape,
            self,
            feature_normal_at_point(feature, point)
        )
    }

    fn compute_swept_aabb(
        &self,
        start_pos: &parry3d::math::Isometry<f32>,
        end_pos: &parry3d::math::Isometry<f32>,
    ) -> parry3d::bounding_volume::Aabb {
        any_shape_dispatch!(
            parry3d::shape::Shape,
            self,
            compute_swept_aabb(start_pos, end_pos)
        )
    }
}

impl parry3d::query::PointQuery for AnyShape {
    fn project_local_point(
        &self,
        pt: &parry3d::math::Point<f32>,
        solid: bool,
    ) -> parry3d::query::PointProjection {
        any_shape_dispatch!(
            parry3d::query::PointQuery,
            self,
            project_local_point(pt, solid)
        )
    }

    fn project_local_point_and_get_feature(
        &self,
        pt: &parry3d::math::Point<f32>,
    ) -> (parry3d::query::PointProjection, parry3d::shape::FeatureId) {
        any_shape_dispatch!(
            parry3d::query::PointQuery,
            self,
            project_local_point_and_get_feature(pt)
        )
    }

    fn project_local_point_with_max_dist(
        &self,
        pt: &parry3d::math::Point<f32>,
        solid: bool,
        max_dist: f32,
    ) -> Option<parry3d::query::PointProjection> {
        any_shape_dispatch!(
            parry3d::query::PointQuery,
            self,
            project_local_point_with_max_dist(pt, solid, max_dist)
        )
    }

    fn project_point_with_max_dist(
        &self,
        m: &parry3d::math::Isometry<f32>,
        pt: &parry3d::math::Point<f32>,
        solid: bool,
        max_dist: f32,
    ) -> Option<parry3d::query::PointProjection> {
        any_shape_dispatch!(
            parry3d::query::PointQuery,
            self,
            project_point_with_max_dist(m, pt, solid, max_dist)
        )
    }

    fn distance_to_local_point(&self, pt: &parry3d::math::Point<f32>, solid: bool) -> f32 {
        any_shape_dispatch!(
            parry3d::query::PointQuery,
            self,
            distance_to_local_point(pt, solid)
        )
    }

    fn contains_local_point(&self, pt: &parry3d::math::Point<f32>) -> bool {
        any_shape_dispatch!(parry3d::query::PointQuery, self, contains_local_point(pt))
    }

    fn project_point(
        &self,
        m: &parry3d::math::Isometry<f32>,
        pt: &parry3d::math::Point<f32>,
        solid: bool,
    ) -> parry3d::query::PointProjection {
        any_shape_dispatch!(
            parry3d::query::PointQuery,
            self,
            project_point(m, pt, solid)
        )
    }

    fn distance_to_point(
        &self,
        m: &parry3d::math::Isometry<f32>,
        pt: &parry3d::math::Point<f32>,
        solid: bool,
    ) -> f32 {
        any_shape_dispatch!(
            parry3d::query::PointQuery,
            self,
            distance_to_point(m, pt, solid)
        )
    }

    fn project_point_and_get_feature(
        &self,
        m: &parry3d::math::Isometry<f32>,
        pt: &parry3d::math::Point<f32>,
    ) -> (parry3d::query::PointProjection, parry3d::shape::FeatureId) {
        any_shape_dispatch!(
            parry3d::query::PointQuery,
            self,
            project_point_and_get_feature(m, pt)
        )
    }

    fn contains_point(
        &self,
        m: &parry3d::math::Isometry<f32>,
        pt: &parry3d::math::Point<f32>,
    ) -> bool {
        any_shape_dispatch!(parry3d::query::PointQuery, self, contains_point(m, pt))
    }
}

impl parry3d::query::RayCast for AnyShape {
    fn cast_local_ray_and_get_normal(
        &self,
        ray: &parry3d::query::Ray,
        max_time_of_impact: f32,
        solid: bool,
    ) -> Option<parry3d::query::RayIntersection> {
        any_shape_dispatch!(
            parry3d::query::RayCast,
            self,
            cast_local_ray_and_get_normal(ray, max_time_of_impact, solid)
        )
    }

    fn cast_local_ray(
        &self,
        ray: &parry3d::query::Ray,
        max_time_of_impact: f32,
        solid: bool,
    ) -> Option<f32> {
        any_shape_dispatch!(
            parry3d::query::RayCast,
            self,
            cast_local_ray(ray, max_time_of_impact, solid)
        )
    }

    fn intersects_local_ray(&self, ray: &parry3d::query::Ray, max_time_of_impact: f32) -> bool {
        any_shape_dispatch!(
            parry3d::query::RayCast,
            self,
            intersects_local_ray(ray, max_time_of_impact)
        )
    }

    fn cast_ray(
        &self,
        m: &parry3d::math::Isometry<f32>,
        ray: &parry3d::query::Ray,
        max_time_of_impact: f32,
        solid: bool,
    ) -> Option<f32> {
        any_shape_dispatch!(
            parry3d::query::RayCast,
            self,
            cast_ray(m, ray, max_time_of_impact, solid)
        )
    }

    fn cast_ray_and_get_normal(
        &self,
        m: &parry3d::math::Isometry<f32>,
        ray: &parry3d::query::Ray,
        max_time_of_impact: f32,
        solid: bool,
    ) -> Option<parry3d::query::RayIntersection> {
        any_shape_dispatch!(
            parry3d::query::RayCast,
            self,
            cast_ray_and_get_normal(m, ray, max_time_of_impact, solid)
        )
    }

    fn intersects_ray(
        &self,
        m: &parry3d::math::Isometry<f32>,
        ray: &parry3d::query::Ray,
        max_time_of_impact: f32,
    ) -> bool {
        any_shape_dispatch!(
            parry3d::query::RayCast,
            self,
            intersects_ray(m, ray, max_time_of_impact)
        )
    }
}

impl ScaleUniformly for AnyShape {
    fn scale_uniformly(&self, scale: f32) -> Self {
        match self {
            AnyShape::Ball(inner) => {
                AnyShape::Ball(<_ as ScaleUniformly>::scale_uniformly(inner, scale))
            }
            AnyShape::Cuboid(inner) => {
                AnyShape::Cuboid(<_ as ScaleUniformly>::scale_uniformly(inner, scale))
            }
            AnyShape::Capsule(inner) => {
                AnyShape::Capsule(<_ as ScaleUniformly>::scale_uniformly(inner, scale))
            }
            AnyShape::Segment(inner) => {
                AnyShape::Segment(<_ as ScaleUniformly>::scale_uniformly(inner, scale))
            }
            AnyShape::Triangle(inner) => {
                AnyShape::Triangle(<_ as ScaleUniformly>::scale_uniformly(inner, scale))
            }
            AnyShape::TriMesh(inner) => {
                AnyShape::TriMesh(<_ as ScaleUniformly>::scale_uniformly(inner, scale))
            }
            AnyShape::Polyline(inner) => {
                AnyShape::Polyline(<_ as ScaleUniformly>::scale_uniformly(inner, scale))
            }
            AnyShape::HalfSpace(inner) => {
                AnyShape::HalfSpace(<_ as ScaleUniformly>::scale_uniformly(inner, scale))
            }
            AnyShape::HeightField(inner) => {
                AnyShape::HeightField(<_ as ScaleUniformly>::scale_uniformly(inner, scale))
            }
            AnyShape::ConvexPolyhedron(inner) => {
                AnyShape::ConvexPolyhedron(<_ as ScaleUniformly>::scale_uniformly(inner, scale))
            }
            AnyShape::Cylinder(inner) => {
                AnyShape::Cylinder(<_ as ScaleUniformly>::scale_uniformly(inner, scale))
            }
            AnyShape::Cone(inner) => {
                AnyShape::Cone(<_ as ScaleUniformly>::scale_uniformly(inner, scale))
            }
            AnyShape::RoundCuboid(inner) => {
                AnyShape::RoundCuboid(<_ as ScaleUniformly>::scale_uniformly(inner, scale))
            }
            AnyShape::RoundTriangle(inner) => {
                AnyShape::RoundTriangle(<_ as ScaleUniformly>::scale_uniformly(inner, scale))
            }
            AnyShape::RoundCylinder(inner) => {
                AnyShape::RoundCylinder(<_ as ScaleUniformly>::scale_uniformly(inner, scale))
            }
            AnyShape::RoundCone(inner) => {
                AnyShape::RoundCone(<_ as ScaleUniformly>::scale_uniformly(inner, scale))
            }
            AnyShape::RoundConvexPolyhedron(inner) => {
                AnyShape::RoundConvexPolyhedron(<_ as ScaleUniformly>::scale_uniformly(
                    inner, scale,
                ))
            }
        }
    }
}

pub trait ScaleUniformly {
    fn scale_uniformly(&self, scale: f32) -> Self;
}

/*
Ball(parry3d::shape::Ball),
    Cuboid(parry3d::shape::Cuboid),
    Capsule(parry3d::shape::Capsule),
    Segment(parry3d::shape::Segment),
    Triangle(parry3d::shape::Triangle),
    TriMesh(parry3d::shape::TriMesh),
    Polyline(parry3d::shape::Polyline),
    HalfSpace(parry3d::shape::HalfSpace),
    HeightField(parry3d::shape::HeightField),
    Compound(parry3d::shape::Compound),
    ConvexPolyhedron(parry3d::shape::ConvexPolyhedron),
    Cylinder(parry3d::shape::Cylinder),
    Cone(parry3d::shape::Cone),
    RoundCuboid(parry3d::shape::RoundCuboid),
    RoundTriangle(parry3d::shape::RoundTriangle),
    RoundCylinder(parry3d::shape::RoundCylinder),
    RoundCone(parry3d::shape::RoundCone),
    RoundConvexPolyhedron(parry3d::shape::RoundConvexPolyhedron),
*/

impl ScaleUniformly for parry3d::shape::Ball {
    fn scale_uniformly(&self, scale: f32) -> Self {
        Self {
            radius: scale * self.radius,
        }
    }
}

impl ScaleUniformly for parry3d::shape::Cuboid {
    fn scale_uniformly(&self, scale: f32) -> Self {
        Self {
            half_extents: scale * self.half_extents,
        }
    }
}

impl ScaleUniformly for parry3d::shape::Capsule {
    fn scale_uniformly(&self, scale: f32) -> Self {
        Self {
            segment: self.segment.scale_uniformly(scale),
            radius: scale * self.radius,
        }
    }
}

impl ScaleUniformly for parry3d::shape::Segment {
    fn scale_uniformly(&self, scale: f32) -> Self {
        self.scaled(&Vector3::repeat(scale))
    }
}

impl ScaleUniformly for parry3d::shape::Triangle {
    fn scale_uniformly(&self, scale: f32) -> Self {
        Self {
            a: self.a * scale,
            b: self.b * scale,
            c: self.c * scale,
        }
    }
}

impl ScaleUniformly for parry3d::shape::TriMesh {
    fn scale_uniformly(&self, scale: f32) -> Self {
        self.clone().scaled(&Vector3::repeat(scale))
    }
}

impl ScaleUniformly for parry3d::shape::Polyline {
    fn scale_uniformly(&self, scale: f32) -> Self {
        self.clone().scaled(&Vector3::repeat(scale))
    }
}

impl ScaleUniformly for parry3d::shape::HalfSpace {
    fn scale_uniformly(&self, scale: f32) -> Self {
        self.scaled(&Vector3::repeat(scale))
            .expect("Can't scale a half-space to zero-size")
    }
}

impl ScaleUniformly for parry3d::shape::HeightField {
    fn scale_uniformly(&self, scale: f32) -> Self {
        self.clone().scaled(&Vector3::repeat(scale))
    }
}

impl ScaleUniformly for parry3d::shape::ConvexPolyhedron {
    fn scale_uniformly(&self, scale: f32) -> Self {
        self.clone()
            .scaled(&Vector3::repeat(scale))
            .expect("Can't scale convex polygedron to zero-size")
    }
}

impl ScaleUniformly for parry3d::shape::Cylinder {
    fn scale_uniformly(&self, scale: f32) -> Self {
        Self {
            half_height: scale * self.half_height,
            radius: scale * self.radius,
        }
    }
}

impl ScaleUniformly for parry3d::shape::Cone {
    fn scale_uniformly(&self, scale: f32) -> Self {
        Self {
            half_height: scale * self.half_height,
            radius: scale * self.radius,
        }
    }
}

impl<S: ScaleUniformly> ScaleUniformly for parry3d::shape::RoundShape<S> {
    fn scale_uniformly(&self, scale: f32) -> Self {
        Self {
            inner_shape: self.inner_shape.scale_uniformly(scale),
            border_radius: scale * self.border_radius,
        }
    }
}
