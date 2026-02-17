use std::collections::HashMap;
use std::sync::Arc;

use tracing::{info, warn};

use agente_domain::core::Error;
use agente_domain::core::command::Command;
use agente_domain::ports::agent::Agent;
use agente_infrastructure::config::Config;

use crate::context::Context;

pub struct Processor {
    agent: Box<dyn Agent>,
    commands: HashMap<String, Box<dyn Command>>,
    context: Context,
}

impl Processor {
    pub fn init(
        agent: Box<dyn Agent>,
        config: Arc<Config>,
        commands: HashMap<String, Box<dyn Command>>,
    ) -> Self {
        Self {
            agent,
            commands,
            context: Context::init(
                config.clone(),
                system_prompt(
                    &config.system_prompt_path.clone().expect(
                        "Bug: system_prompt_path must be always setted",
                    ),
                ),
            ),
        }
    }

    pub async fn handle(
        &mut self,
        input: String,
    ) -> Result<Option<String>, Error> {
        if input.starts_with("/") {
            let command_result = self.process_command(input)?;
            return Ok(match command_result {
                Some(result) => Some(self.process_ask(result).await?),
                None => None,
            });
        }

        let response = self.process_ask(input).await?;
        Ok(Some(response))
    }

    async fn process_ask(&mut self, input: String) -> Result<String, Error> {
        self.context.summarize(&self.agent).await?;

        info!("asking...");
        let ask_response = self.context.ask(&self.agent, input).await?;
        Ok(ask_response.content)
    }

    fn process_command(&self, input: String) -> Result<Option<String>, Error> {
        if let Some(command) = self.commands.get(&input.clone().split_off(1)) {
            return Ok(command.execute()?);
        } else {
            warn!("Command not found for {input}!");
            return Err(Error::new("Command not found"));
        }
    }
}

use agente_application::prompt::load;

fn system_prompt(path: &str) -> String {
    load(path, vec![]).expect("Failed to load system prompt")
}
