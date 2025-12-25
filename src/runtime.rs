use std::collections::HashMap;

use tracing::{error, info};

use agente_domain::core::tool::Tool;
use agente_domain::ports::agent::Agent;

use crate::context::Context;
use crate::prompt::system_prompt;

pub struct Runtime {
    agent: Box<dyn Agent>,
    tools: HashMap<String, Box<dyn Tool>>,
}

impl Runtime {
    pub fn init(
        agent: Box<dyn Agent>,
        tools: HashMap<String, Box<dyn Tool>>,
    ) -> Self {
        Self { agent, tools }
    }

    pub async fn run(&mut self) -> () {
        let mut context = Context::init(system_prompt(&self.tools));

        loop {
            let mut input = String::new();
            println!("Prompt: ");
            std::io::stdin()
                .read_line(&mut input)
                .expect("Should have a input");

            let execution_plan = context
                .ask(&mut self.agent, input)
                .await
                .expect("Failed to ask the agent for the execution plan");

            let mut last_feed_response = String::new();
            for task in execution_plan {
                let key = task.tool();
                let tool = self
                    .tools
                    .get(&key)
                    .expect(&format!("Tool not found: {key}"));

                let mut args = task.arguments();
                args.push(last_feed_response);
                last_feed_response = String::new();

                match tool.handle(args).await {
                    Ok(result) => {
                        info!(name: "tool_result", "{key}: {result:#?}");
                        if result.is_feedable && !result.data.is_empty() {
                            let mut message = result.data;
                            if let Some(usage) = tool.usage_instruction() {
                                message = format!("{usage}: {message}");
                            }

                            last_feed_response = context
                                .feed(&mut self.agent, message)
                                .await
                                .expect(
                                    "Failed to feed agent with tool result \
                                     information",
                                );
                        } else {
                            println!("Response: {}", result.data);
                        }
                    }
                    Err(error) => error!("{}", error.message()),
                }
            }

            context
                .summarize(&mut self.agent)
                .await
                .expect("Failed to summarize messages");
        }
    }
}
