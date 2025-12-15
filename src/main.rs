use agente_domain::tool::Tool;

struct TestTool;

impl TestTool {
    fn new() -> Self {
        Self {}
    }
}

impl Tool<String, TestToolError> for TestTool {
    async fn handle(&self) -> Result<String, TestToolError> {
        Ok(String::from("Hello, handler!"))
    }

    fn context() -> &'static str {
        "Hello, context!"
    }
}

#[derive(Debug)]
struct TestToolError;

impl std::error::Error for TestToolError {}

impl std::fmt::Display for TestToolError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("TestToolError")
    }
}

#[tokio::main]
async fn main() {
    let test_tool = TestTool::new();
    match test_tool.handle().await {
        Ok(result) => println!("{result:#?}"),
        Err(err) => eprint!("{err}"),
    }
}
