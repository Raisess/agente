use std::str::FromStr;
use std::sync::Arc;

use agente_domain::error::Error;
use agente_domain::models::message::Message;

use crate::repositories::conversation::ConversationRepository;

pub async fn get_conversation(
    conversation_repository: Arc<ConversationRepository>,
    session_id: String,
) -> Result<Vec<Message>, Error> {
    let id = uuid::Uuid::from_str(&session_id)?;
    conversation_repository.list(id).await
}
