use crate::core::Error;

/// The tool implementation interface
/// It will be used to execute the tool capabilities, e.g.: Read, Write, etc.
#[async_trait::async_trait]
pub trait Tool: Send + Sync {
    /// Executes the tool action, e.g.: ReadTool: Tool, will read some file on
    /// the host machine, and result in a success or a io error.
    ///
    /// @param argument - Is a text provided from the context that should be
    /// handled how the instruction says.
    async fn handle(
        &self,
        arguments: Vec<String>,
    ) -> Result<ToolResponse, Error>;

    /// Is the tool description, says when the tool should be used.
    fn context(&self) -> &'static str;

    /// Is the tool parameter format instruction, to be passed as arguments.
    fn format_instruction(&self) -> Option<&'static str>;

    /// The tool usage instruction, says how the tool return should be used for
    /// the next prompt.
    fn usage_instruction(&self) -> Option<&'static str>;
}

// @TODO: had stream response (it will be good for the Bash tool)
/// Is the tool execution result
#[derive(Debug)]
pub struct ToolResponse {
    /// The actual tool response data.
    pub data: String,
    /// Says if it should be feeded to the agent or not.
    pub is_feedable: bool,
}
