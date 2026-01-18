use crate::core::Error;

/// The command implementation interface
/// Describes a command execution, e.g.: /exit, should exit the program.
pub trait Command {
    /// Handles the command behavior.
    fn execute(&self) -> Result<(), Error>;

    /// The command name that will be the identifier, e.g.: /name.
    fn name() -> &'static str;

    /// Describes what the commands does.
    fn description() -> &'static str;
}
