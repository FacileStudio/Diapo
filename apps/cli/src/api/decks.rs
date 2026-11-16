//! Decks: list, create, read, and write the source against a revision.

use serde::{Deserialize, Serialize};
use serde_json::json;

use super::{ApiError, Client, Payload, text};
use crate::url::encode;

/// A deck as the list and the create call answer it.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct DeckSummary {
    /// The deck id.
    #[serde(deserialize_with = "text")]
    pub id: String,
    /// The deck title.
    #[serde(default)]
    pub title: String,
    /// The current revision, when the server lists it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rev: Option<i64>,
    /// When the deck last changed, when the server lists it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<String>,
}

/// A deck with its source.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct Deck {
    /// The deck id.
    #[serde(deserialize_with = "text")]
    pub id: String,
    /// The deck title.
    #[serde(default)]
    pub title: String,
    /// The whole source text.
    pub source: String,
    /// The revision this source is.
    pub rev: i64,
}

#[derive(Deserialize)]
struct Written {
    rev: i64,
}

impl Client {
    /// `GET /api/decks`.
    pub fn decks(&self) -> Result<Vec<DeckSummary>, ApiError> {
        self.list("/decks")
    }

    /// `POST /api/decks`: a new personal deck holding `source`, at revision 1.
    pub fn create_deck(&self, title: &str, source: &str) -> Result<DeckSummary, ApiError> {
        self.json("POST", "/decks", Payload::Json(&json!({ "title": title, "source": source })))
    }

    /// `GET /api/decks/{id}`.
    pub fn deck(&self, id: &str) -> Result<Deck, ApiError> {
        self.json("GET", &format!("/decks/{}", encode(id)), Payload::Empty)
    }

    /// `PUT /api/decks/{id}/source`: writes `source` over revision `rev` and answers the new
    /// revision. A stale `rev` is refused with `409`, a source that does not parse with `422`.
    pub fn put_source(&self, id: &str, source: &str, rev: i64) -> Result<i64, ApiError> {
        let body = json!({ "source": source, "rev": rev });
        let written: Written = self.json("PUT", &format!("/decks/{}/source", encode(id)), Payload::Json(&body))?;
        Ok(written.rev)
    }
}
