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
    __last_tool_executed: Option<String>,
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
            let is_the_same_tool = self
                .__last_tool_executed
                .clone()
                .is_some_and(|last_tool| last_tool == tool);
            if is_the_same_tool {
                return Err(Error::new(
                    "Was attempted to execute the exact same command twice",
                ));
            }

            self.__last_tool_executed = Some(tool.clone());
            self.__sender
                .send(TaskResponse::CommandSignature(tool.clone()))
                .await?;

            let (output, croped_output) = self.execute_tool(tool.clone())?;
            self.__sender
                .send(TaskResponse::CommandResponse((tool, croped_output)))
                .await?;

            // @TODO: should crop the output size when too big and how much big?
            self.recursively_process_task(output).await?;
        }

        Ok(())
    }

    async fn process_prompt(
        &mut self,
        input: String,
    ) -> Result<(String, Vec<String>), Error> {
        self.context.summarize(&self.agent, false).await?;

        let response = self.context.ask(&self.agent, input).await?;
        let re = regex::Regex::new(r"(?s)Tool\((.*?)\)").unwrap();
        let tools: Vec<String> = re
            .captures_iter(&response.content)
            .map(|cap| cap[1].to_string())
            .collect();

        Ok((response.content, tools))
    }

    fn execute_tool(&self, tool: String) -> Result<(String, String), Error> {
        let mut parameters = tool.split(" ").collect::<Vec<_>>();
        let arguments = parameters.drain(1..).collect::<Vec<_>>().join(" ");
        let output = self.cmd.exec(&format!(
            "python3 ./__tools/{}.py {}",
            parameters[0], arguments
        ))?;
        let mut croped_output = output.clone();
        croped_output.truncate(MAX_COMMAND_OUTPUT_SIZE);

        Ok((output, croped_output))
    }
}
