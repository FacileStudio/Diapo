//! Storage: the Yjs state, the projection every reader uses, the next `rev` and a revision, in
//! one transaction.

use sqlx::PgPool;
use uuid::Uuid;

use super::space::Live;
use crate::modules::decks::types::{Author, DeckRow, Origin, columns};
use crate::modules::decks::write::revision;

type StoreError = Box<dyn std::error::Error + Send + Sync>;

/// Stores a deck's first state together with `source`, the projection of its legacy source, while
/// it is still at `rev` with no state. The source it held is kept as the revision `rev` when no
/// revision holds it yet. Answers the new rev, or `None` when the deck moved on.
pub async fn convert(pool: &PgPool, id: Uuid, state: &[u8], source: &str, rev: i64) -> Result<Option<i64>, StoreError> {
    let mut tx = pool.begin().await?;
    sqlx::query(
        "INSERT INTO deck_revisions (deck_id, rev, source, created_by, origin) \
         SELECT id, rev, source, updated_by, 'web' FROM decks WHERE id = $1 ON CONFLICT (deck_id, rev) DO NOTHING",
    )
    .bind(id)
    .execute(&mut *tx)
    .await?;
    let row = sqlx::query_as::<_, DeckRow>(concat!(
        "UPDATE decks SET state = $2, source = $3, rev = rev + 1, updated_at = now() \
         WHERE id = $1 AND state IS NULL AND rev = $4 RETURNING ",
        columns!()
    ))
    .bind(id)
    .bind(state)
    .bind(source)
    .bind(rev)
    .fetch_optional(&mut *tx)
    .await?;
    let Some(row) = row else { return Ok(None) };
    revision(&mut tx, &row, &Author { user_id: 0, origin: Origin::Web }).await?;
    tx.commit().await?;
    Ok(Some(row.rev))
}

/// Stores the room's deck: its state, its projection as `source`, `rev + 1` and a revision by
/// `author` through the door the last server-side change named. `None` when the deck is gone.
pub async fn persist(pool: &PgPool, id: Uuid, live: &Live, author: Option<i64>) -> Result<Option<DeckRow>, StoreError> {
    let source = planche::projection::write(&planche::yjs::read(&live.doc)?);
    let state = planche::doc::encode(&live.doc);
    let author = Author { user_id: author.unwrap_or_default(), origin: live.origin.unwrap_or(Origin::Web) };
    let mut tx = pool.begin().await?;
    let row = sqlx::query_as::<_, DeckRow>(concat!(
        "UPDATE decks SET state = $2, source = $3, rev = rev + 1, updated_at = now(), \
         updated_by = coalesce((SELECT u.id FROM porte_users u WHERE u.id = $4), updated_by) \
         WHERE id = $1 RETURNING ",
        columns!()
    ))
    .bind(id)
    .bind(&state)
    .bind(&source)
    .bind(author.user_id)
    .fetch_optional(&mut *tx)
    .await?;
    let Some(row) = row else { return Ok(None) };
    revision(&mut tx, &row, &author).await?;
    tx.commit().await?;
    Ok(Some(row))
}
