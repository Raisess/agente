use std::sync::Arc;

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

    // @TODO: support select agent type
    if config.chat_gpt.api_key.is_empty() {
        panic!("No API Key provided");
    }

    let agent = ChatGPT::new(config.chat_gpt.clone());
    let mut processor = Processor::init(Box::new(agent));

    start_stdio(&mut processor).await;
}

/// Starts the stdio interface
async fn start_stdio(processor: &mut Processor) -> () {
    let listener = processor.listener();
    tokio::spawn(async move {
        let cloned_listener = listener.clone();
        let mut listener = cloned_listener.lock().await;
        while let Some(response) = listener.recv().await {
            match response {
                TaskResponse::Thinking => println!("Thinking..."),
                TaskResponse::MessageResponse(message) => {
                    println!("[◉‿◉] > Agente: {message}")
                }
                TaskResponse::CommandSignature(command) => {
                    println!("< Running({command})")
                }
                TaskResponse::CommandResponse((command, result)) => {
                    if result.is_empty() {
                        println!("> Resolved({command})")
                    } else {
                        println!("> Resolved({command}): {result}")
                    }
                }
                TaskResponse::Error(error) => eprintln!("> System: {error:#?}"),
            };
        }
    });

    use std::io::Write;
    fn draw_input() {
        print!("\r\x1b[K{}", "> Type something: ");
        std::io::stdout().flush().unwrap();
    }

    let banner = format!(
    "
┌───────────────────────────── \x1b[32mAGENTE\x1b[0m ─────────────────────────────┐
│  \x1b[32m[◉‿◉]\x1b[0m   > I'ready!                                              │
│ \x1b[32m/|   |\\\x1b[0m  Running at: localhost:{:<34}│
│ \x1b[32m |   |\x1b[0m   Dir: {:<51}│
│ \x1b[32m/ \\ / \\\x1b[0m                                                          │
└──────────────────────────────────────────────────────────────────┘
",
    "0000",
    Config::pwd()
);

    print!("{banner}\n");
    draw_input();

    loop {
        let mut prompt = String::new();
        std::io::stdin()
            .read_line(&mut prompt)
            .expect("Failed to read from stdin");
        if prompt.is_empty() {
            continue;
        }

        let _ = processor.handle(prompt).await;
    }
}

// @TODO: start websocket server interface
