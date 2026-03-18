use sqlx::sqlite::SqlitePool;

use agente_domain::error::Error;
use agente_domain::ports::database::Database;

pub struct SqliteDatabase {
    connection: sqlx::Pool<sqlx::Sqlite>,
}

impl SqliteDatabase {
    pub async fn new(db_path: &str) -> Result<Self, Error> {
        Ok(Self {
            connection: SqlitePool::connect(&format!("sqlite://{db_path}"))
                .await?,
        })
    }
}

#[async_trait::async_trait]
impl<T> Database<T> for SqliteDatabase {
    async fn query(&self, sql: &str) -> Result<u64, Error> {
        let result = sqlx::query(sql.trim()).execute(&self.connection).await?;
        Ok(result.rows_affected())
    }

    async fn fetch(&self, sql: &str) -> Result<Vec<T>, Error>
    where
        T: for<'r> sqlx::FromRow<'r, sqlx::sqlite::SqliteRow> + Send + Unpin,
    {
        Ok(sqlx::query_as::<_, T>(sql.trim())
            .fetch_all(&self.connection)
            .await?)
    }
}
