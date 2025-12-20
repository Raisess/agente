use std::collections::HashMap;

use tracing::{error, info};

use agente_domain::core::tool::Tool;
use agente_domain::ports::agent::{Agent, FeedResponse, MessageRequest};

use crate::prompt::prompt;

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
        // @FIXME: this should only initialize if the user sends a prompt
        let initial_feed_response = self
            .agent
            .feed(MessageRequest {
                previous_message_id: None,
                prompt: prompt(&self.tools),
            })
            .await
            .expect("Failed to setup agent initial prompt.");

        let mut feed_result = FeedResponse::default();
        loop {
            let mut input = String::new();
            println!("Prompt: ");
            std::io::stdin()
                .read_line(&mut input)
                .expect("Should have a input");

            let execution_plan = self
                .agent
                .ask(MessageRequest {
                    previous_message_id: initial_feed_response
                        .message_id
                        .clone(),
                    prompt: input.trim().to_string(),
                })
                .await
                .expect("Failed to ask the agent for the execution plan.");
            info!(name: "response", "{execution_plan:#?}");

            for task in execution_plan {
                let key = task.tool();
                let tool = self
                    .tools
                    .get(&key)
                    .expect(&format!("Tool not found: {key}"));

                // @FIXME: improve this hardcoded ugly thing here
                let mut args = task.arguments();
                if key == "Write"
                    && args.get(1).is_some_and(|value| value == "<NONE>")
                {
                    args[1] = feed_result.content.clone();
                }

                let result = tool.handle(args).await;
                info!(name: "tool_result", "{key}: {result:#?}");
                match result {
                    // @TODO: there should be two types of message, one for
                    // feeding and another only for showing up, create a enum a
                    // process it.
                    Ok(message) => {
                        if let Some(mut message) = message {
                            if let Some(usage_instruction) =
                                tool.usage_instruction()
                            {
                                message =
                                    format!("{usage_instruction}: {message}");
                            }

                            feed_result = self
                                .agent
                                .feed(MessageRequest {
                                    previous_message_id: None,
                                    prompt: message,
                                })
                                .await
                                .expect(
                                    "Failed to feed agent with tool result \
                                     information.",
                                );
                            info!(name: "feed_result", "{feed_result:#?}");
                        }
                    }
                    Err(error) => error!("{}", error.message()),
                }
            }
        }
    }
}
