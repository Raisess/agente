#[derive(Debug, Clone)]
pub struct ToolCall {
    pub name: String,
    pub arg: Option<String>,
    pub content: Option<String>,
}

impl ToolCall {
    pub fn to_command(&self) -> String {
        format!(
            "python3 ./__tools/{}.py {} {}",
            self.name,
            self.arg.clone().unwrap_or("".to_string()),
            self.content
                .clone()
                .map(|c| format!("<<EOF\n{c}\nEOF"))
                .unwrap_or("".to_string())
        )
    }
}

impl ToString for ToolCall {
    fn to_string(&self) -> String {
        format!(
            "{} {} {}",
            self.name,
            self.arg.clone().unwrap_or("".to_string()),
            self.content
                .clone()
                .unwrap_or("".to_string())
        )
    }
}

pub fn parse_tools(response_content: &str) -> Vec<ToolCall> {
    let mut tools = Vec::new();
    let re_tool = regex::Regex::new(r"(?m)^Tool:\s*(.*)").unwrap();

    let lines: Vec<&str> = response_content.lines().collect();
    let mut current_tool_start = None;

    for (i, line) in lines.iter().enumerate() {
        if re_tool.is_match(line) {
            if let Some(start) = current_tool_start {
                let block = &lines[start..i].join("\n");
                tools.push(parse_tool_block(block));
            }
            current_tool_start = Some(i);
        }
    }

    // push the last block if exists
    if let Some(start) = current_tool_start {
        let block = &lines[start..].join("\n");
        tools.push(parse_tool_block(block));
    }

    tools
}

fn parse_tool_block(block: &str) -> ToolCall {
    let mut lines: Vec<&str> = block.lines().collect();
    let first_line = lines.remove(0).trim();

    // Remove "Tool:" prefix
    let rest = first_line.trim_start_matches("Tool:").trim();

    // First word = tool name, rest = first argument (optional)
    let mut parts = rest.splitn(2, ' ');
    let name = parts.next().unwrap_or_default().to_string();
    let arg = parts.next().map(|s| s.to_string());

    let content = if lines.is_empty() {
        None
    } else {
        Some(lines.join("\n"))
    };

    ToolCall { name, arg, content }
}
