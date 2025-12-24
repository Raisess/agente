use tracing::info;

use agente_domain::core::models::task::Task;
use agente_domain::ports::agent::{
    Agent, AgentError, MessageRequest, MessageRole,
};

use crate::prompt::summarize_messages_prompt;

const MAX_MESSAGES: usize = 5;

pub struct Context {
    messages: Vec<MessageRequest>,
}

impl Context {
    pub fn init(system_prompt: String) -> Self {
        Self {
            messages: vec![MessageRequest {
                role: MessageRole::System,
                content: system_prompt,
            }],
        }
    }

    pub async fn ask(
        &mut self,
        agent: &mut Box<dyn Agent>,
        prompt: String,
    ) -> Result<Vec<Task>, AgentError> {
        self.messages.push(MessageRequest {
            role: MessageRole::User,
            content: prompt,
        });

        let execution_plan = agent.ask(&self.messages).await?;

        let execution_summary = format!(
            "This is the execution plan: {}",
            execution_plan
                .iter()
                .map(|task| task.summary())
                .collect::<Vec<_>>()
                .join(" -> ")
        );
        self.messages.push(MessageRequest {
            role: MessageRole::User,
            content: execution_summary,
        });

        Ok(execution_plan)
    }

    pub async fn feed(
        &mut self,
        agent: &mut Box<dyn Agent>,
        content: String,
    ) -> Result<String, AgentError> {
        info!(name: "message_history", "{:#?}", self.messages);
        self.messages.push(MessageRequest {
            role: MessageRole::User,
            content,
        });

        let result = agent.feed(&self.messages.clone().split_off(1)).await?;
        info!(name: "feed_response", "{result:#?}");
        Ok(result.content)
    }

    pub async fn summarize(
        &mut self,
        agent: &mut Box<dyn Agent>,
    ) -> Result<(), AgentError> {
        if self.messages.len() >= MAX_MESSAGES {
            info!("summarizing...");
            let messages = self.messages.drain(1..).collect::<Vec<_>>();
            let message_request = MessageRequest {
                role: MessageRole::User,
                content: summarize_messages_prompt(&messages),
            };
            let result = agent.feed(&vec![message_request]).await?;

            self.messages.push(MessageRequest {
                role: MessageRole::System,
                content: result.content,
            });
            info!("summarized: {:#?}", self.messages);
        }

        Ok(())
    }
}
