use std::sync::{Arc, Mutex};

use agente_domain::error::Error;
use agente_domain::ports::ai_provider::{MessageRequest, MessageRole};
use agente_infrastructure::config::Config;

use crate::core::processor::Processor;

const CONTEXT_CHECK_PROMPT_BASE: &str =
    "Based on this message list, was the user request successfully responded?";

// @TODO: this is a working progress feature that should improve the harness by
// verifying if the prompt was successfully processed or not and aplying a
// execution loop that executes until the user request is successfully
// fulfilled.
// @TODO: check input length and split it when it exceeds maximum
// allowed size (ps: use embed paragraphs)
pub async fn preprocess(
    processor: &mut Arc<Mutex<Processor>>,
    prompt: String,
) -> Result<(), Error> {
    let sanitized_prompt = sanitize_prompt(prompt);

    let mut p = processor.lock().unwrap();
    if Config::use_preprocessor() && !sanitized_prompt.starts_with("/") {
        let (start_offset, end_offset) = p.handle(sanitized_prompt.clone()).await?;
        let mut messages = p.context.messages.clone();
        let related_messages =
            messages.drain(start_offset..end_offset).collect::<Vec<_>>();
        let related_small_context_messages_summary = related_messages
            .iter()
            .map(|m| format!("{}: {}", m.role, m.content))
            .collect::<Vec<_>>()
            .join("|");

        println!(
            "related_small_context_messages_summary: {related_small_context_messages_summary:#?}"
        );
        let context_check_prompt = format!(
            "{CONTEXT_CHECK_PROMPT_BASE} {related_small_context_messages_summary}"
        );
        let response = p
            .agent
            .plain_ask(vec![MessageRequest {
                role: MessageRole::User,
                content: context_check_prompt,
            }])
            .await?;
        println!("Response: {response:#?}");
        p.allow_prompt().await?;
    } else {
        p.handle(sanitized_prompt).await?;
        p.allow_prompt().await?;
    }

    Ok(())
}

fn sanitize_prompt(prompt: String) -> String {
    prompt
        .split(" ")
        .map(|word| word.trim())
        .filter(|word| !word.is_empty())
        .collect::<Vec<_>>()
        .join(" ")
}
