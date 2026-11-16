use axum::response::{IntoResponse, Response};
use http::StatusCode;
use serde_json::{Value, json};
use tronc::mcp::ToolError;

/// Why a deck write was refused. Every door answers the same three cases: REST as a status,
/// MCP as a tool error.
#[derive(Debug)]
pub enum DeckError {
    /// Any failure the suite envelope already names.
    Api(tronc::Error),
    /// The write named `sent`, and the deck has moved on to `rev`: `409`.
    Stale {
        /// The revision the deck is at.
        rev: i64,
        /// The revision the write named.
        sent: i64,
    },
    /// The source does not parse: `422`, every problem with its line.
    Invalid(scene::Errors),
}

impl From<tronc::Error> for DeckError {
    fn from(error: tronc::Error) -> Self {
        DeckError::Api(error)
    }
}

fn stale(rev: i64, sent: i64) -> String {
    format!("the deck is at rev {rev}, not {sent}: read it again, then retry")
}

/// The `409` body: tronc's `already_exists` envelope, plus the `rev` the deck is at.
pub fn stale_body(rev: i64, sent: i64) -> Value {
    let mut body = tronc::errors::envelope(tronc::Code::AlreadyExists.as_str(), &stale(rev, sent), None);
    body["rev"] = json!(rev);
    body
}

/// The `422` body: tronc's `invalid_argument` envelope, plus `errors`, one `{line, message}` per
/// problem in line order.
pub fn invalid_body(errors: &scene::Errors) -> Value {
    let mut body = tronc::errors::envelope(tronc::Code::InvalidArgument.as_str(), &errors.to_string(), None);
    let problem = |error: &scene::Error| json!({"line": error.line, "message": error.kind.to_string()});
    body["errors"] = Value::Array(errors.0.iter().map(problem).collect());
    body
}

impl IntoResponse for DeckError {
    fn into_response(self) -> Response {
        match self {
            DeckError::Api(error) => error.into_response(),
            DeckError::Stale { rev, sent } => tronc::json::write(StatusCode::CONFLICT, &stale_body(rev, sent)),
            DeckError::Invalid(errors) => tronc::json::write(StatusCode::UNPROCESSABLE_ENTITY, &invalid_body(&errors)),
        }
    }
}

impl From<DeckError> for ToolError {
    fn from(error: DeckError) -> Self {
        match error {
            DeckError::Api(error) => tronc::mcp::failure(error, "diapo_list_decks lists deck ids."),
            DeckError::Stale { rev, sent } => {
                ToolError(format!("{}. diapo_get_deck answers the current source and rev.", stale(rev, sent)))
            }
            DeckError::Invalid(errors) => ToolError(format!(
                "the source is not a valid deck, nothing was saved:\n{errors}\ndiapo_format_guide explains the format."
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_stale_write_is_a_409_naming_the_current_rev() {
        let response = DeckError::Stale { rev: 5, sent: 3 }.into_response();
        assert_eq!(response.status(), StatusCode::CONFLICT);
        let body = stale_body(5, 3);
        assert_eq!((body["error"]["code"].clone(), body["rev"].clone()), (json!("already_exists"), json!(5)));
    }

    #[test]
    fn an_invalid_source_is_a_422_listing_each_line() {
        let errors = scene::parse("# A\n\n---\ncolour: red\n---\n\n# B\n").unwrap_err();
        let body = invalid_body(&errors);
        assert_eq!(body["errors"][0], json!({"line": 4, "message": "unknown key `colour`"}));
        assert_eq!(body["error"]["message"], json!("line 4: unknown key `colour`"));
        assert_eq!(DeckError::Invalid(errors).into_response().status(), StatusCode::UNPROCESSABLE_ENTITY);
    }
}
