use std::collections::HashMap;
use std::sync::Arc;

use tokio::sync::Mutex;
use tracing_subscriber;
use tracing_subscriber::EnvFilter;

use agente::gui::GUI;
use agente::processor::Processor;
use agente_application::commands::exit::ExitCommand;
use agente_domain::core::command::Command;
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

    let config = Config::load(fs, None).expect("Failed to load config");

    let exit_command = ExitCommand::default();
    let mut commands = HashMap::<String, Box<dyn Command>>::new();
    commands.insert(exit_command.name().into(), Box::new(exit_command));

    let agent = ChatGPT::new(config.chat_gpt_api_key.clone());
    let processor = Processor::init(Box::new(agent), config.clone(), commands);
    GUI::run(config, Arc::new(Mutex::new(processor)))
        .expect("Failed to start gui application");
}
