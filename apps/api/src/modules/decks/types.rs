use serde::{Deserialize, Deserializer, Serialize};
use time::OffsetDateTime;
use uuid::Uuid;

/// The columns every deck answer reads, as a literal for `concat!`.
macro_rules! columns {
    () => {
        "id, space_id, title, source, rev, created_at, updated_at, trashed_at"
    };
}
pub(crate) use columns;

/// Where a write came from, stored with its revision.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Origin {
    /// The web editor, in the person's browser session.
    Web,
    /// An agent, through `/mcp`.
    Mcp,
    /// The `diapo` CLI or any other bearer token.
    Cli,
}

impl Origin {
    /// The name stored in `deck_revisions.origin`.
    pub fn name(self) -> &'static str {
        match self {
            Origin::Web => "web",
            Origin::Mcp => "mcp",
            Origin::Cli => "cli",
        }
    }
}

/// Who writes, and through what.
#[derive(Debug, Clone, Copy)]
pub struct Author {
    /// The porte user id.
    pub user_id: i64,
    /// The door the write came through.
    pub origin: Origin,
}

impl Author {
    /// The author behind a REST request: `web` for a browser session, `cli` for any token.
    pub fn of(identity: &porte::Identity) -> Self {
        let origin = if identity.browser_session { Origin::Web } else { Origin::Cli };
        Author { user_id: identity.user_id, origin }
    }

    /// The author behind an MCP call.
    pub fn agent(user_id: i64) -> Self {
        Author { user_id, origin: Origin::Mcp }
    }
}

/// One `decks` row.
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct DeckRow {
    pub id: Uuid,
    pub space_id: Option<String>,
    pub title: String,
    pub source: String,
    pub rev: i64,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
    pub trashed_at: Option<OffsetDateTime>,
}

/// A deck as the API answers it.
#[derive(Debug, Serialize)]
pub struct Deck {
    pub id: Uuid,
    pub space_id: Option<String>,
    pub title: String,
    pub source: String,
    pub rev: i64,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
    #[serde(with = "time::serde::rfc3339")]
    pub updated_at: OffsetDateTime,
    #[serde(with = "time::serde::rfc3339::option")]
    pub trashed_at: Option<OffsetDateTime>,
    pub access: &'static str,
}

/// One deck of a list: no source.
#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct DeckEntry {
    pub id: Uuid,
    pub space_id: Option<String>,
    pub title: String,
    pub rev: i64,
    #[serde(with = "time::serde::rfc3339")]
    pub updated_at: OffsetDateTime,
    #[serde(with = "time::serde::rfc3339::option", skip_serializing_if = "Option::is_none")]
    pub trashed_at: Option<OffsetDateTime>,
    /// The caller's role in the deck's space; none for a personal deck.
    #[serde(skip)]
    pub role: Option<String>,
    #[sqlx(skip)]
    pub access: &'static str,
}

/// `POST /decks`.
#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CreateRequest {
    #[serde(default)]
    pub title: String,
    pub space_id: Option<String>,
    pub source: Option<String>,
}

/// `PUT /decks/{id}/source`: the whole source and the `rev` it was read at.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SourceRequest {
    pub source: String,
    pub rev: i64,
}

fn present<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Option<Option<String>>, D::Error> {
    Option::<String>::deserialize(deserializer).map(Some)
}

/// `PATCH /decks/{id}`. `space_id: null` moves the deck to the caller's personal decks; a missing
/// `space_id` leaves it where it is.
#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PatchRequest {
    pub title: Option<String>,
    #[serde(default, deserialize_with = "present")]
    pub space_id: Option<Option<String>>,
    pub trashed: Option<bool>,
}

/// `?space_id=&trashed=` on the list.
#[derive(Debug, Default, Deserialize)]
pub struct ListQuery {
    pub space_id: Option<String>,
    #[serde(default)]
    pub trashed: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_patch_tells_a_null_space_from_a_missing_one() {
        let missing: PatchRequest = serde_json::from_str(r#"{"title": "x"}"#).unwrap();
        assert_eq!(missing.space_id, None);
        let personal: PatchRequest = serde_json::from_str(r#"{"space_id": null}"#).unwrap();
        assert_eq!(personal.space_id, Some(None));
        let moved: PatchRequest = serde_json::from_str(r#"{"space_id": "s1"}"#).unwrap();
        assert_eq!(moved.space_id, Some(Some("s1".to_owned())));
        assert!(serde_json::from_str::<PatchRequest>(r#"{"icon": "x"}"#).is_err());
    }
}
