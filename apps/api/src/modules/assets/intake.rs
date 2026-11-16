use tronc::{Error, Result};
use uuid::Uuid;

use super::{name, sniff};
use crate::extract::db;
use crate::state::AppState;

/// One file sent to `POST /decks/{id}/assets`.
pub struct Upload<'a> {
    /// The `name` part, when the upload names its asset.
    pub name: Option<&'a str>,
    /// The file name the browser sent.
    pub file_name: &'a str,
    /// The bytes.
    pub bytes: &'a [u8],
}

/// The name the upload is stored under: the one it asks for when it is valid, else one made
/// from its file name.
pub fn named(file: &Upload<'_>) -> Result<String> {
    let Some(asked) = file.name.map(str::trim).filter(|asked| !asked.is_empty()) else {
        return Ok(name::from_file(file.file_name));
    };
    match name::valid(asked) {
        true => Ok(asked.to_owned()),
        false => Err(Error::invalid(
            "an asset name is a relative path of letters, digits, `.`, `_` and `-`, such as assets/chart.png",
        )),
    }
}

/// The type of `bytes`, sniffed, within `max` bytes.
pub fn typed(bytes: &[u8], max: usize) -> Result<&'static str> {
    if bytes.len() > max {
        return Err(Error::too_large(format!("an asset is at most {} MiB", max >> 20)));
    }
    sniff::sniff(bytes)
        .ok_or_else(|| Error::invalid("unsupported file: an asset is an image, a video, a sound or a font"))
}

/// Refuses an upload that takes the assets of `deck` past the deck cap. Replacing `name` frees
/// what it held.
pub async fn room(state: &AppState, deck: Uuid, name: &str, size: usize) -> Result<()> {
    let held: i64 =
        sqlx::query_scalar("SELECT coalesce(sum(size), 0)::bigint FROM assets WHERE deck_id = $1 AND name <> $2")
            .bind(deck)
            .bind(name)
            .fetch_one(&state.pool)
            .await
            .map_err(db)?;
    match held as usize + size > state.limits.deck_assets {
        true => {
            Err(Error::too_large(format!("the assets of a deck hold at most {} MiB", state.limits.deck_assets >> 20)))
        }
        false => Ok(()),
    }
}
