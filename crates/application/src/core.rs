pub mod context;
pub mod execution_plan;
pub mod processor;

use std::str::FromStr;
use std::sync::Arc;

use agente_domain::error::Error;
use agente_domain::models::message::Message;
use agente_domain::models::session::Session;
use agente_domain::ports::ai_provider::MessageRole;
use agente_infrastructure::config::Config;

use crate::repositories::conversation::ConversationRepository;
use crate::repositories::session::SessionRepository;

pub async fn init_session(
    session_repository: Arc<SessionRepository>,
    id: Option<String>,
) -> Result<Session, Error> {
    let session = match id {
        Some(session_id) => {
            session_repository
                .find_by_id(uuid::Uuid::from_str(&session_id)?)
                .await?
        }
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

pub async fn append_to_conversation(
    conversation_repository: Arc<ConversationRepository>,
    session_id: String,
    role: MessageRole,
    content: String,
) -> Result<(), Error> {
    let message = Message::new(
        uuid::Uuid::from_str(&session_id)?,
        role.to_string(),
        content,
    );

    conversation_repository.append(&message).await?;
    Ok(())
}

pub async fn get_conversation(
    conversation_repository: Arc<ConversationRepository>,
    session_id: String,
) -> Result<Vec<Message>, Error> {
    let id = uuid::Uuid::from_str(&session_id)?;
    conversation_repository.list(id).await
}
