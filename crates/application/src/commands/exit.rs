use agente_domain::core::command::Command;

#[derive(Default)]
pub struct ExitCommand;

impl Command for ExitCommand {
    fn execute(&self) -> Result<Option<String>, agente_domain::core::Error> {
        std::process::exit(0);
    }

    fn name(&self) -> &'static str {
        "exit"
    }

    fn description(&self) -> &'static str {
        "Exit's the program."
    }
}
