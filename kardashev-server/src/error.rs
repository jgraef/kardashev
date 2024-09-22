#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("axum error")]
    Axum(#[from] axum::Error),

    #[error("sqlx error")]
    Sqlx(#[from] sqlx::Error),

    #[error("io error")]
    Io(#[from] std::io::Error),

    #[error("sqlx migrate error")]
    SqlxMigrate(#[from] sqlx::migrate::MigrateError),
}
