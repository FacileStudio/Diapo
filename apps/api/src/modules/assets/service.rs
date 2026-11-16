use std::collections::HashSet;

use serde::Serialize;
use time::OffsetDateTime;
use tronc::{Error, Result};
use uuid::Uuid;

use super::files;
use super::intake::{Upload, named, room, typed};
use crate::extract::db;
use crate::modules::decks::access::{self, Access};
use crate::modules::decks::service::fetch;
use crate::state::AppState;

/// One stored asset, as the API answers it.
#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct Asset {
    /// The relative path the deck's source writes to load it.
    pub name: String,
    pub mime: String,
    pub size: i64,
    pub sha256: String,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
    /// Where a signed-in caller reads the bytes.
    #[sqlx(skip)]
    pub url: String,
}

/// What `GET /decks/{id}/assets` answers.
#[derive(Debug, Serialize)]
pub struct Assets {
    pub assets: Vec<Asset>,
}

/// The route that serves the asset `name` of `deck`.
pub fn url(deck: Uuid, name: &str) -> String {
    format!("/api/decks/{deck}/assets/{name}")
}

fn located(deck: Uuid, asset: Asset) -> Asset {
    Asset { url: url(deck, &asset.name), ..asset }
}

/// `POST /decks/{id}/assets`: stores the bytes once, typed by their content, under the name
/// asked for or one made from the file name. The same name again replaces the asset.
pub async fn upload(state: &AppState, user_id: i64, deck: Uuid, file: Upload<'_>) -> Result<Asset> {
    access::check(state, user_id, deck, Access::Editor).await?;
    let row = fetch(state, deck).await?;
    if row.trashed_at.is_some() {
        return Err(Error::failed("the deck is in the trash"));
    }
    let (name, mime) = (named(&file)?, typed(file.bytes, state.limits.asset)?);
    room(state, deck, &name, file.bytes.len()).await?;
    let sha256 =
        files::store(&state.data_dir, file.bytes).await.map_err(|e| Error::internal("failed to store a file", e))?;
    let asset = sqlx::query_as::<_, Asset>(
        "INSERT INTO assets (deck_id, name, mime, size, sha256, created_by) VALUES ($1, $2, $3, $4, $5, $6) \
         ON CONFLICT (deck_id, name) DO UPDATE SET mime = EXCLUDED.mime, size = EXCLUDED.size, \
         sha256 = EXCLUDED.sha256, created_by = EXCLUDED.created_by, created_at = now() \
         RETURNING name, mime, size, sha256, created_at",
    )
    .bind(deck)
    .bind(&name)
    .bind(mime)
    .bind(file.bytes.len() as i64)
    .bind(&sha256)
    .bind(user_id)
    .fetch_one(&state.pool)
    .await
    .map_err(db)?;
    state.events.changed(deck, row.rev);
    Ok(located(deck, asset))
}

/// `GET /decks/{id}/assets`: the assets of a deck the caller may read, by name.
pub async fn list(state: &AppState, user_id: i64, deck: Uuid) -> Result<Assets> {
    access::check(state, user_id, deck, Access::Viewer).await?;
    let rows = sqlx::query_as::<_, Asset>(
        "SELECT name, mime, size, sha256, created_at FROM assets WHERE deck_id = $1 ORDER BY name",
    )
    .bind(deck)
    .fetch_all(&state.pool)
    .await
    .map_err(db)?;
    Ok(Assets { assets: rows.into_iter().map(|asset| located(deck, asset)).collect() })
}

/// `DELETE /decks/{id}/assets/{name}`: forgets the asset; the sweep removes its bytes once no
/// deck names them.
pub async fn remove(state: &AppState, user_id: i64, deck: Uuid, name: &str) -> Result<()> {
    access::check(state, user_id, deck, Access::Editor).await?;
    let deleted = sqlx::query("DELETE FROM assets WHERE deck_id = $1 AND name = $2")
        .bind(deck)
        .bind(name)
        .execute(&state.pool)
        .await
        .map_err(db)?;
    if deleted.rows_affected() == 0 {
        return Err(Error::not_found("asset not found"));
    }
    state.events.changed(deck, fetch(state, deck).await?.rev);
    Ok(())
}

/// The asset `name` of `deck`, whoever asks: callers check access first.
pub async fn find(state: &AppState, deck: Uuid, name: &str) -> Result<Asset> {
    sqlx::query_as::<_, Asset>(
        "SELECT name, mime, size, sha256, created_at FROM assets WHERE deck_id = $1 AND name = $2",
    )
    .bind(deck)
    .bind(name)
    .fetch_optional(&state.pool)
    .await
    .map_err(db)?
    .ok_or_else(|| Error::not_found("asset not found"))
}

/// The names of every asset of `deck`, whoever asks.
pub async fn names(state: &AppState, deck: Uuid) -> Result<HashSet<String>> {
    let names: Vec<String> = sqlx::query_scalar("SELECT name FROM assets WHERE deck_id = $1")
        .bind(deck)
        .fetch_all(&state.pool)
        .await
        .map_err(db)?;
    Ok(names.into_iter().collect())
}
