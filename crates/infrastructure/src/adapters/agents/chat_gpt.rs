use serde::{Deserialize, Serialize};
use serde_json::Value;

use agente_domain::ports::agent::{
    Agent, AgentError, AskResponse, MessageRequest,
};

#[derive(Debug, Default, Clone, Deserialize, Serialize)]
pub struct ChatGPTConfig {
    #[serde(rename = "chat_gpt::api_key")]
    pub api_key: String,
    #[serde(rename = "chat_gpt::model")]
    pub model: String,
}

pub struct ChatGPT {
    config: ChatGPTConfig,
    client: reqwest::Client,
}

impl ChatGPT {
    pub fn new(config: ChatGPTConfig) -> Self {
        Self {
            config,
            client: reqwest::Client::new(),
        }
    }

    async fn handle_message_request(
        &self,
        messages: Vec<MessageRequest>,
    ) -> Result<String, AgentError> {
        let response = self
            .send_message(messages)
            .await
            .map_err(|error| AgentError::Other(error.to_string()))?;

        let status = response.status().as_u16();
        if let Some(error) = status_to_error(status) {
            return Err(error);
        }

        let data = response.json::<Response>().await.map_err(|error| {
            AgentError::FailedToParseResponse(error.to_string())
        })?;

        Ok(extract_response_text(data))
    }

    async fn send_message(
        &self,
        messages: Vec<MessageRequest>,
    ) -> Result<reqwest::Response, reqwest::Error> {
        let input = messages
            .iter()
            .map(|message| {
                serde_json::json!({
                  "role": message.role.to_string(),
                  "content": message.content
                })
            })
            .collect::<Vec<_>>();
        let json = serde_json::json!({
            "input": input,
            "model": self.config.model, // gpt-3.5-turbo
        });

        self.client
            .post("https://api.openai.com/v1/responses")
            .header("Content-Type", "application/json")
            .header("Authorization", format!("Bearer {}", self.config.api_key))
            .json(&json)
            .send()
            .await
    }
}

#[async_trait::async_trait]
impl Agent for ChatGPT {
    async fn ask(
        &self,
        messages: Vec<MessageRequest>,
    ) -> Result<AskResponse, AgentError> {
        let content = self.handle_message_request(messages).await?;
        Ok(AskResponse { content })
    }
}

fn extract_response_text(data: Response) -> String {
    data.output
        .get(0)
        .unwrap()
        .content
        .get(0)
        .unwrap()
        .text
        .clone()
}

fn status_to_error(status: u16) -> Option<AgentError> {
    match status {
        429 => Some(AgentError::Limited),
        503 => Some(AgentError::ServicesOverloaded),
        _ => None,
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Response {
    pub id: String,
    pub model: String,
    pub output: Vec<Output>,
    pub usage: Usage,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Output {
    pub id: String,
    pub status: String,
    pub role: String,
    pub content: Vec<Content>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Content {
    pub text: String,
    pub annotations: Vec<Value>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Usage {
    pub input_tokens: i64,
    pub output_tokens: i64,
    pub total_tokens: i64,
}
