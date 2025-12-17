use std::collections::HashMap;

use agente_domain::core::tool::Tool;

pub fn prompt(tools: &HashMap<&str, Box<dyn Tool>>) -> String {
    let tools_prompt = tools
        .iter()
        .map(|(name, tool)| {
            format!(
                "{name}(context: \"{}\", reponse format: \"{}\", usage: \
                 \"{}\")",
                tool.context(),
                tool.format_instruction().unwrap_or(""),
                tool.usage_instruction().unwrap_or("")
            )
        })
        .collect::<Vec<String>>()
        .join(", ");

    format!(
        "consider this tool set {tools_prompt}, now determine what to do for \
         the next prompt"
    )
}
