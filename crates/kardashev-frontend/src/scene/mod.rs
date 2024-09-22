pub mod camera;
pub mod mesh;
pub mod renderer;
pub mod texture;
pub mod transform;
pub mod window;

use std::sync::{
    Arc,
    Mutex,
};

use hecs::World;

#[derive(Clone)]
pub struct Scene {
    world: Arc<Mutex<World>>,
}

impl Scene {
    pub fn new(world: World) -> Self {
        Self {
            world: Arc::new(Mutex::new(world)),
        }
    }
}
