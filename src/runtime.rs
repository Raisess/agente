use std::collections::HashMap;

use tracing::{error, info};

use agente_domain::core::tool::Tool;
use agente_domain::ports::agent::{
    Agent, FeedResponse, MessageRequest, MessageRole,
};

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
        // @TODO: summarize message history time to time
        let mut message_history = Vec::<MessageRequest>::new();
        message_history.push(MessageRequest {
            role: MessageRole::System,
            content: system_prompt(&self.tools),
        });

        loop {
            let mut input = String::new();
            println!("Prompt: ");
            std::io::stdin()
                .read_line(&mut input)
                .expect("Should have a input");

            message_history.push(MessageRequest {
                role: MessageRole::User,
                content: input,
            });
            println!("{message_history:#?}");
            let execution_plan = self
                .agent
                .ask(&message_history)
                .await
                .expect("Failed to ask the agent for the execution plan");
            info!(name: "response", "{execution_plan:#?}");

            let execution_summary = format!(
                "This is the execution plan: {}",
                execution_plan
                    .iter()
                    .map(|task| task.summary())
                    .collect::<Vec<_>>()
                    .join(" -> ")
            );
            message_history.push(MessageRequest {
                role: MessageRole::User,
                content: execution_summary,
            });

            let mut last_feed_response = FeedResponse::default();
            for task in execution_plan {
                let key = task.tool();
                let tool = self
                    .tools
                    .get(&key)
                    .expect(&format!("Tool not found: {key}"));

                let mut args = task.arguments();
                args.push(last_feed_response.content.clone());
                last_feed_response = FeedResponse::default();

                match tool.handle(args).await {
                    // @TODO: there should be two types of message, one for
                    // feeding and another only for showing up, create a enum a
                    // process it.
                    Ok(result) => {
                        info!(name: "tool_result", "{key}: {result:#?}");
                        if let Some(mut message) = result {
                            if let Some(usage) = tool.usage_instruction() {
                                message = format!("{usage}: {message}");
                            }

                            message_history.push(MessageRequest {
                                role: MessageRole::User,
                                content: message,
                            });

                            info!(name: "message_history", "{message_history:#?}");
                            last_feed_response = self
                                .agent
                                .feed(&message_history.clone().split_off(1))
                                .await
                                .expect(
                                    "Failed to feed agent with tool result \
                                     information",
                                );
                            info!(name: "feed_response", "{last_feed_response:#?}");

                            // message_history.push(MessageRequest {
                            // role: MessageRole::Assistant,
                            // content: format!("Done: {}", task.summary()),
                            // });
                        }
                    }
                    Err(error) => error!("{}", error.message()),
                }
            }
        }
    }
}
