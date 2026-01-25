use std::collections::HashMap;
use std::sync::Arc;

use tokio::sync::Mutex;
use tracing_subscriber;
use tracing_subscriber::EnvFilter;

use agente::gui::GUI;
use agente::processor::Processor;
use agente_application::commands::exit::ExitCommand;
use agente_application::tools::bash::BashTool;
use agente_application::tools::read::ReadTool;
use agente_application::tools::talk::TalkTool;
use agente_application::tools::write::WriteTool;
use agente_domain::core::command::Command;
use agente_domain::core::tool::Tool;
use agente_infrastructure::adapters::agents::chat_gpt::ChatGPT;
use agente_infrastructure::adapters::cmd::CMD;
use agente_infrastructure::adapters::file_system::FileSystem;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let fs = Arc::new(FileSystem::default());
    let cmd = Arc::new(CMD::default());

    let mut tools = HashMap::<String, Box<dyn Tool>>::new();
    tools.insert("Bash".to_string(), Box::new(BashTool::new(cmd.clone())));
    tools.insert("Read".to_string(), Box::new(ReadTool::new(fs.clone())));
    tools.insert("Write".to_string(), Box::new(WriteTool::new(fs.clone())));
    tools.insert("Talk".to_string(), Box::new(TalkTool::new()));

    let exit_command = ExitCommand::default();
    let mut commands = HashMap::<String, Box<dyn Command>>::new();
    commands.insert(exit_command.name().into(), Box::new(exit_command));

    let api_key =
        std::env::var("CHAT_GPT_API_KEY").expect("CHAT_GPT_API_KEY to be set");
    let agent = ChatGPT::new(String::from(api_key));

    let processor = Processor::init(Box::new(agent), tools, commands);
    GUI::run(Arc::new(Mutex::new(processor)))
        .expect("Failed to start gui application");

    // let (tx, mut rx) = mpsc::channel::<String>(1);
    // let input_thread = tokio::spawn(async move {
    // loop {
    // let mut input = String::new();
    // println!("Prompt: ");
    // std::io::stdin()
    // .read_line(&mut input)
    // .expect("Should have a input");
    //
    // match tx.send(input).await {
    // Ok(_) => {}
    // Err(error) => {
    // error!("Failed to send input to main thread: {error}");
    // break;
    // }
    // }
    // }
    // });
    //
    // let mut processor = Processor::init(Box::new(agent), tools, commands);
    // while let Some(input) = rx.recv().await.map(|i| i.trim().to_string())
    // && !input.is_empty()
    // {
    // if let Some(output) = processor.handle(input).await {
    // for entry in output {
    // info!("Response: {}", entry);
    // }
    // }
    // }
    //
    // input_thread.abort()
}
