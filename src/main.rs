use std::sync::Arc;

use agente_application::tools::{read::ReadTool, write::WriteTool};
use agente_domain::core::tool::Tool;
use agente_infrastructure::file_system::FileSystem;

#[tokio::main]
async fn main() {
    let fs = Arc::new(FileSystem::default());
    let read = ReadTool::new(fs.clone());
    match read.handle(vec![String::from("src/main.rs")]).await {
        Ok(result) => {
            println!("{result:#?}");
            let write = WriteTool::new(fs);
            write
                .handle(vec![String::from("copy.txt"), result])
                .await
                .expect("Failed to write text file");
        }
        Err(err) => eprintln!("{err}"),
    }
}
