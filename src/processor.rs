use std::sync::Arc;

use agente_domain::error::Error;
use agente_domain::ports::agent::Agent;
use agente_infrastructure::config::Config;

use crate::context::Context;
use crate::prompt::load;

pub struct Processor {
    agent: Box<dyn Agent>,
    context: Context,
}

impl Processor {
    pub fn init(agent: Box<dyn Agent>, config: Arc<Config>) -> Self {
        Self {
            agent,
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

    pub async fn handle(&mut self, input: String) -> Result<String, Error> {
        self.context.summarize(&self.agent).await?;

        let ask_response = self.context.ask(&self.agent, input).await?;
        Ok(ask_response.content)
    }
}

fn system_prompt(path: &str) -> String {
    load(path, vec![]).expect("Failed to load system prompt")
}
