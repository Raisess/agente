use std::sync::Arc;

use agente_domain::core::Error;
use agente_domain::core::tool::{Tool, ToolResponse};
use agente_domain::ports::io::Writer;

pub struct WriteTool {
    writer: Arc<dyn Writer>,
}

impl WriteTool {
    pub fn new(writer: Arc<dyn Writer>) -> Self {
        Self { writer }
    }
}

#[async_trait::async_trait]
impl Tool for WriteTool {
    async fn handle(
        &self,
        arguments: Vec<String>,
    ) -> Result<ToolResponse, Error> {
        let path = arguments
            .get(0)
            .expect("`path` must be provided as argument 0");
        let content = arguments
            .get(1)
            .expect("`content` must be provided as argument 1");
        // @NOTE: inferred content is a new argument passed from the execution
        // context.
        let inferred_content = arguments.get(2);

        let value = if inferred_content.is_some_and(|ic| !ic.is_empty()) {
            inferred_content.unwrap()
        } else {
            content
        };

        self.writer.write(path, value.as_bytes())?;
        Ok(ToolResponse {
            data: String::from("Writed to the file"),
            is_feedable: false,
        })
    }

    fn context(&self) -> &'static str {
        "this tool should be used only when a file need to be created, writed \
         or updated"
    }

    fn format_instruction(&self) -> Option<&'static str> {
        Some(
            "provide the file path as the result like this: [\"<file path>\", \
             \"<content to be writed must be infered by the context of the \
             last message, but only if the last message provided a clear \
             context of what should be write, if not, just put <NONE> here \
             instead.>\"]",
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

    struct FailWriterMock;

    impl FailWriterMock {
        fn new() -> Self {
            Self {}
        }
    }

    impl Writer for FailWriterMock {
        fn write(
            &self,
            _path: &str,
            _data: &[u8],
        ) -> Result<(), std::io::Error> {
            Err(std::io::ErrorKind::Other.into())
        }
    }

    #[tokio::test]
    async fn should_fail_to_write() {
        let writer = FailWriterMock::new();
        let tool = WriteTool::new(std::sync::Arc::new(writer));
        let result = tool
            .handle(vec![String::from("file_path"), String::from("content")])
            .await;
        assert_eq!(
            result.is_err_and(|err| { err.message() == "other error" }),
            true
        )
    }
}
