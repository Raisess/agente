use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use rustyline::Editor;
use rustyline::error::ReadlineError;
use rustyline::history::DefaultHistory;

use agente_application::core::processor::{Processor, TaskResponse};
use agente_domain::{error::Error, models::session::Session};
use agente_infrastructure::config::Config;

use crate::ansi::Ansi;

/// Starts the stdio interface
pub async fn start_stdio(
    name: String,
    session: &Session,
    processor: &mut Arc<Mutex<Processor>>,
) -> () {
    let _name = name.clone();
    let listener = processor.lock().unwrap().listener();
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

    draw_banner(session.id.to_string(), session.summary_phrase.clone());
    draw_message(&name, "Hello! Send me a message!");
    draw_input();

    let mut rl: Editor<CustomRustyLineHelper, DefaultHistory> =
        Editor::new().expect("Failed to start rustyline");
    rl.set_helper(Some(CustomRustyLineHelper));

    loop {
        let readline = rl.readline("");
        match readline {
            Ok(prompt) => {
                let _ = processor.lock().unwrap().handle(prompt).await;
            }
            Err(ReadlineError::Interrupted) => {
                processor
                    .lock()
                    .unwrap()
                    .exit()
                    .await
                    .expect("Failed to exit program!");
                break;
            }
            Err(ReadlineError::Eof) => break,
            Err(err) => {
                println!("Readline Error: {:?}", err);
                break;
            }
        }
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
    let parsed_args = __parse_tool_arguments(arguments);

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
        let cropped_response = __truncate_chars(&response, MAX_COMMAND_OUTPUT_SIZE);
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
    let parsed_args = __parse_tool_arguments(arguments);

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

fn __parse_tool_arguments(arguments: HashMap<String, String>) -> String {
    arguments
        .iter()
        .map(|(key, value)| format!("{key}: {}", __truncate_chars(value, 200)))
        .collect::<Vec<_>>()
        .join(", ")
}

fn __truncate_chars(s: &str, max_chars: usize) -> String {
    s.chars().take(max_chars).collect()
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
    println!("{}> Type something:{}", Ansi::BOLD, Ansi::RESET);
}

fn draw_message(name: &String, message: impl Into<String> + std::fmt::Display) {
    print!(
        "{}{}[◉‿◉] > {name}{}: {}",
        Ansi::BOLD,
        Ansi::FG_GREEN,
        Ansi::RESET,
        termimad::text(&message.into()),
    );
}

fn draw_banner(session_id: String, session_summary: Option<String>) {
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
* {}
",
        session_id,
        Config::port(),
        Config::pwd(),
        session_summary.unwrap_or("Fresh session starting...".to_string()),
    );

    print!("{banner}\n");
}

use rustyline::{
    Helper,
    completion::{Completer, Pair},
    highlight::{CmdKind, Highlighter},
    hint::Hinter,
    validate::{ValidationContext, ValidationResult, Validator},
};

use std::borrow::Cow;

struct CustomRustyLineHelper;

impl Completer for CustomRustyLineHelper {
    type Candidate = Pair;

    fn complete(
        &self,
        _line: &str,
        _pos: usize,
        _ctx: &rustyline::Context<'_>,
    ) -> rustyline::Result<(usize, Vec<Pair>)> {
        Ok((0, Vec::new()))
    }
}

impl Hinter for CustomRustyLineHelper {
    type Hint = String;

    fn hint(
        &self,
        _line: &str,
        _pos: usize,
        _ctx: &rustyline::Context<'_>,
    ) -> Option<String> {
        None
    }
}

impl Validator for CustomRustyLineHelper {
    fn validate(
        &self,
        _ctx: &mut ValidationContext<'_>,
    ) -> rustyline::Result<ValidationResult> {
        Ok(ValidationResult::Valid(None))
    }
}

impl Highlighter for CustomRustyLineHelper {
    fn highlight<'l>(&self, line: &'l str, _pos: usize) -> Cow<'l, str> {
        Cow::Owned(format!("\x1b[48;5;236m{}\x1b[0m", line))
    }

    fn highlight_char(&self, _line: &str, _pos: usize, _kind: CmdKind) -> bool {
        true
    }
}

impl Helper for CustomRustyLineHelper {}
