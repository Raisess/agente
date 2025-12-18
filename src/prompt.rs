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
        "your actions should be based on the next described functions, you \
         shouldn’t use any of pre built tools you have, consider this tool \
         set: {tools_prompt}, return just like each tool described using this \
         format: {{ \"tool\": \"<ToolName>\", \"result\": \"<ToolResult>\" \
         }}, now determine what to do for the next prompt"
    )
}
