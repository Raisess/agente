use std::sync::Arc;

use agente_domain::error::Error;
use agente_domain::models::session::Session;
use agente_infrastructure::config::Config;

use crate::repositories::session::SessionRepository;

pub async fn init_session(
    session_repository: Arc<SessionRepository>,
    id: Option<String>,
) -> Result<Session, Error> {
    let session = match id {
        Some(session_id) => session_repository.find_by_id(session_id).await?,
        None => {
            let session = Session::new(Config::pwd());
            session_repository.create(&session).await?;

            Some(session)
        }
    };

    match session {
        Some(session) => Ok(session),
        None => panic!("Invalid session id!"),
    }
}
