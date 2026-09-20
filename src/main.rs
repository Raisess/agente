use std::sync::{Arc, Mutex};

use clap::Parser;
use tracing_subscriber;
use tracing_subscriber::EnvFilter;

use agente::stdio::start_stdio;

use agente_application::core::context::Context;
use agente_application::core::processor::Processor;
use agente_application::core::{
    get_conversation, init_session, list_sessions_per_users_and_directory,
};
use agente_application::repositories::conversation::ConversationRepository;
use agente_application::repositories::session::SessionRepository;
use agente_domain::ports::ai_provider::{AiProvider, AiProviderConfig};
use agente_infrastructure::adapters::database::sqlite::SqliteDatabase;
use agente_infrastructure::adapters::providers::generic::GenericAiProvider;
use agente_infrastructure::adapters::util::file_system::FileSystem;
use agente_infrastructure::config::{Config, ConfigBuilder};

#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Args {
    /// Session ID - Use to return to a previous conversation session.
    #[arg(long)]
    session: Option<String>,
    /// AI Provider - Use to select the desired AI provider: openai, groq.
    #[arg(long)]
    provider: Option<Provider>,
    /// Custom system prompt - Use to set a custom system prompt for the agent.
    #[arg(long)]
    system: Option<String>,
    /// Custom name - Use to set a custom agent name.
    #[arg(long)]
    name: Option<String>,
    /// List sessions - list the available sessions for the directory.
    #[arg(short, long)]
    ls: bool,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let (config, session_repository, conversation_repository) = setup().await;
    let args = Args::parse();
    if args.ls == true {
        print_sessions_ls_command(session_repository.clone()).await;
        return;
    }

    let session = init_session(session_repository.clone(), args.session)
        .await
        .expect("Failed to init session");
    let conversation =
        get_conversation(conversation_repository.clone(), session.id.to_string())
            .await
            .expect("Failed to load conversation");

    let provider = args.provider.unwrap_or(Provider::OPENAI);
    let name = args
        .name
        .unwrap_or(config.name.clone().unwrap_or("Agente".to_string()));
    let current_session_id = session.id.to_string();
    let custom_system_prompt = args.system;

    let agent = provider_factory(provider, &config);
    let context = Context::init(
        session_repository,
        conversation_repository,
        name.clone(),
        current_session_id,
        conversation,
        custom_system_prompt,
    );
    let mut processor = Arc::new(Mutex::new(Processor::init(agent, context)));

    start_stdio(name, &session, &mut processor).await;
}

async fn setup() -> (
    Arc<Config>,
    Arc<SessionRepository>,
    Arc<ConversationRepository>,
) {
    // @FIXME: instead of recreating the entire config file, just append new
    // keys.
    let fs = Arc::new(FileSystem::default());
    let config = match Config::load(fs.clone(), None) {
        Ok(c) => c,
        Err(_) => {
            let setup_config = live_setup();
            Config::setup_fallback(fs, Some(setup_config)).expect(
                "Failed to load config.json on the current path and from \
               ~/.config/agente/config.json",
            )
        }
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

fn live_setup() -> Config {
    use agente_infrastructure::adapters::util::readline::Readline;

    println!("Welcome to the agente setup helper, let's start!\n");

    let mut rl = Readline::new();
    let provider_input = rl.read("Provider Select:\n\n1. OpenAI\n2. OPENROUTER\n3. Groq\n\nOptions: [1, 2, 3] > ").expect("Failed to read provider input");
    let provider = match provider_input.as_str() {
        "1" => Provider::OPENAI,
        "2" => Provider::OPENROUTER,
        "3" => Provider::GROQ,
        _ => panic!("Invalid provider option"),
    };

    let api_key_input = rl
        .read("Provide the api key > ")
        .expect("Failed to read api key input");
    let model_input = rl
        .read("Provide the model > ")
        .expect("Failed to read model input");
    let cheap_model_input = rl
        .read("Provide the cheap model (blank to use the same as model) > ")
        .expect("Failed to read cheap model input");
    let name_input = rl
        .read("Provide your agent name > ")
        .expect("Failed to read agent name");

    let mut builder = ConfigBuilder::default();
    builder
        .name(name_input)
        .provider(provider.to_string())
        .api_key(api_key_input)
        .model(model_input)
        .cheap_model(if cheap_model_input != "" {
            Some(cheap_model_input)
        } else {
            None
        })
        .build()
}

#[derive(Debug, Clone)]
enum Provider {
    OPENAI,
    OPENROUTER,
    GROQ,
}

impl<'s> From<&'s str> for Provider {
    fn from(value: &'s str) -> Self {
        match value {
            "openai" => Provider::OPENAI,
            "openrouter" => Provider::OPENROUTER,
            "groq" => Provider::GROQ,
            _ => panic!("Invalid provider option!"),
        }
    }
}

impl ToString for Provider {
    fn to_string(&self) -> String {
        String::from(match self {
            Provider::OPENAI => "openai",
            Provider::OPENROUTER => "openrouter",
            Provider::GROQ => "groq",
        })
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
        Provider::OPENAI => init_provider("Open AI", config.openai.clone(), |config| {
            Box::new(GenericAiProvider::new(
                config,
                "https://api.openai.com/v1/responses",
            ))
        }),
        // @TODO: change to openrouter specific config
        Provider::OPENROUTER => {
            init_provider("OpenRouter", config.openai.clone(), |config| {
                Box::new(GenericAiProvider::new(
                    config,
                    "https://openrouter.ai/api/v1/responses",
                ))
            })
        }
        Provider::GROQ => init_provider("Groq", config.groq.clone(), |config| {
            Box::new(GenericAiProvider::new(
                config,
                "https://api.groq.com/openai/v1/responses",
            ))
        }),
    }
}

async fn print_sessions_ls_command(session_repository: Arc<SessionRepository>) {
    let sessions = list_sessions_per_users_and_directory(session_repository)
        .await
        .expect("Failed to list available sessions");
    let text_to_print = sessions
        .iter()
        .map(|session| {
            format!(
                "- **ID:** {}\n  - **Content:** `{}`\n  - **Last update:** {}",
                session.id.to_string(),
                session
                    .summary_phrase
                    .clone()
                    .unwrap_or("No content yet".to_string()),
                session.updated_at.to_string(),
            )
        })
        .collect::<Vec<_>>()
        .join("\n");
    termimad::print_text(&text_to_print);
}
