use std::time::Duration;

use tokio_util::sync::CancellationToken;

use uuid::Uuid;

use crate::modules::exports::leftovers;
use crate::state::AppState;

/// How often the trash is purged.
pub const EVERY: Duration = Duration::from_secs(60 * 60);

/// Deletes for good every deck trashed more than 30 days ago, with its exports and thumbnails. The only path to permanent
/// deletion besides a manager's explicit delete.
pub async fn purge(state: &AppState) -> Result<u64, sqlx::Error> {
    let mut tx = state.pool.begin().await?;
    let decks: Vec<Uuid> =
        sqlx::query_scalar("SELECT id FROM decks WHERE trashed_at < now() - interval '30 days' FOR UPDATE")
            .fetch_all(&mut *tx)
            .await?;
    let exports = leftovers::exports_of(&mut tx, &decks, &[]).await?;
    sqlx::query("DELETE FROM decks WHERE id = ANY($1)").bind(&decks).execute(&mut *tx).await?;
    tx.commit().await?;
    state.hub.delete(&decks).await;
    leftovers::forget(&state.data_dir, &decks, &exports).await;
    Ok(decks.len() as u64)
}

/// Purges every hour until `token` is cancelled. Run it in one replica.
pub async fn run(state: AppState, token: CancellationToken) {
    let mut tick = tokio::time::interval(EVERY);
    loop {
        tokio::select! {
            () = token.cancelled() => return,
            _ = tick.tick() => match purge(&state).await {
                Ok(0) => {}
                Ok(count) => tracing::info!(count, "decks: purged the trash"),
                Err(error) => tracing::error!(%error, "decks: trash purge failed"),
            },
        }
    }
}
