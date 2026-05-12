use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use agente_domain::ports::ai_provider::{
    AiProvider, AiProviderConfig, AiProviderError, AskResponse, MessageRequest,
    MessageRole,
};

use crate::adapters::util::load_file_installed::load_file_installed;

const DEFAULT_OPENAI_URL: &str = "https://api.openai.com/v1/responses";

pub struct OpenAI {
    config: AiProviderConfig,
    client: reqwest::Client,
}

impl OpenAI {
    pub fn new(config: AiProviderConfig) -> Self {
        Self {
            config,
            client: reqwest::Client::new(),
        }
    }

    async fn handle_message_request(
        &self,
        model: &String,
        messages: Vec<MessageRequest>,
    ) -> Result<Vec<Output>, AiProviderError> {
        let input = messages
            .iter()
            .map(|message| {
                serde_json::json!({
                  "role": message.role.to_string(),
                  "content": message.content
                })
            })
            .collect::<Vec<_>>();

        let tools = serde_json::from_str::<serde_json::Value>(&load_file_installed(
            "tools.json",
            vec![],
        ))
        .expect("Failed to parse tools json");

        let json = serde_json::json!({
            "input": input,
            "model": model,
            "tools": tools,
            "tool_choice": "auto",
        });

        let data = self.send_message(json).await?;
        Ok(serde_json::from_value(data).unwrap())
    }

    async fn handle_plain_message(
        &self,
        model: &String,
        messages: Vec<MessageRequest>,
    ) -> Result<String, AiProviderError> {
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
            "model": model,
        });

        let data = self.send_message(json).await?;
        let value = data
            .get(0)
            .unwrap()
            .get("content")
            .unwrap()
            .get(0)
            .unwrap()
            .get("text")
            .unwrap()
            .to_owned();
        Ok(serde_json::from_value(value).unwrap())
    }

    async fn send_message(
        &self,
        json: serde_json::Value,
    ) -> Result<serde_json::Value, AiProviderError> {
        let response = self
            .client
            .post(DEFAULT_OPENAI_URL)
            .header("Content-Type", "application/json")
            .header("Authorization", format!("Bearer {}", self.config.api_key))
            .json(&json)
            .send()
            .await
            .map_err(|error| AiProviderError::Other(error.to_string()))?;

        let status = response.status().as_u16();
        if let Some(error) = status_to_error(status) {
            return Err(error);
        }

        let data = response
            .json::<HashMap<String, serde_json::Value>>()
            .await
            .map_err(|error| AiProviderError::FailedToParseResponse(error.to_string()))?;

        let output = data.get("output");
        match output {
            Some(valid_output) => Ok(valid_output.clone()),
            None => {
                eprintln!("Response: {data:#?}");
                Err(AiProviderError::FailedToParseResponse(
                    "Invalid response".to_string(),
                ))
            }
        }
    }
}

#[async_trait::async_trait]
impl AiProvider for OpenAI {
    async fn ask(
        &self,
        messages: Vec<MessageRequest>,
    ) -> Result<AskResponse, AiProviderError> {
        let output = self
            .handle_message_request(&self.config.model, messages)
            .await?;

        Ok(match &output[0] {
            Output::Text { content, .. } => {
                AskResponse::Content(content[0].text.clone().unwrap())
            }
            Output::Tool { .. } => AskResponse::ToolCall(
                output
                    .iter()
                    .filter_map(|tool| {
                        if let Output::Tool {
                            name, arguments, ..
                        } = tool
                        {
                            Some((name.clone(), serde_json::from_str(arguments).unwrap()))
                        } else {
                            None
                        }
                    })
                    .collect::<Vec<_>>(),
            ),
        })
    }

    async fn plain_ask(
        &self,
        system: String,
        content: String,
    ) -> Result<String, AiProviderError> {
        let model = self
            .config
            .cheap_model
            .clone()
            .unwrap_or(self.config.model.clone());

        let messages = vec![
            MessageRequest {
                role: MessageRole::System,
                content: system,
            },
            MessageRequest {
                role: MessageRole::User,
                content,
            },
        ];

        self.handle_plain_message(&model, messages).await
    }
}

fn status_to_error(status: u16) -> Option<AiProviderError> {
    match status {
        429 => Some(AiProviderError::Limited),
        503 => Some(AiProviderError::ServicesOverloaded),
        _ => None,
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
enum Output {
    #[serde(rename = "message")]
    Text {
        id: String,
        role: String,
        status: String,
        content: Vec<MessageContent>,
    },
    #[serde(rename = "function_call")]
    Tool {
        id: String,
        call_id: String,
        name: String,
        status: String,
        arguments: String,
    },
}

#[derive(Debug, Serialize, Deserialize)]
struct MessageContent {
    #[serde(rename = "type")]
    content_type: String, // e.g., "output_text"
    text: Option<String>,
    annotations: Option<Vec<String>>,
    logprobs: Option<Vec<String>>,
}
