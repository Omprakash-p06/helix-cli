use rustyline::validate::{ValidationContext, ValidationResult, Validator};
use rustyline::highlight::Highlighter;
use rustyline::hint::Hinter;
use rustyline::completion::Completer;
use rustyline::Helper;
use rustyline::Editor;
use rustyline::error::ReadlineError;

/// Multi-line input helper.
/// Rules:
///   - If the current buffer ends with a blank line (double-Enter), submit.
///   - Otherwise, treat Enter as "insert newline" (continue editing).
pub struct MultiLineHelper;

impl Completer for MultiLineHelper {
    type Candidate = String;
}

impl Hinter for MultiLineHelper {
    type Hint = String;
}

impl Highlighter for MultiLineHelper {}

impl Helper for MultiLineHelper {}

impl Validator for MultiLineHelper {
    fn validate(&self, _ctx: &mut ValidationContext) -> rustyline::Result<ValidationResult> {
        // Standard submission on Enter.
        // Users pasting multi-line text are protected by bracketed_paste(true).
        Ok(ValidationResult::Valid(None))
    }
}

pub fn create_editor() -> Result<Editor<MultiLineHelper, rustyline::history::FileHistory>, ReadlineError> {
    let config = rustyline::Config::builder()
        .auto_add_history(true)
        .bracketed_paste(true)
        .build();

    let helper = MultiLineHelper;
    let mut rl = Editor::with_config(config)?;
    rl.set_helper(Some(helper));

    // Load history from user home directory
    let history_path = history_file_path();
    let _ = rl.load_history(&history_path);

    Ok(rl)
}

pub fn save_history(rl: &mut Editor<MultiLineHelper, rustyline::history::FileHistory>) {
    let path = history_file_path();
    let _ = rl.save_history(&path);
}

fn history_file_path() -> std::path::PathBuf {
    // Cross-platform: HOME on Linux/Mac, USERPROFILE on Windows
    if let Some(home) = std::env::var_os("HOME")
        .or_else(|| std::env::var_os("USERPROFILE"))
    {
        std::path::PathBuf::from(home).join(".helix_history")
    } else {
        std::path::PathBuf::from(".helix_history")
    }
}
