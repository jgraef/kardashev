use std::ops::{
    Deref,
    DerefMut,
};

use chrono::{
    DateTime,
    Utc,
};
use sqlx::{
    PgPool,
    Postgres,
};
use tokio_util::sync::CancellationToken;

use crate::error::Error;

#[derive(Clone)]
pub struct Context {
    pub shutdown: CancellationToken,
    pub up_since: DateTime<Utc>,
    db: PgPool,
}

impl Context {
    pub fn new(db: PgPool) -> Self {
        Self {
            shutdown: CancellationToken::new(),
            up_since: Utc::now(),
            db,
        }
    }

    pub async fn transaction<'a>(&'a self) -> Result<Transaction<'a>, Error> {
        let transaction = self.db.begin().await?;

        Ok(Transaction { transaction })
    }
}

pub struct Transaction<'a> {
    transaction: sqlx::Transaction<'a, Postgres>,
}

impl<'a> Deref for Transaction<'a> {
    type Target = sqlx::Transaction<'a, Postgres>;

    fn deref(&self) -> &Self::Target {
        &self.transaction
    }
}

impl<'a> DerefMut for Transaction<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.transaction
    }
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
