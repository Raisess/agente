use tracing::info;

use agente_domain::core::models::task::Task;
use agente_domain::ports::agent::{
    Agent, AgentError, MessageRequest, MessageRole,
};

const MAX_MESSAGES: usize = 10;

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
        &self,
        agent: &Box<dyn Agent>,
        prompt: String,
    ) -> Result<Vec<Task>, AgentError> {
        // @NOTE: copys the messages and keep the user prompt temporaly because
        // the context will only keep the execution summary information.
        let mut execution_prompt = self.messages.clone();
        execution_prompt.push(MessageRequest {
            role: MessageRole::User,
            content: prompt,
        });

        let execution_plan = agent.ask(execution_prompt).await?;
        info!(name: "execution_plan", "{:#?}", execution_plan);
        Ok(execution_plan)
    }

    pub async fn feed(
        &mut self,
        agent: &Box<dyn Agent>,
        content: String,
    ) -> Result<String, AgentError> {
        self.messages.push(MessageRequest {
            role: MessageRole::User,
            content,
        });

        let result = agent.feed(self.messages.clone().split_off(1)).await?;
        info!(name: "feed_response", "{result:#?}");
        self.messages.push(MessageRequest {
            role: MessageRole::Assistant,
            content: result.content.clone(),
        });

        Ok(result.content)
    }

    pub async fn summarize(
        &mut self,
        agent: &Box<dyn Agent>,
    ) -> Result<(), AgentError> {
        if self.messages.len() >= MAX_MESSAGES {
            info!("summarizing...");
            let messages = self.messages.drain(1..).collect::<Vec<_>>();
            let result = agent
                .feed(vec![MessageRequest {
                    role: MessageRole::User,
                    content: summarize_messages_prompt(messages),
                }])
                .await?;

            info!("summarized: {}", result.content);
            self.messages.push(MessageRequest {
                role: MessageRole::System,
                content: result.content,
            });
        }

        Ok(())
    }
}

use agente_application::prompt::load;

fn summarize_messages_prompt(messages: Vec<MessageRequest>) -> String {
    let messages_prompt = messages
        .iter()
        .map(|MessageRequest { role, content }| {
            format!("Role: {role}, Content: {content}")
        })
        .collect::<Vec<_>>()
        .join(", ");

    load("summarizer", vec![("messages", messages_prompt)])
        .expect("Failed to load summarizer prompt")
}
