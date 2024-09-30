use nalgebra::Similarity3;

pub struct Transform {
    pub matrix: Similarity3<f32>,
}

pub struct LocalTransform {
    pub matrix: Similarity3<f32>,
    //pub parent: EntityId,
}
