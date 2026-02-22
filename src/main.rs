use std::sync::Arc;

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

    let agent = ChatGPT::new(config.chat_gpt_api_key.clone());
    let mut processor = Processor::init(Box::new(agent), config.clone());

    // @TODO: load a file as a task
    let args = std::env::args().collect::<Vec<_>>().split_off(1);
    let input = args.get(0);
    if input.is_none() || input.unwrap().is_empty() {
        return ();
    }

    match processor.handle(input.unwrap().clone()).await {
        Ok(response) => println!("{response}"),
        Err(error) => eprintln!("{error:#?}"),
    }
}
