use std::sync::Arc;

use agente_application::tools::read::ReadTool;
use agente_domain::core::tool::Tool;
use agente_infrastructure::file_system::FileSystem;

#[tokio::main]
async fn main() {
    let fs = Arc::new(FileSystem::default());
    let read_tool = ReadTool::new(fs);
    match read_tool.handle(vec![String::from("src/main.rs")]).await {
        Ok(result) => println!("{result:#?}"),
        Err(err) => eprintln!("{err}"),
    }
}
