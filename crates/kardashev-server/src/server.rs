use std::net::SocketAddr;

use axum::Router;
use sqlx::{
    PgPool,
    Postgres,
};
use tokio::net::TcpListener;

use crate::error::Error;

pub struct Server {
    router: Router,
}

impl Server {
    pub async fn new(db: PgPool) -> Result<Self, Error> {
        sqlx::migrate!("../../migrations").run(&db).await?;

        let router = Router::new()
            .nest("api", crate::api::router())
            .with_state(Context { db });

        Ok(Self { router })
    }

    pub async fn bind(self, address: SocketAddr) -> Result<(), Error> {
        let listener = TcpListener::bind(address).await?;
        axum::serve(listener, self.router).await?;
        Ok(())
    }
}

#[derive(Clone)]
pub struct Context {
    db: PgPool,
}

impl Context {
    pub async fn transaction<'a>(&'a self) -> Result<Transaction<'a>, Error> {
        let transaction = self.db.begin().await?;

        Ok(Transaction { transaction })
    }
}

pub struct Transaction<'a> {
    transaction: sqlx::Transaction<'a, Postgres>,
}

impl<'a> Transaction<'a> {
    pub async fn commit(self) -> Result<(), Error> {
        self.transaction.commit().await?;
        Ok(())
    }

    pub async fn rollback(self) -> Result<(), Error> {
        self.transaction.rollback().await?;
        Ok(())
    }
}
