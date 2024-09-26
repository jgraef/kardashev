use nalgebra::Similarity3;

pub struct Transform {
    pub transform: Similarity3<f32>,
}

pub struct LocalTransform {
    pub transform: Similarity3<f32>,
    //pub parent: EntityId,
}
