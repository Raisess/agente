use agente_application::core::processor::{Processor, TaskResponse};
use agente_domain::models::session::Session;
use agente_infrastructure::config::Config;

/// Starts the stdio interface
pub async fn start_stdio(session: &Session, processor: &mut Processor) -> () {
    let listener = processor.listener();
    tokio::spawn(async move {
        let cloned_listener = listener.clone();
        let mut listener = cloned_listener.lock().await;
        while let Some(response) = listener.recv().await {
            match response {
                TaskResponse::Thinking => println!("Thinking..."),
                TaskResponse::MessageResponse(message) => {
                    println!("[◉‿◉] > Agente: {message}")
                }
                TaskResponse::CommandSignature(command) => {
                    println!("< Running({command})")
                }
                TaskResponse::CommandResponse((command, result)) => {
                    if result.is_empty() {
                        println!("> Resolved({command})")
                    } else {
                        println!("> Resolved({command}): {result}")
                    }
                }
                TaskResponse::Error(error) => eprintln!("> System: {error:#?}"),
            };
        }
    });

    use std::io::Write;
    fn draw_input() {
        print!("\r\x1b[K{}", "> Type something: ");
        std::io::stdout().flush().unwrap();
    }

    let banner = format!(
    "
┌───────────────────────────── \x1b[32mAGENTE\x1b[0m ─────────────────────────────┐
│  \x1b[32m[◉‿◉]\x1b[0m   > I'ready!                                              │
│ \x1b[32m/|   |\\\x1b[0m  Session: {}           │
│ \x1b[32m |   |\x1b[0m   Running at: http://localhost:{:<27}│
│ \x1b[32m/ \\ / \\\x1b[0m  Working dir: {:<43}│
└──────────────────────────────────────────────────────────────────┘
* Resuming sessions can have a lot of context, use a MEMORY.md to not waste tokens!
",
    session.id,
    Config::port(),
    Config::pwd(),
);

    print!("{banner}\n");
    draw_input();

    loop {
        let mut prompt = String::new();
        std::io::stdin()
            .read_line(&mut prompt)
            .expect("Failed to read from stdin");
        if prompt.is_empty() {
            continue;
        }

        let _ = processor.handle(prompt.trim().to_string()).await;
    }
}
