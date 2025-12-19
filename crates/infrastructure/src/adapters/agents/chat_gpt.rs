use serde::{Deserialize, Serialize};
use serde_json::Value;

use agente_domain::core::models::task::Task;
use agente_domain::ports::agent::{Agent, AgentError};

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
}

#[async_trait::async_trait]
impl Agent for ChatGPT {
    async fn ask(&self, prompt: &str) -> Result<Vec<Task>, AgentError> {
        let response = self
            .client
            .post("https://api.openai.com/v1/responses")
            .header("Content-Type", "application/json")
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&serde_json::json!({ "model": "gpt-3.5-turbo", "input": prompt }))
            .send()
            .await;

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
        429 => Some(AgentError::OutOfCredits),
        503 => Some(AgentError::ServicesOverloaded),
        _ => None,
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Response {
    pub id: String,
    pub object: String,
    #[serde(rename = "created_at")]
    pub created_at: i64,
    pub status: String,
    pub error: Option<Value>,
    #[serde(rename = "incomplete_details")]
    pub incomplete_details: Option<Value>,
    pub instructions: Option<Value>,
    #[serde(rename = "max_output_tokens")]
    pub max_output_tokens: Option<Value>,
    pub model: String,
    pub output: Vec<Output>,
    #[serde(rename = "parallel_tool_calls")]
    pub parallel_tool_calls: bool,
    #[serde(rename = "previous_response_id")]
    pub previous_response_id: Option<Value>,
    pub reasoning: Reasoning,
    pub store: bool,
    pub temperature: f64,
    pub text: TextFormat,
    #[serde(rename = "tool_choice")]
    pub tool_choice: String,
    pub tools: Vec<Value>,
    #[serde(rename = "top_p")]
    pub top_p: f64,
    pub truncation: String,
    pub usage: Usage,
    pub user: Option<Value>,
    pub metadata: Value,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Output {
    pub r#type: String,
    pub id: String,
    pub status: String,
    pub role: String,
    pub content: Vec<Content>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Content {
    pub r#type: String,
    pub text: String,
    pub annotations: Vec<Value>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Reasoning {
    pub effort: Option<Value>,
    pub summary: Option<Value>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TextFormat {
    pub r#type: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Usage {
    #[serde(rename = "input_tokens")]
    pub input_tokens: i64,
    #[serde(rename = "input_tokens_details")]
    pub input_tokens_details: TokenDetails,
    #[serde(rename = "output_tokens")]
    pub output_tokens: i64,
    #[serde(rename = "output_tokens_details")]
    pub output_tokens_details: TokenDetails,
    #[serde(rename = "total_tokens")]
    pub total_tokens: i64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TokenDetails {
    #[serde(rename = "cached_tokens")]
    pub cached_tokens: i64,
}
