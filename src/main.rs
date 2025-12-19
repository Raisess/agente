use std::collections::HashMap;
use std::sync::Arc;

use agente::prompt::prompt;
use agente_application::tools::{read::ReadTool, write::WriteTool};
use agente_domain::core::tool::Tool;
use agente_domain::ports::agent::Agent;
use agente_infrastructure::adapters::agents::chat_gpt::ChatGPT;
use agente_infrastructure::adapters::file_system::FileSystem;

#[tokio::main]
async fn main() {
    let fs = Arc::new(FileSystem::default());

    let mut tools = HashMap::<&str, Box<dyn Tool>>::new();
    tools.insert("Read", Box::new(ReadTool::new(fs.clone())));
    tools.insert("Write", Box::new(WriteTool::new(fs.clone())));

    let base_prompt = prompt(&tools);
    println!("{base_prompt}");

    let api_key =
        std::env::var("CHAT_GPT_API_KEY").expect("CHAT_GPT_API_KEY to be set");
    let mut agent = ChatGPT::new(String::from(api_key));
    agent.prepare(&base_prompt).await.expect("To not fail");

    let execution_plan = agent
        .ask("Read file ./src/main.rs")
        .await
        .expect("To not fail");
    println!("RESPONSE: {execution_plan:#?}");

    for task in execution_plan {
        println!("Summary: {}", task.summary());

        let key = task.tool();
        let tool = tools
            .get(&key.as_str())
            .expect(&format!("Tool not found: {key}"));
        let result = tool.handle(task.arguments()).await;
        println!("{key}: {result:#?}");
    }

    // let read = tools.get("Read").expect("Read tool to be ready");
    // match read.handle(vec![String::from("src/main.rs")]).await {
    // Ok(result) => {
    // println!("{result:#?}");
    // let write = tools.get("Write").expect("Write tool to be ready");
    // write
    // .handle(vec![String::from("copy.txt"), result])
    // .await
    // .expect("Failed to write text file");
    // }
    // Err(err) => eprintln!("{err:#?}"),
    // };
}
