use axum::Router;
use sqlx::PgPool;
use tokio_util::sync::CancellationToken;

use crate::context::Context;

mod api;
mod context;
mod error;
mod util;

pub use crate::error::Error;

#[derive(Clone, Debug, Default)]
pub struct Builder {
    shutdown: Option<CancellationToken>,
    db: Option<PgPool>,
}

impl Builder {
    pub fn with_shutdown(mut self, shutdown: CancellationToken) -> Self {
        self.shutdown = Some(shutdown);
        self
    }

    pub fn with_db(mut self, db: PgPool) -> Self {
        self.db = Some(db);
        self
    }

    pub async fn with_connect_db(self, database_url: &str) -> Result<Self, Error> {
        let db = PgPool::connect(database_url).await?;
        Ok(self.with_db(db))
    }

    pub fn build(self) -> Router<()> {
        let mut context = Context::new(self.db.expect("no database provided"));

        if let Some(shutdown) = self.shutdown {
            context.shutdown = shutdown;
        }

        crate::api::router().with_state(context)
    }
}
