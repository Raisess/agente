use crate::adapters::util::load_file::load;

pub mod chat_gpt;

/// Loads tools to be passed to AI Providers implementations
pub fn load_tools() -> serde_json::Value {
    let tools = serde_json::from_str::<serde_json::Value>(
        &load("./tools.json", vec![]).unwrap_or_else(|_| {
            load(&format!("{}/.agente/tools.json", std::env!("HOME")), vec![])
                .expect("Failed to load tools.json file")
        }),
    )
    .expect("Failed to parse tools json");

    tools
}
