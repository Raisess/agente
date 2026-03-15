use std::sync::Arc;

use tokio::sync::Mutex;
use tokio::sync::mpsc::{Receiver, Sender};

use agente_domain::error::Error;
use agente_domain::ports::agent::Agent;
use agente_domain::ports::io::Executor;
use agente_infrastructure::adapters::cmd::CMD;

use crate::context::Context;

const MAX_COMMAND_OUTPUT_SIZE: usize = 2500;

// @TODO: link a message id, will be useful for websocket server to know message
// and tool contexts
pub enum TaskResponse {
    Thinking,
    MessageResponse(String),
    CommandSignature(String),
    CommandResponse((String, String)),
    Error(Error),
}

pub struct Processor {
    __receiver: Arc<Mutex<Receiver<TaskResponse>>>,
    __sender: Sender<TaskResponse>,
    __last_tool_executed: Option<ToolCall>,
    agent: Box<dyn Agent>,
    context: Context,
    cmd: CMD,
}

impl Processor {
    pub fn init(agent: Box<dyn Agent>) -> Self {
        let (tx, rx) = tokio::sync::mpsc::channel::<TaskResponse>(10);
        Self {
            __receiver: Arc::new(Mutex::new(rx)),
            __sender: tx,
            __last_tool_executed: None,
            agent,
            context: Context::init(),
            cmd: CMD::default(),
        }
    }

    pub fn listener(&self) -> Arc<Mutex<Receiver<TaskResponse>>> {
        self.__receiver.clone()
    }

    pub async fn handle(&mut self, prompt: String) -> Result<(), Error> {
        // @FIXME: removed the task planner since it was causing a lot of
        // problems generating too many things to do and removing the ability to
        // talk from the model.
        // let tasks_text =
        // self.context.generate_tasks(&self.agent, prompt).await?;
        // println!("{}", tasks_text.content);

        // let tasks_queue = tasks_text.content.split(";").map(|task|
        // task.trim());
        for task in vec![prompt] {
            match self.recursively_process_task(task.to_string()).await {
                Ok(_) => {}
                Err(error) => {
                    self.__sender.send(TaskResponse::Error(error)).await?;
                    break;
                }
            }
        }

        Ok(())
    }

    #[async_recursion::async_recursion]
    async fn recursively_process_task(
        &mut self,
        task: String,
    ) -> Result<(), Error> {
        if task.is_empty() {
            return Ok(());
        }

        self.__sender.send(TaskResponse::Thinking).await?;
        let (response, tools) = self.process_prompt(task).await?;
        self.__sender
            .send(TaskResponse::MessageResponse(response))
            .await?;

        for tool in tools {
            // @NOTE: this will prevent bugs when the agent try to run the same
            // tool twice
            let is_the_same_tool =
                self.__last_tool_executed.clone().is_some_and(|last_tool| {
                    last_tool.to_string() == tool.to_string()
                });
            if is_the_same_tool {
                return Err(Error::new(
                    "Was attempted to execute the exact same command twice",
                ));
            }

            self.__last_tool_executed = Some(tool.clone());
            self.__sender
                .send(TaskResponse::CommandSignature(tool.to_string()))
                .await?;

            let (output, croped_output) = self.execute_tool(tool.clone())?;
            self.__sender
                .send(TaskResponse::CommandResponse((
                    tool.to_string(),
                    croped_output,
                )))
                .await?;

            // @TODO: should crop the output size when too big and how much big?
            self.recursively_process_task(output).await?;
        }

        Ok(())
    }

    async fn process_prompt(
        &mut self,
        input: String,
    ) -> Result<(String, Vec<ToolCall>), Error> {
        self.context.summarize(&self.agent, false).await?;

        let response = self.context.ask(&self.agent, input).await?;
        let tools = parse_tools(&response.content);

        Ok((response.content, tools))
    }

    fn execute_tool(&self, tool: ToolCall) -> Result<(String, String), Error> {
        println!("{tool:#?}");
        // @NOTE: execute a plain linux command if the tool don't match
        // const TOOLS: [&str; 3] = ["write", "read", "explore"];

        let output = self.cmd.exec(&tool.to_string())?;
        let mut croped_output = output.clone();
        croped_output.truncate(MAX_COMMAND_OUTPUT_SIZE);

        Ok((output, croped_output))
    }
}

#[derive(Debug, Clone)]
struct ToolCall {
    pub name: String,
    pub arg: Option<String>,
    pub content: Option<String>,
}

impl ToString for ToolCall {
    fn to_string(&self) -> String {
        format!(
            "python3 ./__tools/{}.py {} {}",
            self.name,
            self.arg.clone().unwrap_or("".to_string()),
            self.content.clone().map(|c| format!("\"{c}\"")).unwrap_or("".to_string())
        )
    }
}

fn parse_tools(response_content: &str) -> Vec<ToolCall> {
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
