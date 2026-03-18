use std::sync::Arc;

use sqlx::sqlite::SqlitePool;

use agente_domain::error::Error;
use agente_domain::ports::database::Database;

use crate::config::Config;

pub struct SqliteDatabase {
    connection: Arc<sqlx::Pool<sqlx::Sqlite>>,
}

impl SqliteDatabase {
    pub async fn new(db_path: &str) -> Result<Self, Error> {
        let url = format!("sqlite://{}/{db_path}?mode=rwc", Config::pwd());
        Ok(Self {
            connection: Arc::new(SqlitePool::connect(&url).await?),
        })
    }
}

impl Database<sqlx::Pool<sqlx::Sqlite>> for SqliteDatabase {
    fn expose(&self) -> Arc<sqlx::Pool<sqlx::Sqlite>> {
        self.connection.clone()
    }
}
