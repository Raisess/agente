use crate::error::Error;

/// Represents a SQL Database definition
#[async_trait::async_trait]
pub trait Database<T> {
    /// Executes a sql query and return the number of affected rows
    async fn query(&self, sql: &str) -> Result<u64, Error>;
    /// Executes a sql query and fetch the results converting to right struct
    // @TODO: make from row more generic to support different databases
    async fn fetch(&self, sql: &str) -> Result<Vec<T>, Error>
    where
        T: for<'r> sqlx::FromRow<'r, sqlx::sqlite::SqliteRow> + Send + Unpin;
}
