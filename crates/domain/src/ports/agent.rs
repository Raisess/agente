use thiserror::Error;

use crate::core::models::task::Task;

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

#[derive(Debug)]
pub struct MessageRequest {
    pub previous_message_id: Option<String>,
    pub prompt: String,
}

#[derive(Debug, Default)]
pub struct FeedResponse {
    pub message_id: Option<String>,
    pub content: String,
}

/// This is the Agent interface, it can represent a AI agent implementation,
/// e.g.: ChatGPT, DeepSeek, etc.
#[async_trait::async_trait]
pub trait Agent {
    /// Feed the agent with important info like file contents, summaries, etc
    /// and update the previous message context.
    /// @NOTE: Use send the base prompt to feed the agent instructions set.
    async fn feed(
        &mut self,
        message: MessageRequest,
    ) -> Result<FeedResponse, AgentError>;
    /// Send a prompt to the AI agente and wait for the result.
    /// @NOTE: For each ask iteration the usage should be updated.
    async fn ask(
        &mut self,
        message: MessageRequest,
    ) -> Result<Vec<Task>, AgentError>;
}
