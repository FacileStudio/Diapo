use tronc::{Error, Result};
use uuid::Uuid;

use super::access::{self, Access};
use super::error::DeckError;
use super::service::fetch;
use super::types::{Author, Deck, SourceRequest};
use super::write;
use crate::state::AppState;

/// One exact replacement in a deck's source, at the `rev` it was read at.
#[derive(Debug)]
pub struct Replacement {
    /// The text to find, exactly once.
    pub old: String,
    /// What takes its place.
    pub new: String,
    /// The revision the source was read at.
    pub rev: i64,
}

/// `source` with its single `old` replaced by `new`. Refuses an empty `old`, one that is not
/// there, and one that is there more than once: the edit must name one place.
pub fn replace_once(source: &str, old: &str, new: &str) -> Result<String> {
    if old.is_empty() {
        return Err(Error::invalid("old_string is empty: give the exact text to replace"));
    }
    match source.matches(old).count() {
        0 => Err(Error::invalid("old_string is not in the source: copy it exactly, whitespace included")),
        1 => Ok(source.replacen(old, new, 1)),
        count => Err(Error::invalid(format!(
            "old_string is in the source {count} times: add surrounding text until it names one place"
        ))),
    }
}

/// Replaces one exact string in the deck's source and saves the result as [`write::save`] does.
pub async fn edit(
    state: &AppState,
    author: &Author,
    id: Uuid,
    change: Replacement,
) -> std::result::Result<Deck, DeckError> {
    access::check(state, author.user_id, id, Access::Editor).await?;
    let row = fetch(state, id).await?;
    if row.rev != change.rev {
        return Err(DeckError::Stale { rev: row.rev, sent: change.rev });
    }
    let source = replace_once(&row.source, &change.old, &change.new)?;
    write::save(state, author, id, SourceRequest { source, rev: change.rev }).await
}

#[cfg(test)]
mod tests {
    use super::replace_once;

    #[test]
    fn a_replacement_names_exactly_one_place() {
        assert_eq!(replace_once("# A\n\n# B\n", "# B", "# C").unwrap(), "# A\n\n# C\n");
        assert!(replace_once("# A\n", "# Z", "x").unwrap_err().message.contains("not in the source"));
        assert!(replace_once("- a\n- a\n", "- a", "x").unwrap_err().message.contains("2 times"));
        assert!(replace_once("# A\n", "", "x").unwrap_err().message.contains("empty"));
    }
}
