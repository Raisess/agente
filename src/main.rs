use agente_application::tools::read::ReadTool;
use agente_domain::core::tool::Tool;

#[tokio::main]
async fn main() {
    let mut read_tool = ReadTool::new();
    match read_tool.handle("src/main.rs").await {
        Ok(result) => println!("{result:#?}"),
        Err(err) => eprintln!("{err}"),
    }
}
