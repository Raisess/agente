use std::collections::HashMap;
use std::sync::Arc;

use tokio::sync::mpsc;
use tracing_subscriber;
use tracing_subscriber::EnvFilter;

use agente::runtime::Runtime;
use agente_application::tools::bash::BashTool;
use agente_application::tools::read::ReadTool;
use agente_application::tools::talk::TalkTool;
use agente_application::tools::write::WriteTool;
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

    let api_key =
        std::env::var("CHAT_GPT_API_KEY").expect("CHAT_GPT_API_KEY to be set");
    let agent = ChatGPT::new(String::from(api_key));

    let (input_tx, mut input_rx) = mpsc::channel::<String>(1);
    tokio::spawn(async move {
        loop {
            let mut input = String::new();
            println!("Prompt: ");
            std::io::stdin()
                .read_line(&mut input)
                .expect("Should have a input");

            match input_tx.send(input).await {
                Ok(_) => {}
                Err(error) => {
                    eprintln!("Failed to send input to main thread: {error}")
                }
            }
        }
    });

    let (output_tx, mut output_rx) = mpsc::channel::<String>(1);
    tokio::spawn(async move {
        while let Some(output) = output_rx.recv().await {
            println!("Response: {}", output);
        }
    });

    let mut runtime = Runtime::init(Box::new(agent), tools);
    runtime.run(&mut input_rx, output_tx).await;
}
