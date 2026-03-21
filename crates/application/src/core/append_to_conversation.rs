use std::str::FromStr;
use std::sync::Arc;

use agente_domain::error::Error;
use agente_domain::models::message::Message;
use agente_domain::ports::ai_provider::MessageRole;

use crate::repositories::conversation::ConversationRepository;

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
