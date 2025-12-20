use agente_domain::core::tool::{Tool, ToolError};

pub struct TalkTool;

impl TalkTool {
    pub fn new() -> Self {
        Self {}
    }
}

// @TODO: in progress
#[async_trait::async_trait]
impl Tool for TalkTool {
    async fn handle(
        &self,
        _arguments: Vec<String>,
    ) -> Result<Option<String>, ToolError> {
        Ok(None)
    }

    fn context(&self) -> &'static str {
        "this tool should be used when you the prompt is a simple sentence or \
         question and  no other tool is matched"
    }

    fn format_instruction(&self) -> Option<&'static str> {
        None
    }

    fn usage_instruction(&self) -> Option<&'static str> {
        None
    }
}
