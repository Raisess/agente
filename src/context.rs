use tracing::info;

use agente_domain::ports::agent::{
    Agent, AgentError, AskResponse, MessageRequest, MessageRole,
};
use agente_infrastructure::config::Config;
use agente_infrastructure::load_file::load;

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
            content: match ask_response {
                AskResponse::Content(ref text) => text.clone(),
                AskResponse::ToolCall(ref tools) => {
                    // @FIXME: this should consider tool arguments
                    format!(
                        "Executed tools: {}",
                        tools
                            .iter()
                            .map(|(tool, _)| tool.clone())
                            .collect::<Vec<_>>()
                            .join(", ")
                    )
                }
            },
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

            match result {
                AskResponse::Content(text) => {
                    info!("summarized: {}", text);
                    self.messages.push(MessageRequest {
                        role: MessageRole::System,
                        content: text,
                    });
                }
                _ => {}
            }
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

fn system_prompt() -> String {
    load("__prompts/system.md", vec![("current_dir", Config::pwd())])
        .expect("Failed to load system prompt")
}
