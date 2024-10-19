#[derive(Debug, thiserror::Error)]
#[error("app error")]
pub enum Error {
    Graphics(#[from] crate::graphics::Error),
    Client(#[from] kardashev_client::Error),
    Schedule(#[from] crate::ecs::Error),
}
