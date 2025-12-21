use std::collections::HashMap;

use tracing::{error, info};

use agente_domain::core::tool::Tool;
use agente_domain::ports::agent::{
    Agent, FeedResponse, MessageRequest, MessageRole,
};

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
        let tools_context_message = MessageRequest {
            role: MessageRole::System,
            content: prompt(&self.tools),
        };

        let mut feed_result = FeedResponse::default();
        loop {
            let mut input = String::new();
            println!("Prompt: ");
            std::io::stdin()
                .read_line(&mut input)
                .expect("Should have a input");

            let message = MessageRequest {
                role: MessageRole::User,
                content: input,
            };
            let execution_plan = self
                .agent
                .ask(&vec![tools_context_message.clone(), message])
                .await
                .expect("Failed to ask the agent for the execution plan");
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
                                // @TODO: describe better what to do with the
                                // result message
                                message =
                                    format!("{usage_instruction}: {message}");
                            }

                            // @TODO: a feed message context for execution plan
                            let feed_message = MessageRequest {
                                role: MessageRole::User,
                                content: message,
                            };
                            feed_result = self
                                .agent
                                .feed(&vec![feed_message])
                                .await
                                .expect(
                                    "Failed to feed agent with tool result \
                                     information",
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
