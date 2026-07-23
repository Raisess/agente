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
        conversation_repository: Arc<ConversationRepository>,
        name: String,
        custom_system_prompt: Option<String>,
        session_id: String,
        messages: Vec<Message>,
    ) -> Self {
        // @TODO: should get messages from conversation as summarized
        let date = chrono::Utc::now().format("%Y/%m/%d").to_string();
        let mut message_requests = vec![MessageRequest {
            role: MessageRole::System,
            content: system_prompt(name, Config::pwd(), date, custom_system_prompt),
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

    #[inline]
    pub fn session_id(&self) -> String {
        self.session_id.clone()
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
                            format!(
                                "Executed tools: {}",
                                tools
                                    .iter()
                                    .map(|(tool, args)| AskResponse::generate_tool_hash(
                                        tool, args
                                    ))
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
    // instead of listing every message.
    // @TODO: should save a conversation resume in the session table to us can
    // list the current sessions and see what they are about and get the id to
    // start it.
    pub async fn summarize(
        &mut self,
        agent: &Box<dyn AiProvider>,
        force: bool,
    ) -> Result<(), AiProviderError> {
        if self.messages.len() >= Config::max_context_memory_size() || force {
            info!("summarizing...");
            let messages = self.messages.drain(1..).collect::<Vec<_>>();
            let messages_prompt = messages
                .iter()
                .map(|MessageRequest { role, content }| {
                    format!("Role: {role}, Content: {content}")
                })
                .collect::<Vec<_>>()
                .join(", ");

            let summarized_text = agent
                .plain_ask(vec![
                    MessageRequest {
                        role: MessageRole::System,
                        content: summarize_messages_prompt(),
                    },
                    MessageRequest {
                        role: MessageRole::User,
                        content: messages_prompt,
                    },
                ])
                .await?;

            info!("summarized: {}", summarized_text);
            self.messages.push(MessageRequest {
                role: MessageRole::Assistant,
                content: format!("Conversation summary until now: {summarized_text}"),
            });
        }

        Ok(())
    }
}

fn summarize_messages_prompt() -> String {
    load_file_installed("prompts/context/summarizer.md", vec![])
}

fn system_prompt(
    name: String,
    current_dir: String,
    date: String,
    custom_prompt: Option<String>,
) -> String {
    load_file_installed(
        "prompts/context/system.md",
        vec![("name", name), ("current_dir", current_dir), ("date", date), ("custom_prompt", custom_prompt.unwrap_or("You are an autonomous AI agent running in a agentic loop, designed to help users accomplish tasks, solve problems, and provide accurate information.".to_string()))],
    )
}
