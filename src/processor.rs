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
// and command contexts
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
    __last_command_executed: Option<String>,
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
            __last_command_executed: None,
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
        let (response, commands) = self.process_prompt(task).await?;
        self.__sender
            .send(TaskResponse::MessageResponse(response))
            .await?;

        for command in commands {
            // @NOTE: this will prevent bugs when the agent try to run the same
            // command twice
            let is_the_same_command = self
                .__last_command_executed
                .clone()
                .is_some_and(|last_command| last_command == command);
            if is_the_same_command {
                return Ok(());
            }

            self.__sender
                .send(TaskResponse::CommandSignature(command.clone()))
                .await?;

            let output = self.cmd.exec(&command)?;
            let mut croped_output = output.clone();
            croped_output.truncate(MAX_COMMAND_OUTPUT_SIZE);

            self.__sender
                .send(TaskResponse::CommandResponse((
                    command.clone(),
                    croped_output,
                )))
                .await?;

            // @TODO: should crop the output size when too big and how much big?
            self.recursively_process_task(output).await?;
            self.__last_command_executed = Some(command);
        }

        Ok(())
    }

    async fn process_prompt(
        &mut self,
        input: String,
    ) -> Result<(String, Vec<String>), Error> {
        self.context.summarize(&self.agent, false).await?;

        let response = self.context.ask(&self.agent, input).await?;
        let re = regex::Regex::new(r"Command\((.*)\)").unwrap();
        let commands: Vec<String> = re
            .captures_iter(&response.content)
            .map(|cap| cap[1].to_string())
            .collect();

        Ok((response.content, commands))
    }
}
