use std::sync::Arc;

use tokio::sync::mpsc;
use tracing_subscriber;
use tracing_subscriber::EnvFilter;

use agente::processor::{Processor, TaskResponse};
use agente_infrastructure::adapters::agents::chat_gpt::ChatGPT;
use agente_infrastructure::adapters::file_system::FileSystem;
use agente_infrastructure::config::Config;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let fs = Arc::new(FileSystem::default());
    let config = match Config::load(fs.clone(), None) {
        Ok(c) => c,
        Err(_) => Config::setup_fallback(fs).expect(
            "Failed to load config.json on the current path and from \
             ~/.config/agente/config.json",
        ),
    };

    // @TODO: support select agent
    if config.chat_gpt.api_key.is_empty() {
        panic!("No API Key provided");
    }

    let agent = ChatGPT::new(config.chat_gpt.clone());
    let mut processor = Processor::init(Box::new(agent));

    start_stdio(&mut processor).await;
}

/// Starts the stdio interface
async fn start_stdio(processor: &mut Processor) -> () {
    const BUFFER_SIZE: usize = 10;

    let (tx, mut rx) = mpsc::channel::<String>(BUFFER_SIZE);
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

    let listener = processor.listener();
    tokio::spawn(async move {
        let cloned_listener = listener.clone();
        let mut listener = cloned_listener.lock().await;
        while let Some(response) = listener.recv().await {
            match response {
                TaskResponse::Thinking => println!("Thinking..."),
                TaskResponse::MessageResponse(message) => {
                    println!("> Agente: {message}")
                }
                TaskResponse::CommandResponse(command) => {
                    println!("< Running({command})")
                }
                TaskResponse::Error(error) => eprintln!("> System: {error:#?}"),
            }
        }
    });

    while let Some(prompt) = rx.recv().await {
        if prompt.is_empty() {
            continue;
        }

        let _ = processor.handle(prompt).await;
    }
}

// @TODO: start websocket server interface
