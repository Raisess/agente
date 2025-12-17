use futures::stream::Stream;
use thiserror::Error;

#[derive(Clone, Debug, Error)]
pub enum AgentError {
    #[error("other error")]
    Other,
}

/// This is the Agent interface, it can represent a AI agent implementation,
/// e.g.: ChatGPT, DeepSeek, etc.
#[async_trait::async_trait]
pub trait Agent {
    /// Send a prompt to the AI agente and wait for the result.
    async fn ask(
        &self,
        prompt: &str,
    ) -> Result<Box<dyn Stream<Item = String>>, AgentError>;
}
