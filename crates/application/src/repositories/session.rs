use std::sync::Arc;

use agente_domain::error::Error;
use agente_domain::models::session::Session;
use agente_domain::ports::database::Database;

const TABLE: &str = r#"
CREATE TABLE IF NOT EXISTS sessions(
    id UUID PRIMARY KEY NOT NULL,
    started_at TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL,
    username VARCHAR(100) NOT NULL,
    hostname VARCHAR(100) NOT NULL,
    directory VARCHAR(500) NOT NULL,
    summary_phrase VARCHAR(100) NULL
);

CREATE INDEX IF NOT EXISTS sessions_per_directory ON sessions(directory);
CREATE INDEX IF NOT EXISTS sessions_per_user ON sessions(username, directory);
"#;

pub struct SessionRepository {
    db: Arc<dyn Database<sqlx::Pool<sqlx::Sqlite>>>,
}

impl SessionRepository {
    pub async fn new(
        db: Arc<dyn Database<sqlx::Pool<sqlx::Sqlite>>>,
    ) -> Result<Self, Error> {
        sqlx::query(TABLE).execute(&*db.expose()).await?;
        Ok(Self { db })
    }

    pub async fn create(&self, session: &Session) -> Result<(), Error> {
        let sql = "INSERT INTO sessions(id, started_at, updated_at, username, hostname, \
                   directory) VALUES(?, ?, ?, ?, ?, ?);";

        sqlx::query(&sql)
            .bind(session.id)
            .bind(session.started_at)
            .bind(session.updated_at)
            .bind(session.username.clone())
            .bind(session.hostname.clone())
            .bind(session.directory.clone())
            .execute(&*self.db.expose())
            .await?;
        Ok(())
    }

    pub async fn change_summary_phrase(
        &self,
        id: uuid::Uuid,
        summary_phrase: String,
    ) -> Result<(), Error> {
        let sql = "UPDATE sessions SET summary_phrase = ?, updated_at = datetime('now') WHERE id = ?;";

        sqlx::query(&sql)
            .bind(summary_phrase)
            .bind(id)
            .execute(&*self.db.expose())
            .await?;
        Ok(())
    }

    pub async fn find_by_username_and_directory(
        &self,
        username: String,
        directory: String,
    ) -> Result<Vec<Session>, Error> {
        let sql = "SELECT * FROM sessions WHERE username = ? AND directory = ? ORDER BY updated_at ASC;";

        Ok(sqlx::query_as::<_, Session>(&sql)
            .bind(username)
            .bind(directory)
            .fetch_all(&*self.db.expose())
            .await?)
    }

    pub async fn find_by_id(&self, id: uuid::Uuid) -> Result<Option<Session>, Error> {
        let sql = "SELECT * FROM sessions WHERE id = ? LIMIT 1;";

        Ok(sqlx::query_as::<_, Session>(&sql)
            .bind(id)
            .fetch_optional(&*self.db.expose())
            .await?)
    }
}
