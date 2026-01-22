use std::sync::Arc;

use agente_domain::core::Error;
use agente_domain::core::tool::{Tool, ToolResponse};
use agente_domain::ports::io::Executor;

pub struct BashTool {
    executor: Arc<dyn Executor>,
}

impl BashTool {
    pub fn new(executor: Arc<dyn Executor>) -> Self {
        Self { executor }
    }
}

#[async_trait::async_trait]
impl Tool for BashTool {
    async fn handle(
        &self,
        arguments: Vec<String>,
    ) -> Result<ToolResponse, Error> {
        let command = arguments
            .get(0)
            .expect("Command must be provided as argument 0");

        let output = self.executor.exec(command)?;
        Ok(ToolResponse {
            data: output,
            is_feedable: true,
        })
    }

    fn context(&self) -> &'static str {
        "This tool should be used to execute bash commands and get the output."
    }

    fn format_instruction(&self) -> Option<&'static str> {
        Some(
            "provide the bash command as the argument like this: [\"<bash \
             command>\"]",
        )
    }

    fn usage_instruction(&self) -> Option<&'static str> {
        Some("Execute a bash command and get its output")
    }
}
