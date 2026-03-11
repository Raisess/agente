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

    // @TODO: to process the prompt first divide it by tasks and the execute
    // each per time
    pub async fn handle(&mut self, prompt: String) -> () {
        self.process_prompt_task(prompt).await;
    }

    // @TODO: this should be a streamable result for the caller
    // @FIXME: move the println calls to the stdio interface after having the
    // streamable result
    #[async_recursion::async_recursion]
    async fn process_prompt_task(&mut self, task: String) -> () {
        println!("Thinking...");
        let result = self.process_input(task).await;
        match result {
            Ok((response, command)) => {
                println!("> Agente: {response}");
                // @TODO: should ask for permission before running the command
                if let Some(command_result) = command.map(|c| self.cmd.exec(&c))
                {
                    match command_result {
                        // @TODO: crop the output size when too big
                        Ok(output) => self.process_prompt_task(output).await,
                        Err(error) => eprintln!("> System: {error:#?}"),
                    }
                }
            }
            Err(error) => eprintln!("> System: {error:#?}"),
        }
    }

    async fn process_input(
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
