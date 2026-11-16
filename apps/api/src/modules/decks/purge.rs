use std::time::Duration;

use tokio_util::sync::CancellationToken;

use crate::state::AppState;

/// How often the trash is purged.
pub const EVERY: Duration = Duration::from_secs(60 * 60);

/// Deletes for good every deck trashed more than 30 days ago. The only path to permanent
/// deletion besides a manager's explicit delete.
pub async fn purge(state: &AppState) -> Result<u64, sqlx::Error> {
    let deleted =
        sqlx::query("DELETE FROM decks WHERE trashed_at < now() - interval '30 days'").execute(&state.pool).await?;
    Ok(deleted.rows_affected())
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
