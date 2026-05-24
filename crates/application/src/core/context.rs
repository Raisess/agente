use std::sync::Arc;

use tracing::info;

use agente_domain::models::message::Message;
use agente_domain::ports::ai_provider::{
    AiProvider, AiProviderError, AskResponse, MessageRequest, MessageRole,
};
use agente_infrastructure::adapters::util::load_file_installed::load_file_installed;
use agente_infrastructure::config::Config;

use crate::core::append_to_conversation;
use crate::repositories::conversation::ConversationRepository;

pub struct Context {
    conversation_repository: Arc<ConversationRepository>,
    session_id: String,
    messages: Vec<MessageRequest>,
}

impl Context {
    pub fn init(
        name: String,
        conversation_repository: Arc<ConversationRepository>,
        session_id: String,
        messages: Vec<Message>,
    ) -> Self {
        // @TODO: should get messages from conversation as summarized
        let mut message_requests = vec![MessageRequest {
            role: MessageRole::System,
            content: system_prompt(name),
        }];

        for message in messages {
            message_requests.push(MessageRequest {
                role: message.role.into(),
                content: message.content,
            })
        }

        info!(name: "messages", "{:#?}", message_requests);
        Self {
            conversation_repository,
            session_id,
            messages: message_requests,
        }
    }

    pub async fn ask(
        &mut self,
        agent: &Box<dyn AiProvider>,
        prompt: String,
        is_refeed: bool,
    ) -> Result<AskResponse, AiProviderError> {
        info!("asking...");
        let role = if is_refeed {
            MessageRole::Assistant
        } else {
            MessageRole::User
        };

        self.messages.push(MessageRequest {
            role: role.clone(),
            content: prompt.clone(),
        });

        info!(name: "history", "{:#?}", self.messages);
        match agent.ask(self.messages.clone()).await {
            Ok(ask_response) => {
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

                append_to_conversation(
                    self.conversation_repository.clone(),
                    self.session_id.clone(),
                    role,
                    prompt,
                )
                .await
                .expect("Failed to append message to conversation");

                Ok(ask_response)
            }
            Err(error) => {
                self.messages.pop();
                Err(error)
            }
        }
    }

    // @TODO: should save summarized conversations into the db and fetch it
    // instead of listing every message
    pub async fn summarize(
        &mut self,
        agent: &Box<dyn AiProvider>,
        force: bool,
    ) -> Result<(), AiProviderError> {
        if self.messages.len() >= Config::max_context_memory_size() || force {
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

    load_file_installed(
        "prompts/context/summarizer.md",
        vec![("messages", messages_prompt)],
    )
}

fn system_prompt(name: String) -> String {
    load_file_installed(
        "prompts/context/system.md",
        vec![("name", name), ("current_dir", Config::pwd())],
    )
}
