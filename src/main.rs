use agente_app::tools::read::ReadTool;
use agente_domain::tool::Tool;

#[tokio::main]
async fn main() {
    let mut read_tool = ReadTool::new();
    match read_tool.handle("src/main.rs").await {
        Ok(result) => println!("{result:#?}"),
        Err(err) => eprintln!("{err}"),
    }
}
