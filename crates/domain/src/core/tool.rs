use std::error::Error;

/// The tool implementation interface
/// It will be used to execute the tool capabilities, e.g.: Read, Write, etc.
pub trait Tool<HandlerResult, HandlerError>
where
    HandlerError: Error,
{
    /// Executes the tool action, e.g.: ReadTool: Tool, will read some file on
    /// the host machine, and result in a success or a io error.
    ///
    /// @param argument - Is a text provided from the context that should be
    /// handled how the instruction says.
    fn handle(
        &self,
        argument: &str,
    ) -> impl std::future::Future<Output = Result<HandlerResult, HandlerError>> + Send;

    /// Is the tool description, says when the tool should be used.
    fn context(&self) -> &'static str;

    /// The tool usage instruction, says how the tool should be used.
    fn instruction(&self) -> &'static str;
}
