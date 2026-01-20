use std::process::Command;

use agente_domain::ports::io::Executor;

#[derive(Default)]
pub struct CMD;

// @TODO: had stream response
impl Executor for CMD {
    fn exec(&self, cmd: &str) -> Result<String, std::io::Error> {
        let output = Command::new("bash").arg("-c").arg(cmd).output()?;

        if output.status.code().is_some_and(|status| status != 0) {
            return Ok(String::from_utf8_lossy(&output.stderr).into());
        }

        return Ok(String::from_utf8_lossy(&output.stdout).into());
    }
}
