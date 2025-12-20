use std::collections::HashMap;
use std::sync::Arc;

use agente::prompt::prompt;
use agente_application::tools::{read::ReadTool, write::WriteTool};
use agente_domain::core::tool::Tool;
use agente_domain::ports::agent::{Agent, FeedResponse, MessageRequest};
use agente_infrastructure::adapters::agents::chat_gpt::ChatGPT;
use agente_infrastructure::adapters::file_system::FileSystem;

#[tokio::main]
async fn main() {
    let fs = Arc::new(FileSystem::default());

    let mut tools = HashMap::<&str, Box<dyn Tool>>::new();
    tools.insert("Read", Box::new(ReadTool::new(fs.clone())));
    tools.insert("Write", Box::new(WriteTool::new(fs.clone())));

    let api_key =
        std::env::var("CHAT_GPT_API_KEY").expect("CHAT_GPT_API_KEY to be set");
    let mut agent = ChatGPT::new(String::from(api_key));
    let initial_feed_response = agent
        .feed(MessageRequest {
            previous_message_id: None,
            prompt: prompt(&tools),
        })
        .await
        .expect("Failed to setup agent initial prompt.");

    let mut feed_result = FeedResponse::default();
    loop {
        let mut input = String::new();
        println!("Prompt: ");
        std::io::stdin()
            .read_line(&mut input)
            .expect("Should have a input");

        let execution_plan = agent
            .ask(MessageRequest {
                previous_message_id: initial_feed_response.message_id.clone(),
                prompt: input.trim().to_string(),
            })
            .await
            .expect("Failed to ask the agent for the execution plan.");
        println!("RESPONSE: {execution_plan:#?}");

        for task in execution_plan {
            println!("Summary: {}", task.summary());

            let key = task.tool();
            let tool = tools
                .get(&key.as_str())
                .expect(&format!("Tool not found: {key}"));

            // @FIXME: improve this hardcoded ugly thing here
            let mut args = task.arguments();
            if key == "Write"
                && args.get(1).is_some_and(|value| value == "<NONE>")
            {
                args[1] = feed_result.content.clone();
            }

            let result = tool.handle(args).await;
            println!("{key}: {result:#?}");
            match result {
                Ok(mut message) => {
                    if let Some(usage_instruction) = tool.usage_instruction() {
                        message = format!("{usage_instruction}: {message}");
                    }

                    feed_result = agent
                        .feed(MessageRequest {
                            previous_message_id: None,
                            prompt: message,
                        })
                        .await
                        .expect(
                            "Failed to feed agent with tool result \
                             information.",
                        );
                    println!("FEED RESULT: {:#?}", feed_result);
                }
                Err(error) => eprintln!("{}", error.message()),
            }
        }
    }
}
