use std::collections::HashMap;
use std::sync::LazyLock;

use agente_domain::core::tool::Tool;
use agente_domain::ports::agent::MessageRequest;

const __CACHE: LazyLock<HashMap<String, String>> =
    LazyLock::new(|| HashMap::new());
const PROMPTS_FOLDER_PATH: &str = "__prompts";

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

    load("system", vec![("tools", tools_prompt)])
        .expect("Failed to load system prompt")
}

pub fn summarize_messages_prompt(messages: Vec<MessageRequest>) -> String {
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

fn load(
    name: &str,
    replace: Vec<(&str, String)>,
) -> Result<String, std::io::Error> {
    let path = format!("{PROMPTS_FOLDER_PATH}/{name}.md");
    let binding = __CACHE;
    let Some(content) = binding.get(&path) else {
        let mut content = std::fs::read_to_string(path)?;

        for (key, value) in replace {
            content = content.replace(&format!("{{{{{key}}}}}"), &value);
        }

        return Ok(content);
    };

    Ok(content.clone())
}
