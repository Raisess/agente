use std::collections::HashMap;

use agente_domain::core::tool::Tool;

pub fn prompt(tools: &HashMap<&str, Box<dyn Tool>>) -> String {
    let tools_prompt = tools
        .iter()
        .map(|(name, tool)| {
            format!(
                "{name}(context: \"{}\", arguments format: \"{}\")",
                tool.context(),
                tool.format_instruction().unwrap_or("")
            )
        })
        .collect::<Vec<String>>()
        .join(", ");

    format!(
        "your actions should be based on the next described functions, you \
         shouldn’t use any of pre built tools you have, consider this tool \
         set: {tools_prompt}, when the prompt matches one of more tool \
         requirement return just like each tool described using this format: \
         [{{ \"tool\": \"<ToolName>\", \"summary\": \"<summarize what you \
         gonna do>\", \"arguments\": <ToolArguments> }}], now determine what \
         to do for the next prompt"
    )
}
