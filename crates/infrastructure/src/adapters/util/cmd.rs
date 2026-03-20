use std::process::Command;

use agente_domain::ports::io::{Executor, ExecutorArgument};

#[derive(Default)]
pub struct CMD;

impl Executor for CMD {
    fn exec(
        &self,
        cmd: &str,
        args: Vec<ExecutorArgument>,
    ) -> Result<String, std::io::Error> {
        let mut command = Command::new(cmd);
        for arg in args {
            match arg {
                ExecutorArgument::Arg(argument) => command.arg(argument),
                ExecutorArgument::Flag((flag, argument)) => {
                    command.args([format!("--{flag}"), argument])
                }
            };
        }

        let output = command.output()?;
        if output.status.code().is_some_and(|status| status != 0) {
            return Ok(String::from_utf8_lossy(&output.stderr).into());
        }

        return Ok(String::from_utf8_lossy(&output.stdout).into());
    }
}
