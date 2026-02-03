use thiserror::Error;

/// This is the Agent interface, it can represent a AI agent implementation,
/// e.g.: ChatGPT, DeepSeek, etc.
#[async_trait::async_trait]
pub trait Agent: Send + Sync {
    /// Ask the agent with important info like file contents, summaries, etc
    /// and update the previous message context.
    /// @NOTE: Use send the base prompt to feed the agent instructions set.
    async fn ask(
        &self,
        messages: Vec<MessageRequest>,
    ) -> Result<AskResponse, AgentError>;
}

#[derive(Clone, Debug, Error)]
pub enum AgentError {
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

#[derive(Debug, Default)]
pub struct AskResponse {
    pub content: String,
}

#[derive(Debug, Clone)]
pub struct MessageRequest {
    pub role: MessageRole,
    pub content: String,
}

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
