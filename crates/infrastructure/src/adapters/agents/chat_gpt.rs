use serde::{Deserialize, Serialize};
use serde_json::Value;

use agente_domain::core::models::task::Task;
use agente_domain::ports::agent::{
    Agent, AgentError, FeedResponse, MessageRequest,
};

pub struct ChatGPT {
    api_key: String,
    client: reqwest::Client,
}

impl ChatGPT {
    pub fn new(api_key: String) -> Self {
        Self {
            api_key,
            client: reqwest::Client::new(),
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
        let json = serde_json::json!({
            "input": input,
            "model": "gpt-3.5-turbo",
        });

        self.client
            .post("https://api.openai.com/v1/responses")
            .header("Content-Type", "application/json")
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&json)
            .send()
            .await
    }
}

#[async_trait::async_trait]
impl Agent for ChatGPT {
    async fn feed(
        &mut self,
        messages: Vec<MessageRequest>,
    ) -> Result<FeedResponse, AgentError> {
        let response = self.send_message(messages).await;
        match response {
            Ok(response) => {
                let status = response.status().as_u16();
                if let Some(error) = status_to_error(status) {
                    return Err(error);
                }

                match response.json::<Response>().await {
                    Ok(data) => {
                        let content = extract_response_text(data);
                        Ok(FeedResponse { content })
                    }
                    Err(error) => Err(AgentError::FailedToParseResponse(
                        error.to_string(),
                    )),
                }
            }
            Err(error) => Err(AgentError::Other(error.to_string())),
        }
    }

    async fn ask(
        &mut self,
        messages: Vec<MessageRequest>,
    ) -> Result<Vec<Task>, AgentError> {
        let response = self.send_message(messages).await;
        match response {
            Ok(response) => {
                let status = response.status().as_u16();
                if let Some(error) = status_to_error(status) {
                    return Err(error);
                }

                match response.json::<Response>().await {
                    Ok(data) => {
                        let text = extract_response_text(data);
                        let tasks = serde_json::from_str::<Vec<Task>>(&text)
                            .expect(&format!(
                                "To deserialize to task model, provided text: \
                                 {text}"
                            ));

                        Ok(tasks)
                    }
                    Err(error) => Err(AgentError::FailedToParseResponse(
                        error.to_string(),
                    )),
                }
            }
            Err(error) => Err(AgentError::Other(error.to_string())),
        }
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
