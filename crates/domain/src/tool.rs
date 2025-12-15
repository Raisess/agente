use std::error::Error;

/// The tool implementation interface
/// It will be used to execute the tool capabilities, e.g.: Read, Write, etc.
pub trait Tool<HandlerResult, HandlerError>
where
    HandlerError: Error,
{
    /// Executes the tool action, e.g.: ReadTool: Tool, will read some file on
    /// the host machine, and result in a success or a io error.
    fn handle(
        &self,
    ) -> impl std::future::Future<Output = Result<HandlerResult, HandlerError>> + Send;

    /// Is the tool description, in what context that tool should be used.
    fn context() -> &'static str;
}
