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

/// Fails every job left `queued` or `running`: at boot nothing runs, so they are leftovers of a
/// restart. Answers how many.
pub async fn recover(state: &AppState) -> Result<u64, sqlx::Error> {
    let left = sqlx::query(
        "UPDATE exports SET status = 'failed', error = 'the server restarted during this export; start it again', \
         finished_at = now() WHERE status IN ('queued', 'running')",
    )
    .execute(&state.pool)
    .await?;
    Ok(left.rows_affected())
}

/// Removes the capture job directories older than an hour: a job is collected within minutes,
/// so an old one was left by a restart.
pub async fn sweep_captures(state: &AppState) {
    let Ok(mut entries) = tokio::fs::read_dir(state.captures.dir()).await else { return };
    while let Ok(Some(entry)) = entries.next_entry().await {
        let age = entry.metadata().await.ok().and_then(|meta| meta.modified().ok()).and_then(|at| at.elapsed().ok());
        let old = age.is_some_and(|age| age > Duration::from_secs(60 * 60));
        if old && entry.file_type().await.is_ok_and(|kind| kind.is_dir()) {
            tokio::fs::remove_dir_all(entry.path()).await.ok();
        }
    }
}

/// Purges every hour until `token` is cancelled. Run it in one replica.
pub async fn run(state: AppState, token: CancellationToken) {
    let mut tick = tokio::time::interval(EVERY);
    loop {
        tokio::select! {
            () = token.cancelled() => return,
            _ = tick.tick() => {
                sweep_captures(&state).await;
                match purge(&state).await {
                    Ok(0) => {}
                    Ok(count) => tracing::info!(count, "exports: removed old exports"),
                    Err(error) => tracing::error!(%error, "exports: purge failed"),
                }
            }
        }
    }
}
