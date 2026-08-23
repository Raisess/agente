use std::str::FromStr;
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
use crate::repositories::session::SessionRepository;

const DEFAULT_CUSTOM_SYSTEM_PROMPT: &str = "You are an autonomous AI agent running in a agentic loop, designed to help users accomplish tasks, solve problems, and provide accurate information.";

pub struct Context {
    __start_messages_count: usize,
    session_repository: Arc<SessionRepository>,
    conversation_repository: Arc<ConversationRepository>,
    session_id: String,
    messages: Vec<MessageRequest>,
}

impl Context {
    pub fn init(
        session_repository: Arc<SessionRepository>,
        conversation_repository: Arc<ConversationRepository>,
        name: String,
        session_id: String,
        messages: Vec<Message>,
        custom_system_prompt: Option<String>,
    ) -> Self {
        let date = chrono::Utc::now().format("%Y/%m/%d").to_string();
        let mut message_requests = vec![MessageRequest {
            role: MessageRole::System,
            content: system_prompt(name, Config::pwd(), date, custom_system_prompt),
        }];

        let start_messages_count = messages.len();
        for message in messages {
            message_requests.push(MessageRequest {
                role: message.role.into(),
                content: message.content,
            })
        }

        info!(name: "messages", "{:#?}", message_requests);
        Self {
            __start_messages_count: start_messages_count,
            session_repository,
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
        is_tool_execution_result: bool,
    ) -> Result<AskResponse, AiProviderError> {
        info!("Asking...");
        let role = if is_tool_execution_result {
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
                let response = match ask_response {
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
                };

                // @NOTE: Drop tool result and keep only the response
                if is_tool_execution_result {
                    self.messages.pop();
                }

                self.messages.push(MessageRequest {
                    role: MessageRole::Assistant,
                    content: response.clone(),
                });
                info!("Asked!");

                info!("Appending messages to conversation...");
                if !is_tool_execution_result {
                    append_to_conversation(
                        self.conversation_repository.clone(),
                        self.session_id.clone(),
                        MessageRole::User,
                        prompt,
                        false,
                    )
                    .await
                    .expect("Failed to append user message to conversation");
                }

                append_to_conversation(
                    self.conversation_repository.clone(),
                    self.session_id.clone(),
                    MessageRole::Assistant,
                    response,
                    false,
                )
                .await
                .expect("Failed to append assistent message to conversation");
                info!("Messages appended!");

                Ok(ask_response)
            }
            Err(error) => {
                self.messages.pop();
                Err(error)
            }
        }
    }

    pub async fn summarize(
        &mut self,
        agent: &Box<dyn AiProvider>,
        force: bool,
    ) -> Result<(), AiProviderError> {
        if force && self.__start_messages_count == self.messages.len() - 1 {
            return Ok(());
        }

        if self.messages.len() >= Config::max_context_memory_size() || force {
            info!("summarizing...");
            // @NOTE: clonning the message history so if something fail we don't
            // have drained the entire messages data and can retry properly
            let mut cloned_messages = self.messages.clone();
            let messages = cloned_messages.drain(1..);
            let messages_prompt = messages
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

            let summarized_phrase = agent
                .plain_ask(vec![
                    MessageRequest {
                        role: MessageRole::System,
                        content: summarize_phrase_prompt(),
                    },
                    MessageRequest {
                        role: MessageRole::User,
                        content: summarized_text.clone(),
                    },
                ])
                .await?;

            // @TODO: make this atomic
            // ---
            let message = format!("Conversation summary until now: {summarized_text}");
            append_to_conversation(
                self.conversation_repository.clone(),
                self.session_id.clone(),
                MessageRole::Assistant,
                message.clone(),
                true,
            )
            .await
            .expect("Failed to append summarized message to conversation");

            let session_id =
                uuid::Uuid::from_str(&self.session_id).expect("Invalid session id");
            self.session_repository
                .change_summary_phrase(session_id, summarized_phrase.clone())
                .await
                .expect("Failed to save session summary");
            // ---

            info!("summarized: {} | {}", summarized_phrase, summarized_text);

            self.messages = cloned_messages;
            self.messages.push(MessageRequest {
                role: MessageRole::Assistant,
                content: message,
            });
        }

        Ok(())
    }
}

fn summarize_phrase_prompt() -> String {
    load_file_installed("prompts/context/summarize_as_phrase.txt", vec![])
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
        vec![
            ("name", name),
            ("current_dir", current_dir),
            ("date", date),
            (
                "custom_prompt",
                custom_prompt.unwrap_or(DEFAULT_CUSTOM_SYSTEM_PROMPT.to_string()),
            ),
        ],
    )
}
