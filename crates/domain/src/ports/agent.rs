use thiserror::Error;

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
    /// Send a prompt to the AI agente and wait for the result.
    async fn ask(&self, prompt: &str) -> Result<String, AgentError>;
}
