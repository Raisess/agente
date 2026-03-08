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

            // @TODO: recursively run the processor with command
            // result
            println!("Thinking...");
            let result = self.process_prompt(prompt).await;
            match result {
                Ok((response, command)) => {
                    println!("> Agente: {response}");
                    println!("Extracted command: {command:#?}");
                }
                Err(error) => eprintln!("> System: {error:#?}"),
            }
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
