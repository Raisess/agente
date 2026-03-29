use std::collections::HashMap;

use thiserror::Error;

/// This is the Agent interface, it can represent a AI agent implementation,
/// e.g.: ChatGPT, DeepSeek, etc.
#[async_trait::async_trait]
pub trait AiProvider: Send + Sync {
    /// Ask the agent with important info like file contents, summaries, etc
    /// and update the previous message context.
    async fn ask(
        &self,
        messages: Vec<MessageRequest>,
    ) -> Result<AskResponse, AiProviderError>;
}

#[derive(Clone, Debug, Error)]
pub enum AiProviderError {
    #[error("Other error: {0}")]
    Other(String),
    #[error("The json response is not valid: {0}")]
    FailedToParseResponse(String),
    #[error(
        "You was rate limited or run out of credits, please reload you model \
         agent and retry."
    )]
    Limited,
    #[error(
        "The server of the API provider is currently overloaded, try again \
         later."
    )]
    ServicesOverloaded,
}

/// Represents the response for the ask method, it can be plain text (Content)
/// or tool call invocation (ToolCall)
#[derive(Debug)]
pub enum AskResponse {
    Content(String),
    /// tool name, arguments
    ToolCall(Vec<(String, HashMap<String, String>)>),
}

/// Used to mount the message request vector
#[derive(Debug, Clone)]
pub struct MessageRequest {
    pub role: MessageRole,
    pub content: String,
}

/// Message roles enumeration
#[derive(Debug, Clone)]
pub enum MessageRole {
    Assistant,
    System,
    User,
}

impl std::fmt::Display for MessageRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            MessageRole::Assistant => "assistant",
            MessageRole::System => "system",
            MessageRole::User => "user",
        })
    }
}

impl From<String> for MessageRole {
    fn from(value: String) -> Self {
        match value.as_str() {
            "assistant" => MessageRole::Assistant,
            "system" => MessageRole::System,
            "user" => MessageRole::User,
            _ => panic!("Invalid role type"),
        }
    }
}
