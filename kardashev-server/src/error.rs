#[derive(Debug, thiserror::Error)]
#[error("server error")]
pub enum Error {
    Axum(#[from] axum::Error),
    Sqlx(#[from] sqlx::Error),
    Io(#[from] std::io::Error),
    SqlxMigrate(#[from] sqlx::migrate::MigrateError),
    Assets(#[from] kardashev_assets::Error),
    Trunk(#[from] crate::util::ui::TrunkError),
}
