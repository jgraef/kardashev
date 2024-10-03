#[derive(Debug, thiserror::Error)]
#[error("server error")]
pub enum Error {
    Axum(#[from] axum::Error),
    Sqlx(#[from] sqlx::Error),
    Io(#[from] std::io::Error),
    SqlxMigrate(#[from] sqlx::migrate::MigrateError),
    NotifyAsync(#[from] notify_async::Error),
}
