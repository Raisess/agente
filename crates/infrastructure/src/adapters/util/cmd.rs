use std::process::Command;

use agente_domain::error::Error;
use agente_domain::ports::io::{Executor, ExecutorArgument};

#[derive(Default)]
pub struct CMD;

impl Executor for CMD {
    fn exec(
        &self,
        cmd: &str,
        args: Vec<ExecutorArgument>,
        envs: Vec<(String, String)>,
    ) -> Result<String, Error> {
        let mut command = Command::new(cmd);
        command.envs(envs);
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
            let error_msg = String::from_utf8_lossy(&output.stderr);
            return Err(Error::new(&error_msg));
        }

        return Ok(String::from_utf8_lossy(&output.stdout).into());
    }
}
