use sqlx::PgConnection;
use tronc::Error;
use uuid::Uuid;

use super::access::{self, Access};
use super::error::DeckError;
use super::service::{answer, fetch};
use super::source;
use super::types::{Author, Deck, DeckRow, SourceRequest, columns};
use crate::extract::db;
use crate::modules::rooms::Rewrite;
use crate::state::AppState;

/// Keeps the source `row` holds as the revision `row.rev`, with its author and origin.
pub async fn revision(conn: &mut PgConnection, row: &DeckRow, author: &Author) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO deck_revisions (deck_id, rev, source, created_by, origin) \
         VALUES ($1, $2, $3, (SELECT u.id FROM porte_users u WHERE u.id = $4), $5)",
    )
    .bind(row.id)
    .bind(row.rev)
    .bind(&row.source)
    .bind(author.user_id)
    .bind(author.origin.name())
    .execute(conn)
    .await
    .map(|_| ())
}

async fn store(state: &AppState, author: &Author, id: Uuid, rev: i64, source: &str) -> Stored {
    let mut tx = state.pool.begin().await.map_err(db)?;
    let row = sqlx::query_as::<_, DeckRow>(concat!(
        "UPDATE decks SET source = $2, rev = rev + 1, updated_at = now(), updated_by = $4 \
         WHERE id = $1 AND rev = $3 AND trashed_at IS NULL AND state IS NULL RETURNING ",
        columns!()
    ))
    .bind(id)
    .bind(source)
    .bind(rev)
    .bind(author.user_id)
    .fetch_optional(&mut *tx)
    .await
    .map_err(db)?;
    let Some(row) = row else { return refusal(state, id, rev).await };
    revision(&mut tx, &row, author).await.map_err(db)?;
    tx.commit().await.map_err(db)?;
    Ok(Some(row))
}

type Stored = Result<Option<DeckRow>, DeckError>;

async fn refusal(state: &AppState, id: Uuid, sent: i64) -> Stored {
    let row: Option<(i64, bool, bool)> =
        sqlx::query_as("SELECT rev, trashed_at IS NOT NULL, state IS NOT NULL FROM decks WHERE id = $1")
            .bind(id)
            .fetch_optional(&state.pool)
            .await
            .map_err(db)?;
    match row {
        None => Err(Error::not_found("deck not found").into()),
        Some((_, true, _)) => Err(Error::failed("the deck is in the trash").into()),
        Some((rev, _, _)) if rev != sent => Err(DeckError::Stale { rev, sent }),
        Some(_) => Ok(None),
    }
}

/// `PUT /decks/{id}/source`, and every other door that writes a source: checks the caller may
/// edit, that the source reads in either format and fits once projected, then stores its planche
/// projection only if the deck is still at `request.rev`. A deck that has a live state is written
/// inside its room, as one transaction open editors see; one that has none is written here. The
/// new source is kept as a revision and open editors are told. A stale `rev` changes nothing.
pub async fn save(state: &AppState, author: &Author, id: Uuid, request: SourceRequest) -> Result<Deck, DeckError> {
    let have = access::check(state, author.user_id, id, Access::Editor).await?;
    let prepared = source::checked(&request.source, state.limits.source)?;
    let row = match store(state, author, id, request.rev, &prepared.source).await? {
        Some(row) => {
            state.events.changed(row.id, row.rev);
            state.thumbs.request(state, row.id);
            row
        }
        None => {
            let rewrite = Rewrite { next: prepared.document, rev: request.rev, author: *author };
            state.hub.save(state, id, rewrite).await?;
            fetch(state, id).await?
        }
    };
    Ok(answer(row, have))
}
