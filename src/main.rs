use std::sync::Arc;

use tokio::sync::mpsc;
use tracing_subscriber;
use tracing_subscriber::EnvFilter;

use agente::processor::Processor;
use agente_infrastructure::adapters::agents::chat_gpt::ChatGPT;
// use agente_infrastructure::adapters::cmd::CMD;
use agente_infrastructure::adapters::file_system::FileSystem;
use agente_infrastructure::config::Config;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let fs = Arc::new(FileSystem::default());
    // let cmd = Arc::new(CMD::default());

    let config = match Config::load(fs.clone(), None) {
        Ok(c) => c,
        Err(_) => Config::setup_fallback(fs).expect(
            "Failed to load config.json on the current path and from \
             ~/.config/agente/config.json",
        ),
    };

    let agent = ChatGPT::new(config.chat_gpt.clone());
    let mut processor = Processor::init(Box::new(agent), config.clone());

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
        match processor.handle(prompt).await {
            Ok(response) => {
                println!("> Agente: {response}");

                let re = regex::Regex::new(r"Command\((.*)\)").unwrap();
                if let Some(captured) = re.captures(&response) {
                    let command = captured.get(1).unwrap().as_str();
                    println!("Extracted command: {}", command);
                    // @TODO: recursively run the processor with command result
                }
            },
            Err(error) => eprintln!("> System: {error:#?}"),
        }
    }
}
