use std::sync::Arc;

use agente_domain::core::tool::Tool;
use agente_domain::ports::io::Reader;

const CONTEXT: &str = "This tool should be used when a file need to be readed.";
const INSTRUCTION: &str = "@TODO";

pub struct ReadTool {
    reader: Arc<dyn Reader>,
}

impl ReadTool {
    pub fn new(reader: Arc<dyn Reader>) -> Self {
        Self { reader }
    }
}

impl Tool<String, std::io::Error> for ReadTool {
    async fn handle(&self, path: &str) -> Result<String, std::io::Error> {
        let content = self.reader.read(path)?;
        Ok(content)
    }

    fn context() -> &'static str {
        CONTEXT
    }

    fn instruction() -> &'static str {
        INSTRUCTION
    }
}

#[cfg(test)]
mod tests {
    use agente_domain::core::tool::Tool;
    use agente_domain::ports::io::Reader;

    use super::*;

    struct SuccessReaderMock {
        content: String,
    }

    impl SuccessReaderMock {
        fn new() -> Self {
            Self {
                content: String::from("some text"),
            }
        }
    }

    impl Reader for SuccessReaderMock {
        fn read(&self, path: &str) -> Result<String, std::io::Error> {
            Ok(format!("{path}:{}", self.content.clone()))
        }
    }

    #[tokio::test]
    async fn should_read_a_string_and_return() {
        let reader = SuccessReaderMock::new();
        let tool = ReadTool::new(std::sync::Arc::new(reader));
        let result = tool.handle("file_path").await.expect("Failed to read");
        assert_eq!(result, "file_path:some text");
    }

    struct FailReaderMock;

    impl FailReaderMock {
        fn new() -> Self {
            Self {}
        }
    }

    impl Reader for FailReaderMock {
        fn read(&self, _path: &str) -> Result<String, std::io::Error> {
            Err(std::io::ErrorKind::Other.into())
        }
    }

    #[tokio::test]
    async fn should_fail_to_read() {
        let reader = FailReaderMock::new();
        let tool = ReadTool::new(std::sync::Arc::new(reader));
        let result = tool.handle("file_path").await;
        assert_eq!(
            result.is_err_and(|err| err.kind() == std::io::ErrorKind::Other),
            true
        )
    }
}
