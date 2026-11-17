//! One export job, from `queued` to its file or its reason.

use std::time::Duration;

use capture::job::{Kind, State};
use tronc::{Error, Result};
use uuid::Uuid;

use super::{Format, build, clap, path};
use crate::captures::{Ask, safe};
use crate::extract::db;
use crate::modules::decks::types::DeckRow;
use crate::state::AppState;

/// How long the capture of a PDF may take.
pub const PDF_LIMIT: Duration = Duration::from_secs(300);

/// How long a PPTX waits for the pictures of its animated slides before it does without.
pub const STILLS_LIMIT: Duration = Duration::from_secs(60);

/// What a PPTX says when it holds placeholders for its animated slides.
pub const NO_STILLS: &str =
    "The animated slides are placeholders: the capture service was not available. Export again later for pictures.";

/// A built export, and what it lacks.
pub struct Built {
    /// The bytes of the export, or `None` when the builder already wrote the file in place.
    pub bytes: Option<Vec<u8>>,
    /// The size of the file in bytes.
    pub size: i64,
    /// What the file lacks; empty when nothing.
    pub notice: String,
}

impl Built {
    /// A file held in memory.
    pub fn of(bytes: Vec<u8>, notice: String) -> Self {
        Built { size: bytes.len() as i64, bytes: Some(bytes), notice }
    }
}

/// Records how far the job is.
pub async fn progress(state: &AppState, id: Uuid, done: u32, total: u32) {
    let _ = sqlx::query("UPDATE exports SET done = $2, total = $3 WHERE id = $1")
        .bind(id)
        .bind(done as i32)
        .bind(total as i32)
        .execute(&state.pool)
        .await;
}

async fn pdf(state: &AppState, id: Uuid, deck: &DeckRow, steps: bool) -> Result<Built> {
    let captures = &state.captures;
    captures.require().await?;
    let job = captures.submit(state, deck, Kind::Pdf, Ask { steps, scenes: None }).await?;
    let (sender, mut heard) = tokio::sync::mpsc::unbounded_channel();
    let waited = captures.wait(&job, PDF_LIMIT, |status| _ = sender.send((status.done, status.total)));
    tokio::pin!(waited);
    let status = loop {
        tokio::select! {
            status = &mut waited => break status,
            Some((done, total)) = heard.recv() => progress(state, id, done, total).await,
        }
    };
    let read = match status {
        Ok(status) if status.state == State::Done => {
            captures.read(&captures.layout(&job).output(Kind::Pdf), safe::DOCUMENT).await
        }
        Ok(status) => Err(std::io::Error::other(status.error.unwrap_or_else(|| "the capture was cancelled".into()))),
        Err(error) => Err(std::io::Error::other(error.message)),
    };
    captures.remove(&job).await;
    let bytes = read.map_err(|error| Error::failed(format!("the PDF could not be made: {error}")))?;
    Ok(Built::of(bytes, String::new()))
}

/// The pictures of the `raw` scenes of `deck`, by scene index: empty when it has none, `None`
/// when the capture service could not make them.
async fn stills(state: &AppState, deck: &DeckRow) -> Option<build::Pictures> {
    let project = scene::planche::project(&deck.source).ok()?;
    let raw: Vec<usize> =
        (0..project.scenes().len()).filter(|index| project.scenes()[*index].layout == scene::Layout::Raw).collect();
    if raw.is_empty() {
        return Some(build::Pictures::new());
    }
    let captures = &state.captures;
    if !captures.alive().await {
        return None;
    }
    let job = captures.submit(state, deck, Kind::Stills, Ask { steps: false, scenes: Some(raw.clone()) }).await.ok()?;
    let done = captures.wait(&job, STILLS_LIMIT, |_| {}).await.is_ok_and(|status| status.state == State::Done);
    let mut pictures = build::Pictures::new();
    for scene in raw.iter().filter(|_| done) {
        if let Ok(bytes) = captures.read(&captures.layout(&job).still(*scene), safe::PICTURE).await {
            pictures.insert(*scene, bytes);
        }
    }
    captures.remove(&job).await;
    done.then_some(pictures)
}

async fn built(state: &AppState, id: Uuid, format: Format, deck: DeckRow, steps: bool) -> Result<Built> {
    match format {
        Format::Pdf => pdf(state, id, &deck, steps).await,
        Format::Mp4 => clap::render(state, id, &deck).await,
        Format::Html => {
            let bundle = build::bundle(state, deck.id).await?;
            let bytes = build::file(format, deck.source, bundle, build::Pictures::new()).await?;
            Ok(Built::of(bytes, String::new()))
        }
        Format::Pptx => {
            let pictures = stills(state, &deck).await;
            let notice = if pictures.is_none() { NO_STILLS.to_owned() } else { String::new() };
            let bundle = build::bundle(state, deck.id).await?;
            let bytes = build::file(format, deck.source, bundle, pictures.unwrap_or_default()).await?;
            Ok(Built::of(bytes, notice))
        }
    }
}

async fn store(state: &AppState, id: Uuid, bytes: &[u8]) -> Result<()> {
    let target = path(&state.data_dir, id);
    let failed = |error: std::io::Error| Error::internal("failed to store an export", error);
    tokio::fs::create_dir_all(target.parent().unwrap_or(&state.data_dir)).await.map_err(failed)?;
    tokio::fs::write(&target, bytes).await.map_err(failed)
}

async fn finish(state: &AppState, id: Uuid, built: Result<Built>) {
    let stored = match built {
        Ok(Built { bytes: Some(bytes), size, notice }) => store(state, id, &bytes).await.map(|()| (size, notice)),
        Ok(built) => Ok((built.size, built.notice)),
        Err(error) => Err(error),
    };
    let (status, size, error, notice) = match stored {
        Ok((size, notice)) => ("done", size, String::new(), notice),
        Err(error) if error.code == tronc::Code::Internal => {
            tracing::error!(%error, cause = ?error.cause, job = %id, "exports: a job failed");
            ("failed", 0, "the export failed".to_owned(), String::new())
        }
        Err(error) => ("failed", 0, error.message, String::new()),
    };
    let saved = sqlx::query(
        "UPDATE exports SET status = $2, size = $3, error = $4, notice = $5, finished_at = now() WHERE id = $1",
    )
    .bind(id)
    .bind(status)
    .bind(size)
    .bind(error)
    .bind(notice)
    .execute(&state.pool)
    .await;
    if let Err(error) = saved.map_err(db) {
        tracing::error!(%error, job = %id, "exports: a job could not be closed");
    }
}

/// Runs the job `id` on `deck` as it was when the job was queued. Two lanes, one job at a time
/// in each: MP4, which waits on Clap for as long as a render takes, and every other format.
pub async fn run(state: &AppState, id: Uuid, deck: DeckRow, format: Format) {
    let lane = if format == Format::Mp4 { &state.videos } else { &state.exports };
    let Ok(_turn) = lane.acquire().await else { return };
    let started: std::result::Result<Option<bool>, _> =
        sqlx::query_scalar("UPDATE exports SET status = 'running' WHERE id = $1 AND status = 'queued' RETURNING steps")
            .bind(id)
            .fetch_optional(&state.pool)
            .await;
    let Ok(Some(steps)) = started else { return };
    finish(state, id, built(state, id, format, deck, steps).await).await;
}
