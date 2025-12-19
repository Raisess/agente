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
    agent.feed(&base_prompt).await.expect("To not fail");

    let mut feed_result = String::new();
    loop {
        let mut input = String::new();
        println!("Prompt: ");
        std::io::stdin()
            .read_line(&mut input)
            .expect("Should have a input");

        let execution_plan =
            agent.ask(&input.trim()).await.expect("To not fail");
        println!("RESPONSE: {execution_plan:#?}");

        for task in execution_plan {
            println!("Summary: {}", task.summary());

            let key = task.tool();
            let tool = tools
                .get(&key.as_str())
                .expect(&format!("Tool not found: {key}"));

            let mut args = task.arguments();
            if key == "Write" {
                args[1] = feed_result.clone();
            }

            let result = tool.handle(args).await;
            println!("{key}: {result:#?}");
            match result {
                Ok(mut message) => {
                    if let Some(usage_instruction) = tool.usage_instruction() {
                        message = format!("{usage_instruction}: {message}");
                    }

                    feed_result =
                        agent.feed(&message).await.expect("To not fail");
                    println!("FEED RESULT: {}", feed_result);
                }
                Err(error) => eprintln!("{}", error.message()),
            }
        }
    }
}
