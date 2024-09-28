#[derive(Debug, thiserror::Error)]
#[error("app error")]
pub enum Error {
    Graphics(#[from] crate::graphics::Error),
}
