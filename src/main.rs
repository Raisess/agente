use std::sync::Arc;

use clap::Parser;
use tracing_subscriber;
use tracing_subscriber::EnvFilter;

use agente::processor::{Processor, TaskResponse};
use agente_application::repositories::session::SessionRepository;
use agente_domain::models::session::Session;
use agente_infrastructure::adapters::database::sqlite::SqliteDatabase;
use agente_infrastructure::adapters::file_system::FileSystem;
use agente_infrastructure::adapters::providers::chat_gpt::ChatGPT;
use agente_infrastructure::config::Config;

#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Args {
    /// Session ID
    #[arg(long)]
    session: Option<String>,
}

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

    // @TODO: support select provider
    if config.chat_gpt.api_key.is_empty() {
        panic!("No API Key provided");
    }

    let sqlite = Box::new(
        SqliteDatabase::new("main.db")
            .await
            .expect("Failed to initialize sqlite database"),
    );
    let session_repository = Arc::new(SessionRepository::new(sqlite));
    session_repository
        .setup()
        .await
        .expect("Failed to setup session repository");

    let args = Args::parse();
    let session = match args.session {
        Some(session_id) => session_repository
            .find_by_id(session_id)
            .await
            .expect("Failed to find session on database"),
        None => {
            let session = Session::new(Config::pwd());
            session_repository
                .create(&session)
                .await
                .expect("Failed to store session into database");

            Some(session)
        }
    };

    if session.is_none() {
        panic!("Invalid session id!");
    }

    let agent = ChatGPT::new(config.chat_gpt.clone());
    let mut processor = Processor::init(Box::new(agent));

    start_stdio(&session.unwrap(), &mut processor).await;
}

/// Starts the stdio interface
async fn start_stdio(session: &Session, processor: &mut Processor) -> () {
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
│ \x1b[32m/|   |\\\x1b[0m  Session: {}           │
│ \x1b[32m |   |\x1b[0m   Running at: http://localhost:{:<27}│
│ \x1b[32m/ \\ / \\\x1b[0m  Working dir: {:<43}│
└──────────────────────────────────────────────────────────────────┘
",
    session.id,
    Config::port(),
    Config::pwd(),
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

        let _ = processor.handle(prompt.trim().to_string()).await;
    }
}

// @TODO: start websocket server interface
