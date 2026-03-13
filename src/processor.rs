use std::sync::Arc;

use tokio::sync::Mutex;
use tokio::sync::mpsc::{Receiver, Sender};

use agente_domain::error::Error;
use agente_domain::ports::agent::Agent;
use agente_domain::ports::io::Executor;
use agente_infrastructure::adapters::cmd::CMD;

use crate::context::Context;

pub enum TaskResponse {
    Thinking,
    MessageResponse(String),
    CommandResponse(String),
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
        let (response, command) = self.process_prompt(task).await?;
        self.__sender
            .send(TaskResponse::MessageResponse(response))
            .await?;

        // @TODO: should ask for permission before running the command
        if let Some(command) = command {
            self.__sender
                .send(TaskResponse::CommandResponse(command.clone()))
                .await?;

            // @TODO: crop the output size when too big
            let output = self.cmd.exec(&command)?;
            return Ok(self.recursively_process_task(output).await?);
        }

        Ok(())
    }

    // @FIXME: should support return more than one command
    async fn process_prompt(
        &mut self,
        input: String,
    ) -> Result<(String, Option<String>), Error> {
        self.context.summarize(&self.agent).await?;

        let response = self.context.ask(&self.agent, input).await?;
        let re = regex::Regex::new(r"Command\((.*)\)").unwrap();
        if let Some(captured) = re.captures(&response.content.clone()) {
            //println!("{captured:#?}");
            return Ok((
                response.content,
                Some(captured.get(1).unwrap().as_str().to_string()),
            ));
        }

        Ok((response.content, None))
    }
}
