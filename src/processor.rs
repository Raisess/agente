use std::sync::Arc;

use agente_domain::error::Error;
use agente_domain::ports::agent::Agent;
use agente_infrastructure::config::Config;

use crate::context::Context;

pub struct Processor {
    agent: Box<dyn Agent>,
    config: Arc<Config>,
    context: Context,
}

impl Processor {
    pub fn init(agent: Box<dyn Agent>, config: Arc<Config>) -> Self {
        let context = Context::init(config.clone());
        Self {
            agent,
            config,
            context,
        }
    }

    pub async fn handle(&mut self, input: String) -> Result<String, Error> {
        // @FIXME: support select agent
        if self.config.chat_gpt.api_key.is_empty() {
            return Err(Error::new("No API Key provided"));
        }

        self.context.summarize(&self.agent).await?;

        let ask_response = self.context.ask(&self.agent, input).await?;
        Ok(ask_response.content)
    }
}
