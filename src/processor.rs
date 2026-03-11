use agente_domain::error::Error;
use agente_domain::ports::agent::Agent;
use agente_domain::ports::io::Executor;
use agente_infrastructure::adapters::cmd::CMD;

use crate::context::Context;

pub struct Processor {
    agent: Box<dyn Agent>,
    context: Context,
    cmd: CMD,
}

impl Processor {
    pub fn init(agent: Box<dyn Agent>) -> Self {
        Self {
            agent,
            context: Context::init(),
            cmd: CMD::default(),
        }
    }

    pub async fn handle(&mut self, prompt: String) -> Result<(), Error> {
        let tasks = self.context.generate_tasks(&self.agent, prompt).await?;
        // println!("{}", tasks.content);

        let tasks =
            tasks.content.split(";").into_iter().map(|task| task.trim());
        for task in tasks {
            self.recursively_process_task(task.to_string()).await;
        }

        Ok(())
    }

    // @TODO: this should be a streamable result for the caller
    // @FIXME: move the println calls to the stdio interface after having the
    // streamable result
    #[async_recursion::async_recursion]
    async fn recursively_process_task(&mut self, task: String) -> () {
        let result = self.process_prompt(task).await;
        match result {
            Ok((response, command)) => {
                println!("> Agente: {response}");
                // @TODO: should ask for permission before running the command
                if let Some(command_result) = command.map(|c| {
                    println!("> Running({c})");
                    self.cmd.exec(&c)
                }) {
                    match command_result {
                        // @TODO: crop the output size when too big
                        Ok(output) => {
                            self.recursively_process_task(output).await
                        }
                        Err(error) => eprintln!("> System: {error:#?}"),
                    }
                }
            }
            Err(error) => eprintln!("> System: {error:#?}"),
        }
    }

    async fn process_prompt(
        &mut self,
        input: String,
    ) -> Result<(String, Option<String>), Error> {
        self.context.summarize(&self.agent).await?;

        let response = self.context.ask(&self.agent, input).await?;
        let re = regex::Regex::new(r"Command\((.*)\)").unwrap();
        if let Some(captured) = re.captures(&response.content.clone()) {
            return Ok((
                response.content,
                Some(captured.get(1).unwrap().as_str().to_string()),
            ));
        }

        Ok((response.content, None))
    }
}
