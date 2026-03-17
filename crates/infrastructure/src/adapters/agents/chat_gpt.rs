use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use agente_domain::ports::agent::{
    Agent, AgentError, AskResponse, MessageRequest,
};

use crate::load_file::load;

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
    ) -> Result<Vec<Output>, AgentError> {
        let response = self
            .send_message(messages)
            .await
            .map_err(|error| AgentError::Other(error.to_string()))?;

        let status = response.status().as_u16();
        if let Some(error) = status_to_error(status) {
            return Err(error);
        }

        let data = response
            .json::<HashMap<String, serde_json::Value>>()
            .await
            .map_err(|error| {
                AgentError::FailedToParseResponse(error.to_string())
            })?;

        let output = data.get("output");
        match output {
            Some(valid_output) => {
                Ok(serde_json::from_value(valid_output.clone()).unwrap())
            }
            None => {
                eprintln!("Response: {data:#?}");
                Err(AgentError::FailedToParseResponse(
                    "Invalid response".to_string(),
                ))
            }
        }
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

        let tools = serde_json::from_str::<serde_json::Value>(
            &load("./tools.json", vec![])
                .expect("Failed to load tools.json file"),
        )
        .expect("Failed to parse tools json");

        let json = serde_json::json!({
            "input": input,
            "model": self.config.model, // gpt-3.5-turbo
            "tools": tools,
            "tool_choice": "auto",
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
        let output = self.handle_message_request(messages).await?;

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
                            Some((
                                name.clone(),
                                serde_json::from_str(arguments).unwrap(),
                            ))
                        } else {
                            None
                        }
                    })
                    .collect::<Vec<_>>(),
            ),
        })
    }
}

fn status_to_error(status: u16) -> Option<AgentError> {
    match status {
        429 => Some(AgentError::Limited),
        503 => Some(AgentError::ServicesOverloaded),
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
