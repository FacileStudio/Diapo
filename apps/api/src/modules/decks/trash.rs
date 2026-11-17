use tronc::{Error, Result};
use uuid::Uuid;

use super::access::{self, Access};
use super::service::{answer, fetch};
use super::types::{Deck, PatchRequest};
use crate::extract::db;
use crate::modules::exports::leftovers;
use crate::state::AppState;

/// Moves the deck to the trash, where it stays 30 days. A deck already there is left as it is.
pub async fn trash(state: &AppState, user_id: i64, id: Uuid) -> Result<Deck> {
    super::patch::patch(state, user_id, id, PatchRequest { trashed: Some(true), ..PatchRequest::default() }).await
}

/// `DELETE /decks/{id}`: deletes a trashed deck for good, with its revisions, assets, exports
/// and thumbnails.
pub async fn permanent(state: &AppState, user_id: i64, id: Uuid) -> Result<()> {
    access::check(state, user_id, id, Access::Manager).await?;
    let mut tx = state.pool.begin().await.map_err(db)?;
    let exports = leftovers::exports_of(&mut tx, &[id], &[]).await.map_err(db)?;
    let deleted = sqlx::query("DELETE FROM decks WHERE id = $1 AND trashed_at IS NOT NULL")
        .bind(id)
        .execute(&mut *tx)
        .await
        .map_err(db)?;
    if deleted.rows_affected() == 0 {
        return Err(Error::failed("only a deck in the trash can be deleted"));
    }
    tx.commit().await.map_err(db)?;
    state.hub.delete(&[id]).await;
    leftovers::forget(&state.data_dir, &[id], &exports).await;
    Ok(())
}

/// The deck, for a caller who may manage it: what a confirmation checks before it trashes.
pub async fn managed(state: &AppState, user_id: i64, id: Uuid) -> Result<Deck> {
    let have = access::check(state, user_id, id, Access::Manager).await?;
    Ok(answer(fetch(state, id).await?, have))
}
