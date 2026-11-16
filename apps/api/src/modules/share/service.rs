use serde::Serialize;
use time::OffsetDateTime;
use tronc::{Error, Result};
use uuid::Uuid;

use crate::extract::db;
use crate::modules::decks::access::{self, Access};
use crate::modules::decks::service::fetch;
use crate::state::AppState;

/// The share state of a deck. `url` is set only by the call that created the link: the token is
/// shown once and cannot be read back.
#[derive(Debug, Default, Serialize)]
pub struct Link {
    pub shared: bool,
    #[serde(with = "time::serde::rfc3339::option")]
    pub created_at: Option<OffsetDateTime>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
}

/// The public page of the link `token`: `<public url>/p/<token>`.
pub fn url(state: &AppState, token: &str) -> String {
    format!("{}/p/{token}", state.public_url)
}

async fn live(state: &AppState, deck: Uuid) -> Result<Link> {
    let created_at: Option<OffsetDateTime> =
        sqlx::query_scalar("SELECT created_at FROM share_links WHERE deck_id = $1 AND revoked_at IS NULL")
            .bind(deck)
            .fetch_optional(&state.pool)
            .await
            .map_err(db)?;
    Ok(Link { shared: created_at.is_some(), created_at, url: None })
}

async fn insert(state: &AppState, user_id: i64, deck: Uuid) -> Result<Option<Link>> {
    let token = porte::new_token().map_err(|error| Error::internal("no randomness for a share token", error))?;
    let created_at: Option<OffsetDateTime> = sqlx::query_scalar(
        "INSERT INTO share_links (deck_id, token_hash, created_by) VALUES ($1, $2, $3) \
         ON CONFLICT (deck_id) WHERE revoked_at IS NULL DO NOTHING RETURNING created_at",
    )
    .bind(deck)
    .bind(porte::hash_token(&token))
    .bind(user_id)
    .fetch_optional(&state.pool)
    .await
    .map_err(db)?;
    Ok(created_at.map(|at| Link { shared: true, created_at: Some(at), url: Some(url(state, &token)) }))
}

async fn retire(state: &AppState, deck: Uuid) -> Result<u64> {
    sqlx::query("UPDATE share_links SET revoked_at = now() WHERE deck_id = $1 AND revoked_at IS NULL")
        .bind(deck)
        .execute(&state.pool)
        .await
        .map(|done| done.rows_affected())
        .map_err(db)
}

/// `POST /decks/{id}/share`: shares the deck. A new link answers its `url` and `true`; a deck
/// already shared answers its state without the `url`, and `false`. `rotate` revokes the live
/// link first, so the call always answers a new `url`.
pub async fn create(state: &AppState, user_id: i64, deck: Uuid, rotate: bool) -> Result<(Link, bool)> {
    access::check(state, user_id, deck, Access::Editor).await?;
    if fetch(state, deck).await?.trashed_at.is_some() {
        return Err(Error::failed("the deck is in the trash"));
    }
    if rotate {
        retire(state, deck).await?;
    }
    state.thumbs.request(state, deck);
    match insert(state, user_id, deck).await? {
        Some(link) => Ok((link, true)),
        None => Ok((live(state, deck).await?, false)),
    }
}

/// `GET /decks/{id}/share`: whether the deck is shared, and since when.
pub async fn state(state: &AppState, user_id: i64, deck: Uuid) -> Result<Link> {
    access::check(state, user_id, deck, Access::Editor).await?;
    live(state, deck).await
}

/// `DELETE /decks/{id}/share`: revokes the live link. The public routes answer `404` at once.
pub async fn revoke(state: &AppState, user_id: i64, deck: Uuid) -> Result<()> {
    access::check(state, user_id, deck, Access::Editor).await?;
    match retire(state, deck).await? {
        0 => Err(Error::not_found("the deck is not shared")),
        _ => Ok(()),
    }
}

/// The deck the live link `token` names, unless it is in the trash. Anything else is `404`.
pub async fn resolve(state: &AppState, token: &str) -> Result<Uuid> {
    sqlx::query_scalar(
        "SELECT d.id FROM share_links s JOIN decks d ON d.id = s.deck_id \
         WHERE s.token_hash = $1 AND s.revoked_at IS NULL AND d.trashed_at IS NULL",
    )
    .bind(porte::hash_token(token))
    .fetch_optional(&state.pool)
    .await
    .map_err(db)?
    .ok_or_else(|| Error::not_found("not found"))
}
