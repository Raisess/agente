use std::sync::Arc;

use agente::stdio::start_stdio;
use clap::Parser;
use tracing_subscriber;
use tracing_subscriber::EnvFilter;

use agente_application::core::context::Context;
use agente_application::core::processor::Processor;
use agente_application::core::{get_conversation, init_session};
use agente_application::repositories::conversation::ConversationRepository;
use agente_application::repositories::session::SessionRepository;
use agente_infrastructure::adapters::database::sqlite::SqliteDatabase;
use agente_infrastructure::adapters::providers::chat_gpt::ChatGPT;
use agente_infrastructure::adapters::util::file_system::FileSystem;
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

    let (config, session_repository, conversation_repository) = setup().await;
    let args = Args::parse();
    let session = init_session(session_repository, args.session)
        .await
        .expect("Failed to init session");
    let conversation =
        get_conversation(conversation_repository.clone(), session.id.to_string())
            .await
            .expect("Failed to load conversation");

    let name = config.name.clone().unwrap_or("Agente".to_string());

    let agent = ChatGPT::new(config.chat_gpt.clone());
    let context = Context::init(
        name.clone(),
        conversation_repository,
        session.id.to_string(),
        conversation,
    );
    let mut processor = Processor::init(Box::new(agent), context);

    start_stdio(name, &session, &mut processor).await;
}

async fn setup() -> (
    Arc<Config>,
    Arc<SessionRepository>,
    Arc<ConversationRepository>,
) {
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

    let sqlite = Arc::new(
        SqliteDatabase::new(&Config::db_file())
            .await
            .expect("Failed to initialize sqlite database"),
    );
    let session_repository = Arc::new(
        SessionRepository::new(sqlite.clone())
            .await
            .expect("Failed to setup session repository"),
    );
    let conversation_repository = Arc::new(
        ConversationRepository::new(sqlite)
            .await
            .expect("Failed to setup conversation repository"),
    );

    (config, session_repository, conversation_repository)
}
