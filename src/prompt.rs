use std::collections::HashMap;

use agente_domain::core::tool::Tool;
use agente_domain::ports::agent::MessageRequest;

pub fn system_prompt(tools: &HashMap<String, Box<dyn Tool>>) -> String {
    let tools_prompt = tools
        .iter()
        .map(|(name, tool)| {
            format!(
                "{name}(context: \"{}\", arguments format: \"{}\")",
                tool.context(),
                tool.format_instruction().unwrap_or("[]")
            )
        })
        .collect::<Vec<_>>()
        .join(", ");

    format!(
        "you shouldn’t use any of pre built tools you have, every message \
         response should be based on the next described functions, only \
         consider this tool set: {tools_prompt}, when the prompt matches one \
         of more tool requirement return just like each tool described using \
         only this format: [{{ \"tool\": \"<ToolName>\", \"summary\": \
         \"<summarize what you gonna do>\", \"arguments\": <ToolArguments> \
         }}] always in plain json array and never using the markdown notation \
         and make sure the json is always valid, now determine what to do for \
         the next prompt"
    )
}

pub fn summarize_messages_prompt(messages: Vec<MessageRequest>) -> String {
    let messages_prompt = messages
        .iter()
        .map(|MessageRequest { role, content }| {
            format!("Role: {role}, Content: {content}")
        })
        .collect::<Vec<_>>()
        .join(", ");

    format!("write a summary for this message list: {messages_prompt}")
}
