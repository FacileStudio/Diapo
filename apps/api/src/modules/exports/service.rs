use std::time::{Duration, Instant};

use serde::Serialize;
use time::OffsetDateTime;
use tronc::{Error, Result};
use uuid::Uuid;

use super::{Format, LINK_TTL, SCOPE, file_name, work};
use crate::extract::db;
use crate::modules::decks::access::{self, Access};
use crate::modules::decks::service::fetch;
use crate::modules::frame;
use crate::state::AppState;

/// How long `POST /decks/{id}/exports` waits for a quick job before it answers it as pending.
pub const QUICK: Duration = Duration::from_secs(2);

macro_rules! columns {
    () => {
        "id, deck_id, rev, format, steps, status, done, total, error, notice, file_name, size, created_by, created_at"
    };
}

/// One export job, as the API answers it.
#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct Job {
    pub id: Uuid,
    pub deck_id: Uuid,
    /// The deck revision the file is built from.
    pub rev: i64,
    pub format: String,
    /// A PDF page per step instead of per scene.
    pub steps: bool,
    /// `queued`, `running`, `done` or `failed`.
    pub status: String,
    /// Pages or frames made so far, and how many the job makes; 0 while unknown.
    pub done: i32,
    pub total: i32,
    /// Why the job failed; empty otherwise.
    pub error: String,
    /// What the file lacks, such as pictures of animated slides; empty otherwise.
    pub notice: String,
    pub file_name: String,
    pub size: i64,
    #[serde(skip)]
    pub created_by: Option<i64>,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
    /// Where a signed-in caller downloads the file, once the job is done.
    #[sqlx(skip)]
    pub download_url: Option<String>,
}

impl Job {
    /// Whether the job is over, `done` or `failed`.
    pub fn over(&self) -> bool {
        matches!(self.status.as_str(), "done" | "failed")
    }
}

fn located(job: Job) -> Job {
    let download_url = (job.status == "done").then(|| format!("/api/exports/{}/file", job.id));
    Job { download_url, ..job }
}

/// `POST /decks/{id}/exports`: queues the export of the deck as it is now and starts it. A job
/// that ends within [`QUICK`] is answered finished; a slower one is answered as it is, to be
/// read again with `GET /exports/{id}`. A format the server cannot produce right now is
/// `503 unavailable` and leaves no job. One person has one export under way at a time.
pub async fn create(state: &AppState, user_id: i64, deck: Uuid, format: Format, steps: bool) -> Result<Job> {
    access::check(state, user_id, deck, Access::Viewer).await?;
    super::formats::available(state, format).await?;
    let row = fetch(state, deck).await?;
    let id: Option<Uuid> = sqlx::query_scalar(
        "INSERT INTO exports (deck_id, rev, format, steps, file_name, created_by) \
         VALUES ($1, $2, $3, $4, $5, $6) \
         ON CONFLICT (created_by) WHERE status IN ('queued', 'running') DO NOTHING RETURNING id",
    )
    .bind(deck)
    .bind(row.rev)
    .bind(format.name())
    .bind(steps && format == Format::Pdf)
    .bind(file_name(&row.title, format))
    .bind(user_id)
    .fetch_optional(&state.pool)
    .await
    .map_err(db)?;
    let Some(id) = id else { return live(state, user_id, deck, format).await };
    let worker = state.clone();
    tokio::spawn(async move { work::run(&worker, id, row, format).await });
    settled(state, id, QUICK).await
}

/// The export `user_id` already has under way. One person runs one export at a time: asking
/// again for the same deck and format answers that job, asking for another is `429`.
async fn live(state: &AppState, user_id: i64, deck: Uuid, format: Format) -> Result<Job> {
    let job = sqlx::query_as::<_, Job>(concat!(
        "SELECT ",
        columns!(),
        " FROM exports WHERE created_by = $1 AND status IN ('queued', 'running')"
    ))
    .bind(user_id)
    .fetch_optional(&state.pool)
    .await
    .map_err(db)?
    .map(located)
    .ok_or_else(|| Error::conflict("an export just finished; ask again"))?;
    match job.deck_id == deck && job.format == format.name() {
        true => Ok(job),
        false => Err(Error::rate_limited(format!(
            "you have a {} export under way; wait until it is done, then ask again",
            job.format
        ))),
    }
}

/// The job once it is over, or as it is after `limit`.
pub async fn settled(state: &AppState, id: Uuid, limit: Duration) -> Result<Job> {
    let deadline = Instant::now() + limit;
    loop {
        let job = find(state, id).await?;
        if job.over() || Instant::now() > deadline {
            return Ok(job);
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
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
