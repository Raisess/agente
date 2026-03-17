use std::collections::HashMap;
use std::sync::Arc;

use tokio::sync::Mutex;
use tokio::sync::mpsc::{Receiver, Sender};

use agente_domain::error::Error;
use agente_domain::ports::agent::{Agent, AskResponse};
use agente_domain::ports::io::{Executor, ExecutorArgument};
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
            agent,
            context: Context::init(),
            cmd: CMD::default(),
        }
    }

    pub fn listener(&self) -> Arc<Mutex<Receiver<TaskResponse>>> {
        self.__receiver.clone()
    }

    pub async fn handle(&mut self, prompt: String) -> Result<(), Error> {
        match self.recursively_process_task(prompt).await {
            Ok(_) => {}
            Err(error) => {
                self.__sender.send(TaskResponse::Error(error)).await?;
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
        let response = self.process_prompt(task).await?;
        match response {
            AskResponse::Content(text) => {
                self.__sender
                    .send(TaskResponse::MessageResponse(text))
                    .await?;
            }
            AskResponse::ToolCall(tools) => {
                for (tool, arguments) in tools {
                    self.__sender
                        .send(TaskResponse::CommandSignature(tool.to_string()))
                        .await?;

                    let (output, croped_output) =
                        self.execute_tool(tool.clone(), arguments.into())?;
                    self.__sender
                        .send(TaskResponse::CommandResponse((
                            tool,
                            croped_output,
                        )))
                        .await?;

                    // @TODO: should crop the output size when too big and how
                    // much big?
                    self.recursively_process_task(output).await?;
                }
            }
        }

        Ok(())
    }

    async fn process_prompt(
        &mut self,
        input: String,
    ) -> Result<AskResponse, Error> {
        self.context.summarize(&self.agent, false).await?;

        let response = self.context.ask(&self.agent, input).await?;
        Ok(response)
    }

    fn execute_tool(
        &self,
        tool: String,
        arguments: HashMap<String, String>,
    ) -> Result<(String, String), Error> {
        let mut script =
            vec![ExecutorArgument::Arg(format!("./__tools/{tool}.py"))];
        let mut flags = arguments
            .iter()
            .map(|(key, value)| {
                ExecutorArgument::Flag((key.clone(), value.clone()))
            })
            .collect::<Vec<_>>();

        script.append(&mut flags);
        let output = self.cmd.exec("python3", script)?;
        let mut croped_output = output.clone();
        croped_output.truncate(MAX_COMMAND_OUTPUT_SIZE);

        Ok((output, croped_output))
    }
}
