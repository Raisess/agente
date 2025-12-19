use thiserror::Error;

use crate::core::models::task::Task;

#[derive(Clone, Debug, Error)]
pub enum AgentError {
    #[error("Other error: {0}")]
    Other(String),
    #[error("The json response is not valid: {0}")]
    FailedToParseResponse(String),
    #[error("You run out of credits, please reload you model agent and retry.")]
    OutOfCredits,
    #[error(
        "The server of the API provider is currently overloaded, try again \
         later."
    )]
    ServicesOverloaded,
}

/// This is the Agent interface, it can represent a AI agent implementation,
/// e.g.: ChatGPT, DeepSeek, etc.
#[async_trait::async_trait]
pub trait Agent {
    /// Send the base prompt to feed the agent instructions set.
    async fn prepare(&self, base_prompt: &str) -> Result<(), AgentError>;
    /// Send a prompt to the AI agente and wait for the result,
    /// for each ask iteration the usage should be updated.
    async fn ask(&mut self, prompt: &str) -> Result<Vec<Task>, AgentError>;
}
