use uuid::Uuid;

use super::persist::convert;
use super::space::Live;
use crate::state::AppState;

type LoadError = Box<dyn std::error::Error + Send + Sync>;
type Row = (Option<Vec<u8>>, String, i64, bool);

enum Stored {
    Missing,
    Ready(Live),
    Built(Option<String>, Live),
}

async fn read(state: &AppState, id: Uuid) -> Result<Stored, LoadError> {
    let row: Option<Row> = sqlx::query_as("SELECT state, source, rev, trashed_at IS NOT NULL FROM decks WHERE id = $1")
        .bind(id)
        .fetch_optional(&state.pool)
        .await?;
    let Some((stored, text, rev, trashed)) = row else { return Ok(Stored::Missing) };
    let max = state.limits.source;
    if let Some(bytes) = stored {
        let doc = planche::doc::from_state(&bytes, std::iter::empty())?;
        return Ok(Stored::Ready(Live::new(doc, rev, trashed, max)));
    }
    let document = scene::planche::convert(&text).map_err(|errors| errors.to_string())?;
    let projection = planche::projection::write(&document);
    let live = Live::new(planche::yjs::write(&document), rev, trashed, max);
    Ok(Stored::Built((projection != text).then_some(projection), live))
}

async fn seed(state: &AppState, id: Uuid, projection: Option<String>, live: &mut Live) -> Result<bool, LoadError> {
    let bytes = planche::doc::encode(&live.doc);
    let Some(source) = projection else {
        let done = sqlx::query("UPDATE decks SET state = $2 WHERE id = $1 AND state IS NULL AND rev = $3")
            .bind(id)
            .bind(&bytes)
            .bind(live.rev)
            .execute(&state.pool)
            .await?;
        return Ok(done.rows_affected() == 1);
    };
    let Some(rev) = convert(&state.pool, id, &bytes, &source, live.rev).await? else { return Ok(false) };
    live.rev = rev;
    state.events.changed(id, rev);
    Ok(true)
}

/// Reads `decks.state`, or builds it from `decks.source` when it is empty and stores it at once:
/// every later load must start from the Yjs items the clients already hold. A source that is not
/// already the document's projection (a legacy scene source) is replaced by it, as a new
/// revision. `None` when the deck is gone.
pub async fn load(state: &AppState, id: Uuid) -> Result<Option<Live>, LoadError> {
    for _ in 0..3 {
        match read(state, id).await? {
            Stored::Missing => return Ok(None),
            Stored::Ready(live) => return Ok(Some(live)),
            Stored::Built(projection, mut live) => {
                if seed(state, id, projection, &mut live).await? {
                    return Ok(Some(live));
                }
            }
        }
    }
    Err("the deck kept changing while it loaded".into())
}
