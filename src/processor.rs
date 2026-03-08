use tokio::sync::mpsc;

use agente_domain::error::Error;
use agente_domain::ports::agent::Agent;

use crate::context::Context;

pub struct Processor {
    agent: Box<dyn Agent>,
    context: Context,
}

impl Processor {
    pub fn init(agent: Box<dyn Agent>) -> Self {
        Self {
            agent,
            context: Context::init(),
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

        while let Some(prompt) = rx.recv().await {
            if prompt.is_empty() {
                continue;
            }

            println!("Thinking...");
            match self.handle(prompt).await {
                Ok(response) => {
                    println!("> Agente: {response}");

                    let re = regex::Regex::new(r"Command\((.*)\)").unwrap();
                    if let Some(captured) = re.captures(&response) {
                        // @TODO: handle multiple commands, store then as tasks and
                        // process the entire recursion for each one and the drain
                        // the task vector
                        let command = captured.get(1).unwrap().as_str();
                        println!("Extracted command: {}", command);
                        // @TODO: recursively run the processor with command result
                    }
                }
                Err(error) => eprintln!("> System: {error:#?}"),
            }
        }
    }

    pub async fn handle(&mut self, input: String) -> Result<String, Error> {
        self.context.summarize(&self.agent).await?;

        let ask_response = self.context.ask(&self.agent, input).await?;
        Ok(ask_response.content)
    }
}
