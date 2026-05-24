use std::sync::Arc;

use clap::Parser;
use tracing_subscriber;
use tracing_subscriber::EnvFilter;

use agente::stdio::start_stdio;

use agente_application::core::context::Context;
use agente_application::core::processor::Processor;
use agente_application::core::{get_conversation, init_session};
use agente_application::repositories::conversation::ConversationRepository;
use agente_application::repositories::session::SessionRepository;
use agente_domain::ports::ai_provider::{AiProvider, AiProviderConfig};
use agente_infrastructure::adapters::database::sqlite::SqliteDatabase;
use agente_infrastructure::adapters::providers::openai::OpenAI;
use agente_infrastructure::adapters::util::file_system::FileSystem;
use agente_infrastructure::config::Config;

#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Args {
    /// Session ID
    #[arg(long)]
    session: Option<String>,
    /// AI Provider
    #[arg(long)]
    provider: Option<Provider>,
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

    let provider = args.provider.unwrap_or(Provider::OPENAI);
    let name = config.name.clone().unwrap_or("Agente".to_string());

    let agent = provider_factory(provider, &config);
    let context = Context::init(
        name.clone(),
        conversation_repository,
        session.id.to_string(),
        conversation,
    );
    let mut processor = Processor::init(agent, context);

    start_stdio(name, &session, &mut processor).await;
}

async fn setup() -> (
    Arc<Config>,
    Arc<SessionRepository>,
    Arc<ConversationRepository>,
) {
    // @FIXME: instead of recreating the entire config file, just append new keys.
    let fs = Arc::new(FileSystem::default());
    let config = match Config::load(fs.clone(), None) {
        Ok(c) => c,
        Err(_) => Config::setup_fallback(fs).expect(
            "Failed to load config.json on the current path and from \
             ~/.config/agente/config.json",
        ),
    };

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

#[derive(Debug, Clone)]
enum Provider {
    OPENAI,
    GROQ,
}

impl<'s> From<&'s str> for Provider {
    fn from(value: &'s str) -> Self {
        match value {
            "openai" => Provider::OPENAI,
            "groq" => Provider::GROQ,
            _ => panic!("Invalid provider option!"),
        }
    }
}

fn provider_factory(provider: Provider, config: &Config) -> Box<dyn AiProvider> {
    fn init_provider<F>(
        label: &str,
        config: Option<AiProviderConfig>,
        f: F,
    ) -> Box<dyn AiProvider>
    where
        F: Fn(AiProviderConfig) -> Box<dyn AiProvider>,
    {
        if config.is_none() {
            panic!("No {label} API config provided");
        }

        let c = config.unwrap();
        if c.model.is_empty() {
            panic!("No {label} API model provided");
        }
        if c.api_key.is_empty() {
            panic!("No {label} API key provided");
        }

        f(c)
    }

    match provider {
        Provider::OPENAI => init_provider("Open AI", config.openai.clone(), |c| {
            Box::new(OpenAI::new(c, None))
        }),
        Provider::GROQ => init_provider("Groq", config.groq.clone(), |c| {
            Box::new(OpenAI::new(
                c,
                Some("https://api.groq.com/openai/v1/chat/completions"),
            ))
        }),
    }
}
