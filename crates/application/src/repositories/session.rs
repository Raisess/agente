use agente_domain::error::Error;
use agente_domain::models::session::Session;
use agente_domain::ports::database::Database;

pub struct SessionRepository {
    db: Box<dyn Database<Session>>,
}

impl SessionRepository {
    pub async fn setup(&self) -> Result<(), Error> {
        let sql = "CREATE TABLE IF NOT EXISTS sessions(id UUID PRIMARY KEY \
                   NOT NULL, started_at TIMESTAMPTZ NOT NULL, updated_at \
                   TIMESTAMPTZ NOT NULL, username VARCHAR(100) NOT NULL, \
                   hostname VARCHAR(100) NOT NULL, directory VARCHAR(500) NOT \
                   NULL);";

        self.db.query(&sql).await?;
        Ok(())
    }

    pub async fn create(&self, session: &Session) -> Result<(), Error> {
        let sql = format!(
            "INSERT INTO sessions(id, started_at, updated_at, username, \
             hostname, directory) VALUES('{}', '{}', '{}', '{}', '{}', '{}');",
            session.id,
            session.started_at,
            session.updated_at,
            session.username,
            session.hostname,
            session.directory
        );

        self.db.query(&sql).await?;
        Ok(())
    }

    pub async fn find_by_id(
        &self,
        id: uuid::Uuid,
    ) -> Result<Option<Session>, Error> {
        let sql = format!(
            "SELECT * FROM sessions WHERE id = '{}' LIMIT 1;",
            id.to_string()
        );
        let results = self.db.fetch(&sql).await?;
        Ok(results.get(0).cloned())
    }
}
