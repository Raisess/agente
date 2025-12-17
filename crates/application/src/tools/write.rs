use std::sync::Arc;

use agente_domain::core::tool::Tool;
use agente_domain::ports::io::Writer;

pub struct WriteTool {
    writer: Arc<dyn Writer>,
}

impl WriteTool {
    pub fn new(writer: Arc<dyn Writer>) -> Self {
        Self { writer }
    }
}

impl Tool<(), std::io::Error> for WriteTool {
    async fn handle(
        &self,
        arguments: Vec<String>,
    ) -> Result<(), std::io::Error> {
        let path = arguments
            .get(0)
            .expect("`path` must be provided as argument 0");
        let content = arguments
            .get(1)
            .expect("`content` must be provided as argument 1");

        self.writer.write(path, content.as_bytes())?;
        Ok(())
    }

    fn context(&self) -> &'static str {
        "this tool should be used when a file need to created, writed or \
         updated"
    }

    fn format_instruction(&self) -> Option<&'static str> {
        Some(
            "provide the file path as the result like this: { \"path\": \
             \"<file path>\", \"content\": \"<content to writed>\" }",
        )
    }

    fn usage_instruction(&self) -> Option<&'static str> {
        None
    }
}

#[cfg(test)]
mod tests {
    use agente_domain::core::tool::Tool;
    use agente_domain::ports::io::Writer;

    use super::*;

    #[derive(Default)]
    struct SuccessWriterMock;

    impl Writer for SuccessWriterMock {
        fn write(
            &self,
            _path: &str,
            _data: &[u8],
        ) -> Result<(), std::io::Error> {
            Ok(())
        }
    }

    #[tokio::test]
    async fn should_write_successfully() {
        let writer = SuccessWriterMock::default();
        let tool = WriteTool::new(std::sync::Arc::new(writer));
        let result = tool
            .handle(vec![String::from("path"), String::from("content")])
            .await;
        assert_eq!(result.is_ok(), true);
    }
}
