use rustyline::Editor;
use rustyline::error::ReadlineError;
use rustyline::history::DefaultHistory;

pub struct Readline {
    rl: Editor<CustomRustyLineHelper, DefaultHistory>,
}

impl Readline {
    pub fn new() -> Self {
        let mut rl: Editor<CustomRustyLineHelper, DefaultHistory> =
            Editor::new().expect("Failed to start rustyline");
        rl.set_helper(Some(CustomRustyLineHelper));

        Self { rl }
    }

    pub fn read(&mut self, label: &str) -> Result<String, ReadlineError> {
        let prompt = self.rl.readline(label)?;
        Ok(prompt)
    }
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
