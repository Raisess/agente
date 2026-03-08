use tokio::sync::mpsc;

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

    pub async fn run(&mut self) -> () {
        let (tx, mut rx) = mpsc::channel::<String>(1);
        tokio::spawn(async move {
            println!("> Type something:");

            loop {
                let mut input = String::new();
                std::io::stdin()
                    .read_line(&mut input)
                    .expect("Failed to read from stdin");
                tx.send(input)
                    .await
                    .expect("Failed to send input to main thread");
            }
        });

        // @TODO: process the prompt first divide it by tasks and the execute
        // each per time
        while let Some(prompt) = rx.recv().await {
            if prompt.is_empty() {
                continue;
            }

            self.recursive_handler(prompt).await;
        }
    }

    #[async_recursion::async_recursion]
    async fn recursive_handler(&mut self, prompt: String) -> () {
        println!("Thinking...");
        let result = self.process_prompt(prompt).await;
        match result {
            Ok((response, command)) => {
                println!("> Agente: {response}");
                // @TODO: should ask for permission before running the command
                if let Some(command_result) = command.map(|c| self.cmd.exec(&c))
                {
                    match command_result {
                        // @TODO: crop the output size when too big
                        Ok(output) => self.recursive_handler(output).await,
                        Err(error) => eprintln!("> System: {error:#?}"),
                    }
                }
            }
            Err(error) => eprintln!("> System: {error:#?}"),
        }
    }

    async fn process_prompt(
        &mut self,
        prompt: String,
    ) -> Result<(String, Option<String>), Error> {
        self.context.summarize(&self.agent).await?;

        let response = self.context.ask(&self.agent, prompt).await?;
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
