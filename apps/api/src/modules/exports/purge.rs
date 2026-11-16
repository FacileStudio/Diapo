use std::time::Duration;

use tokio_util::sync::CancellationToken;
use uuid::Uuid;

use super::path;
use crate::state::AppState;

/// How often old exports are removed.
pub const EVERY: Duration = Duration::from_secs(60 * 60);

/// Forgets every job created more than a day ago and removes its file. Answers how many went.
pub async fn purge(state: &AppState) -> Result<usize, sqlx::Error> {
    let gone: Vec<Uuid> =
        sqlx::query_scalar("DELETE FROM exports WHERE created_at < now() - interval '1 day' RETURNING id")
            .fetch_all(&state.pool)
            .await?;
    for id in &gone {
        tokio::fs::remove_file(path(&state.data_dir, *id)).await.ok();
    }
    Ok(gone.len())
}

/// Purges every hour until `token` is cancelled. Run it in one replica.
pub async fn run(state: AppState, token: CancellationToken) {
    let mut tick = tokio::time::interval(EVERY);
    loop {
        tokio::select! {
            () = token.cancelled() => return,
            _ = tick.tick() => match purge(&state).await {
                Ok(0) => {}
                Ok(count) => tracing::info!(count, "exports: removed old exports"),
                Err(error) => tracing::error!(%error, "exports: purge failed"),
            },
        }
    }
}
