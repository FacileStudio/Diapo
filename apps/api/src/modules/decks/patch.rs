use tronc::Result;
use uuid::Uuid;

use super::access::{self, Access};
use super::service::{answer, title};
use super::types::{Deck, DeckRow, PatchRequest, columns};
use crate::extract::db;
use crate::state::AppState;

async fn allowed(state: &AppState, user_id: i64, id: Uuid, request: &PatchRequest) -> Result<()> {
    let managing = request.space_id.is_some() || request.trashed.is_some();
    let need = if managing { Access::Manager } else { Access::Editor };
    access::check(state, user_id, id, need).await?;
    if let Some(space) = &request.space_id {
        let target = access::scope(state, user_id, space.as_deref()).await?;
        access::require(target, Access::Editor)?;
    }
    Ok(())
}

/// `PATCH /decks/{id}`: the title (an editor), the space and the trash (a manager). Moving needs
/// the right to edit in the target space; a deck moved to no space becomes the caller's own.
pub async fn patch(state: &AppState, user_id: i64, id: Uuid, request: PatchRequest) -> Result<Deck> {
    allowed(state, user_id, id, &request).await?;
    let title = request.title.as_deref().map(title).transpose()?;
    let row = sqlx::query_as::<_, DeckRow>(concat!(
        "UPDATE decks SET title = coalesce($2, title), \
         space_id = CASE WHEN $3 THEN $4 ELSE space_id END, owner_id = CASE WHEN $3 THEN $6 ELSE owner_id END, \
         trashed_at = CASE WHEN $5::boolean IS NULL THEN trashed_at WHEN $5 THEN coalesce(trashed_at, now()) END, \
         updated_at = now(), updated_by = $6 WHERE id = $1 RETURNING ",
        columns!()
    ))
    .bind(id)
    .bind(title)
    .bind(request.space_id.is_some())
    .bind(request.space_id.flatten())
    .bind(request.trashed)
    .bind(user_id)
    .fetch_one(&state.pool)
    .await
    .map_err(db)?;
    state.events.changed(row.id, row.rev);
    let have = access::access(state, user_id, id).await?;
    Ok(answer(row, have))
}
