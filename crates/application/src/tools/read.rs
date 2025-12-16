use std::io::Read;

use agente_domain::core::tool::Tool;

const CONTEXT: &str = "This tool should be used when a file need to be readed.";
const INSTRUCTION: &str = "@TODO";

pub struct ReadTool;

impl ReadTool {
    pub fn new() -> Self {
        Self {}
    }
}

impl Tool<String, std::io::Error> for ReadTool {
    async fn handle(&mut self, path: &str) -> Result<String, std::io::Error> {
        let mut file = std::fs::File::open(path)?;

        let mut data = String::new();
        file.read_to_string(&mut data)?;

        Ok(data)
    }

    fn context() -> &'static str {
        CONTEXT
    }

    fn instruction() -> &'static str {
        INSTRUCTION
    }
}
