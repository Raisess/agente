use tracing::info;

use agente_domain::ports::agent::{
    Agent, AgentError, AskResponse, MessageRequest, MessageRole,
};
use agente_infrastructure::config::Config;

use crate::prompt::load;

const MAX_MESSAGE_HISTORY_SIZE: usize = 50;

pub struct Context {
    messages: Vec<MessageRequest>,
}

impl Context {
    pub fn init() -> Self {
        Self {
            messages: vec![MessageRequest {
                role: MessageRole::System,
                content: system_prompt(),
            }],
        }
    }

    pub async fn generate_tasks(
        &mut self,
        agent: &Box<dyn Agent>,
        prompt: String,
    ) -> Result<AskResponse, AgentError> {
        info!("generating tasks...");
        let messages = vec![
            MessageRequest {
                role: MessageRole::System,
                content: task_generator_prompt(),
            },
            MessageRequest {
                role: MessageRole::User,
                content: prompt,
            },
        ];
        let ask_response = agent.ask(messages).await?;
        info!("generated!");

        Ok(ask_response)
    }

    pub async fn ask(
        &mut self,
        agent: &Box<dyn Agent>,
        prompt: String,
    ) -> Result<AskResponse, AgentError> {
        info!("asking...");
        self.messages.push(MessageRequest {
            role: MessageRole::User,
            content: prompt,
        });

        info!(name: "history", "{:#?}", self.messages);
        let ask_response = agent.ask(self.messages.clone()).await?;
        self.messages.push(MessageRequest {
            role: MessageRole::Assistant,
            content: ask_response.content.clone(),
        });
        info!("done!");

        Ok(ask_response)
    }

    pub async fn summarize(
        &mut self,
        agent: &Box<dyn Agent>,
        force: bool,
    ) -> Result<(), AgentError> {
        if self.messages.len() >= MAX_MESSAGE_HISTORY_SIZE || force {
            info!("summarizing...");
            let messages = self.messages.drain(1..).collect::<Vec<_>>();
            let result = agent
                .ask(vec![MessageRequest {
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

fn summarize_messages_prompt(messages: Vec<MessageRequest>) -> String {
    let messages_prompt = messages
        .iter()
        .map(|MessageRequest { role, content }| {
            format!("Role: {role}, Content: {content}")
        })
        .collect::<Vec<_>>()
        .join(", ");

    load(
        "__prompts/summarizer.md",
        vec![("messages", messages_prompt)],
    )
    .expect("Failed to load summarizer prompt")
}

fn task_generator_prompt() -> String {
    load(
        "__prompts/task_generator.md",
        vec![("current_dir", Config::pwd())],
    )
    .expect("Failed to load task generator prompt")
}

fn system_prompt() -> String {
    load("__prompts/system.md", vec![("current_dir", Config::pwd())])
        .expect("Failed to load system prompt")
}
