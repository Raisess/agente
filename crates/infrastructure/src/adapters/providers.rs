use crate::adapters::util::load_file_installed::load_file_installed;

pub mod chat_gpt;

/// Loads tools to be passed to AI Providers implementations
pub fn load_tools() -> serde_json::Value {
    let tools = serde_json::from_str::<serde_json::Value>(&load_file_installed(
        "tools.json",
        vec![],
    ))
    .expect("Failed to parse tools json");

    tools
}
