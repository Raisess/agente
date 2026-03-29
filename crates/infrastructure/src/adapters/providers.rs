use crate::adapters::util::load_file::load;

pub mod chat_gpt;

/// Loads tools and custom tools to be passed to AI Providers implementations
pub fn load_and_merge_tools() -> serde_json::Value {
    let mut tools = serde_json::from_str::<serde_json::Value>(
        &load("./tools.json", vec![]).unwrap_or_else(|_| {
            load(&format!("{}/.agente/tools.json", std::env!("HOME")), vec![])
                .expect("Failed to load tools.json file")
        }),
    )
    .expect("Failed to parse tools json");

    let mut custom_tools = serde_json::from_str::<serde_json::Value>(
        &load("./custom_tools.json", vec![])
            .unwrap_or_else(|_| "[]".to_string()),
    )
    .expect("Failed to load custom tools");

    tools
        .as_array_mut()
        .unwrap()
        .append(custom_tools.as_array_mut().unwrap());

    tools
}
