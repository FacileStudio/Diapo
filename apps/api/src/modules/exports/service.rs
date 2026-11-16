use serde::Serialize;
use time::OffsetDateTime;
use tronc::{Error, Result};
use uuid::Uuid;

use super::{Format, LINK_TTL, SCOPE, build, file_name, path};
use crate::extract::db;
use crate::modules::decks::access::{self, Access};
use crate::modules::decks::service::fetch;
use crate::modules::frame;
use crate::state::AppState;

macro_rules! columns {
    () => {
        "id, deck_id, rev, format, status, error, file_name, size, created_at"
    };
}

/// One export job, as the API answers it.
#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct Job {
    pub id: Uuid,
    pub deck_id: Uuid,
    /// The deck revision the file was built from.
    pub rev: i64,
    pub format: String,
    /// `queued`, `running`, `done` or `failed`.
    pub status: String,
    /// Why the job failed; empty otherwise.
    pub error: String,
    pub file_name: String,
    pub size: i64,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
    /// Where a signed-in caller downloads the file, once the job is done.
    #[sqlx(skip)]
    pub download_url: Option<String>,
}

fn located(job: Job) -> Job {
    let download_url = (job.status == "done").then(|| format!("/api/exports/{}/file", job.id));
    Job { download_url, ..job }
}

async fn finish(state: &AppState, id: Uuid, built: Result<Vec<u8>>) -> Result<Job> {
    let stored = match built {
        Ok(bytes) => store(state, id, &bytes).await.map(|()| bytes.len() as i64),
        Err(error) => Err(error),
    };
    let (status, size, error) = match stored {
        Ok(size) => ("done", size, String::new()),
        Err(error) if error.code == tronc::Code::Internal => {
            tracing::error!(%error, cause = ?error.cause, job = %id, "exports: a job failed");
            ("failed", 0, "the export failed".to_owned())
        }
        Err(error) => ("failed", 0, error.message),
    };
    sqlx::query_as::<_, Job>(concat!(
        "UPDATE exports SET status = $2, size = $3, error = $4, finished_at = now() WHERE id = $1 RETURNING ",
        columns!()
    ))
    .bind(id)
    .bind(status)
    .bind(size)
    .bind(error)
    .fetch_one(&state.pool)
    .await
    .map(located)
    .map_err(db)
}

async fn store(state: &AppState, id: Uuid, bytes: &[u8]) -> Result<()> {
    let target = path(&state.data_dir, id);
    let failed = |error: std::io::Error| Error::internal("failed to store an export", error);
    tokio::fs::create_dir_all(target.parent().unwrap_or(&state.data_dir)).await.map_err(failed)?;
    tokio::fs::write(&target, bytes).await.map_err(failed)
}

/// `POST /decks/{id}/exports`: builds the file of `format` from the deck as it is now and
/// answers the finished job, `done` or `failed`. One job runs at a time. A format this build
/// cannot produce is `503 unavailable` and leaves no job.
pub async fn create(state: &AppState, user_id: i64, deck: Uuid, format: Format) -> Result<Job> {
    access::check(state, user_id, deck, Access::Viewer).await?;
    if !format.ready() {
        return Err(Error::unavailable(format!("exporting to {} is not available yet", format.name())));
    }
    let row = fetch(state, deck).await?;
    let id: Uuid = sqlx::query_scalar(
        "INSERT INTO exports (deck_id, rev, format, status, file_name, created_by) \
         VALUES ($1, $2, $3, 'running', $4, $5) RETURNING id",
    )
    .bind(deck)
    .bind(row.rev)
    .bind(format.name())
    .bind(file_name(&row.title, format))
    .bind(user_id)
    .fetch_one(&state.pool)
    .await
    .map_err(db)?;
    let _turn = state.exports.acquire().await.map_err(|error| Error::internal("the export queue is closed", error))?;
    let built = match build::bundle(state, deck).await {
        Ok(bundle) => build::file(format, row.source, bundle).await,
        Err(error) => Err(error),
    };
    finish(state, id, built).await
}

/// The job, whoever asks: callers check access first.
pub async fn find(state: &AppState, id: Uuid) -> Result<Job> {
    sqlx::query_as::<_, Job>(concat!("SELECT ", columns!(), " FROM exports WHERE id = $1"))
        .bind(id)
        .fetch_optional(&state.pool)
        .await
        .map_err(db)?
        .map(located)
        .ok_or_else(|| Error::not_found("export not found"))
}

/// `GET /exports/{id}`: the job, for a caller who reads its deck. Anyone else gets `404`.
pub async fn read(state: &AppState, user_id: i64, id: Uuid) -> Result<Job> {
    let job = find(state, id).await?;
    match access::access(state, user_id, job.deck_id).await? {
        Access::None => Err(Error::not_found("export not found")),
        _ => Ok(job),
    }
}

/// The download route of `job` with a token that works without a session for [`LINK_TTL`]: what
/// an agent hands the person.
pub fn signed(state: &AppState, job: Uuid) -> String {
    let token = state.frame_key.grant(SCOPE, job, frame::now() + LINK_TTL.as_secs());
    format!("/api/exports/{job}/file?token={token}")
}
