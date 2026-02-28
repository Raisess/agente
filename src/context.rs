use tracing::info;

use agente_domain::ports::agent::{
    Agent, AgentError, AskResponse, MessageRequest, MessageRole,
};
use agente_infrastructure::config::Config;

use crate::prompt::load;

const MAX_MESSAGES: usize = 10;

pub struct Context {
    messages: Vec<MessageRequest>,
}

impl Context {
    pub fn init() -> Self {
        let system_prompt = system_prompt(
            "__prompts/system.md",
            Config::pwd().expect("Can't get current directory"),
        );

        Self {
            messages: vec![MessageRequest {
                role: MessageRole::System,
                content: system_prompt,
            }],
        }
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
    ) -> Result<(), AgentError> {
        if self.messages.len() >= MAX_MESSAGES {
            info!("summarizing...");
            let messages = self.messages.drain(1..).collect::<Vec<_>>();
            println!("{messages:#?}");
            let result = agent
                .ask(vec![MessageRequest {
                    role: MessageRole::User,
                    content: summarize_messages_prompt(
                        "__prompts/summarizer.md",
                        messages,
                    ),
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

fn summarize_messages_prompt(
    path: &str,
    messages: Vec<MessageRequest>,
) -> String {
    let messages_prompt = messages
        .iter()
        .map(|MessageRequest { role, content }| {
            format!("Role: {role}, Content: {content}")
        })
        .collect::<Vec<_>>()
        .join(", ");

    load(path, vec![("messages", messages_prompt)])
        .expect("Failed to load summarizer prompt")
}

fn system_prompt(path: &str, pwd: String) -> String {
    load(path, vec![("current_dir", pwd)])
        .expect("Failed to load system prompt")
}
