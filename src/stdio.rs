use std::{collections::HashMap, io::Write};

use agente_application::core::processor::{Processor, TaskResponse};
use agente_domain::{error::Error, models::session::Session};
use agente_infrastructure::config::Config;

use crate::ansi::Ansi;

/// Starts the stdio interface
pub async fn start_stdio(
    name: String,
    session: &Session,
    processor: &mut Processor,
) -> () {
    let _name = name.clone();
    let listener = processor.listener();
    tokio::spawn(async move {
        let cloned_listener = listener.clone();
        let mut listener = cloned_listener.lock().await;
        while let Some(response) = listener.recv().await {
            match response {
                TaskResponse::Done => draw_input(),
                TaskResponse::Thinking => draw_thinking(),
                TaskResponse::MessageResponse(message) => {
                    draw_message(&_name, message);
                }
                TaskResponse::ToolSignature((command, arguments)) => {
                    draw_command_signature(command, arguments)
                }
                TaskResponse::ToolResponse((command, arguments, response)) => {
                    draw_command_response(command, arguments, response)
                }
                TaskResponse::Error(error) => draw_error(error),
            };
        }
    });

    draw_banner(session.id.to_string());
    draw_message(&name, "Hello! Send me a message!");
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

fn draw_error(error: Error) {
    eprintln!(
        "{}{}> System: {error:#?}{}",
        Ansi::BOLD,
        Ansi::FG_RED,
        Ansi::RESET
    )
}

const MAX_COMMAND_OUTPUT_SIZE: usize = 500;

fn draw_command_response(
    command: String,
    arguments: HashMap<String, String>,
    response: String,
) {
    let parsed_args = arguments
        .iter()
        .map(|(key, value)| format!("{key}: {value}"))
        .collect::<Vec<_>>()
        .join(", ");

    if response.is_empty() {
        println!(
            "{}{}< Resolved({}{command}({parsed_args}){}{}){}",
            Ansi::BOLD,
            Ansi::FG_BLUE,
            Ansi::RESET,
            Ansi::BOLD,
            Ansi::FG_BLUE,
            Ansi::RESET
        );
    } else {
        fn truncate_chars(s: &str, max_chars: usize) -> String {
            s.chars().take(max_chars).collect()
        }

        let cropped_response = truncate_chars(&response, MAX_COMMAND_OUTPUT_SIZE);
        println!(
            "{}{}< Resolved({}{command}({parsed_args}){}{}){}: {cropped_response}...",
            Ansi::BOLD,
            Ansi::FG_BLUE,
            Ansi::RESET,
            Ansi::BOLD,
            Ansi::FG_BLUE,
            Ansi::RESET
        );
    }
}

fn draw_command_signature(command: String, arguments: HashMap<String, String>) {
    let parsed_args = arguments
        .iter()
        .map(|(key, value)| format!("{key}: {value}"))
        .collect::<Vec<_>>()
        .join(", ");

    println!(
        "{}{}< Running({}{command}({parsed_args}){}{}){}",
        Ansi::BOLD,
        Ansi::FG_BLUE,
        Ansi::RESET,
        Ansi::BOLD,
        Ansi::FG_BLUE,
        Ansi::RESET
    );
}

fn draw_thinking() {
    println!(
        "{}{}Thinking...{}",
        Ansi::BOLD,
        Ansi::FG_YELLOW,
        Ansi::RESET
    );
}

fn draw_input() {
    print!("\r\x1b[K{}", "> Type something: ");
    std::io::stdout().flush().unwrap();
}

fn draw_message(name: &String, message: impl Into<String> + std::fmt::Display) {
    println!(
        "{}{}[◉‿◉] > {name}{}: {message}",
        Ansi::BOLD,
        Ansi::FG_GREEN,
        Ansi::RESET
    );
}

fn draw_banner(session_id: String) {
    let banner = format!(
        "
┌───────────────────────────── \x1b[32mAGENTE\x1b[0m ─────────────────────────────┐
│  \x1b[32m[◉‿◉]\x1b[0m   > I'ready!                                              \
         │
│ \x1b[32m/|   |\\\x1b[0m  Session: {}           │
│ \x1b[32m |   |\x1b[0m   Running \
         at: http://localhost:{:<27}│
│ \x1b[32m/ \\ / \\\x1b[0m  Working dir: {:<43}│
└──────────────────────────────────────────────────────────────────┘
* Resuming sessions can have a lot of context, use a MEMORY.md to not waste tokens!
",
        session_id,
        Config::port(),
        Config::pwd(),
    );

    print!("{banner}\n");
}
