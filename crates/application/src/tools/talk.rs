use agente_domain::core::Error;
use agente_domain::core::tool::{Tool, ToolResponse};

pub struct TalkTool;

impl TalkTool {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait::async_trait]
impl Tool for TalkTool {
    async fn handle(
        &self,
        arguments: Vec<String>,
    ) -> Result<ToolResponse, Error> {
        Ok(ToolResponse {
            data: arguments
                .get(0)
                .cloned()
                .unwrap_or(String::from("Empty response")),
            is_feedable: false,
        })
    }

    fn context(&self) -> &'static str {
        "this tool should be used when you the prompt is a simple sentence or \
         question and no other tool is matched, pass the response as the first \
         argument"
    }

    fn format_instruction(&self) -> Option<&'static str> {
        Some(
            "provide the response as the result in a format like this: \
             [\"<response>\"]",
        )
    }

    fn usage_instruction(&self) -> Option<&'static str> {
        None
    }
}
