//! Why a command stopped, and how that is shown.

use std::io;

use serde_json::json;

use crate::api::ApiError;
use crate::ui;

/// What a command answers: nothing, or the reason it stopped.
pub type Outcome<T = ()> = Result<T, Failure>;

/// The reason a command stopped. Every failure exits 1.
#[derive(Debug, Clone, PartialEq)]
pub struct Failure {
    /// A stable identifier scripts can match, such as `conflict`.
    pub code: String,
    /// What failed and what to do about it, lowercase, with no glyph.
    pub message: String,
}

impl Failure {
    /// A failure with its code and its message.
    pub fn new(code: &str, message: impl Into<String>) -> Failure {
        Failure { code: code.to_owned(), message: message.into() }
    }

    /// A failure the command already printed in full.
    pub fn quiet(code: &str) -> Failure {
        Failure::new(code, "")
    }

    /// Prints the failure: the error line on stderr, and with `--json` the suite error
    /// envelope on stdout. A quiet failure prints nothing.
    pub fn report(&self) {
        if self.message.is_empty() {
            return;
        }
        ui::error(&self.message);
        if ui::json() {
            println!("{}", json!({ "error": { "code": self.code, "message": self.message } }));
        }
    }
}

impl From<io::Error> for Failure {
    fn from(error: io::Error) -> Failure {
        Failure::new("io", error.to_string())
    }
}

impl From<ApiError> for Failure {
    fn from(error: ApiError) -> Failure {
        match error {
            ApiError::Unreachable(detail) => Failure::new(
                "unreachable",
                format!("cannot reach the server ({detail}) — check the address and your connection"),
            ),
            ApiError::Malformed(detail) => Failure::new(
                "malformed",
                format!("the server sent an answer diapo cannot read ({detail}) — check the server address"),
            ),
            ApiError::Invalid(errors) => {
                let lines: Vec<String> =
                    errors.iter().map(|error| format!("line {}: {}", error.line, error.message)).collect();
                Failure::new("invalid", format!("the server refused the source: {}", lines.join("; ")))
            }
            ApiError::Refused { status: 401, .. } => {
                Failure::new("unauthorized", "the server refused this login — run `diapo login`")
            }
            ApiError::Refused { status, code, message } => {
                Failure::new(&code, format!("the server answered {status}: {message}"))
            }
        }
    }
}
