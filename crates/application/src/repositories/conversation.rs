use std::sync::Arc;

use agente_domain::error::Error;
use agente_domain::models::message::Message;
use agente_domain::ports::database::Database;
use uuid::Uuid;

const TABLE: &str = r#"
CREATE TABLE IF NOT EXISTS conversations(
    id UUID PRIMARY KEY NOT NULL,
    session_id UUID NOT NULL,
    sent_at TIMESTAMPTZ NOT NULL,
    role VARCHAR(20) NOT NULL,
    content TEXT NOT NULL,
    is_summarized BOOL NOT NULL,

    CONSTRAINT fk_session
        FOREIGN KEY (session_id)
        REFERENCES sessions(id)
        ON DELETE CASCADE
);"#;

pub struct ConversationRepository {
    db: Arc<dyn Database<sqlx::Pool<sqlx::Sqlite>>>,
}

impl ConversationRepository {
    pub async fn new(
        db: Arc<dyn Database<sqlx::Pool<sqlx::Sqlite>>>,
    ) -> Result<Self, Error> {
        sqlx::query(TABLE).execute(&*db.expose()).await?;
        Ok(Self { db })
    }

    pub async fn append(&self, message: &Message) -> Result<(), Error> {
        let sql = "INSERT INTO conversations(id, session_id, sent_at, role, content, is_summarized) \
                   VALUES(?, ?, ?, ?, ?, ?);";

        sqlx::query(&sql)
            .bind(message.id)
            .bind(message.session_id)
            .bind(message.sent_at)
            .bind(message.role.clone())
            .bind(message.content.clone())
            .bind(message.is_summarized)
            .execute(&*self.db.expose())
            .await?;
        Ok(())
    }

    pub async fn list(&self, session_id: Uuid) -> Result<Vec<Message>, Error> {
        // @TODO: increase limit and chunknize results
        let sql = "WITH c AS (SELECT * FROM conversations WHERE session_id = ? AND is_summarized = true ORDER BY \
                   sent_at DESC LIMIT 50) SELECT * FROM c ORDER BY sent_at ASC;";

        Ok(sqlx::query_as::<_, Message>(&sql)
            .bind(session_id)
            .fetch_all(&*self.db.expose())
            .await?)
    }
}
