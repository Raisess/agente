use std::error::Error;

/// The command implementation interface
/// Describes a command execution, e.g.: /exit, should exit the program.
pub trait Command<ExecutionResult, ExecutionError>
where
    ExecutionError: Error,
{
    /// Handles the command behavior.
    fn execute(&self) -> Result<ExecutionResult, ExecutionError>;

    /// The command name that will be the identifier, e.g.: /name.
    fn name() -> &'static str;

    /// Describes what the commands does.
    fn description() -> &'static str;
}
