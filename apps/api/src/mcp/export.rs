use std::time::Duration;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use tronc::mcp::{Answer, Call, ToolError};

use super::{Context, user};
use crate::extract::id;
use crate::modules::decks::error::DeckError;
use crate::modules::decks::service;
use crate::modules::exports::service::Job;
use crate::modules::exports::{Format, service as exports};

/// How long the tool waits for a job before it answers it as pending.
pub const WAIT: Duration = Duration::from_secs(30);

/// `diapo_export`.
#[derive(Deserialize, JsonSchema)]
pub struct ExportIn {
    /// The deck id.
    pub deck_id: String,
    /// The file format: html, pdf, pptx or mp4.
    pub format: Format,
    /// PDF only: a page per step instead of a page per slide.
    pub steps: Option<bool>,
    /// The `id` an earlier call answered as pending: reads that job instead of starting another.
    pub job_id: Option<String>,
}

/// What `diapo_export` answers.
#[derive(Serialize, JsonSchema)]
pub struct ExportOut {
    /// The export job id.
    pub id: String,
    /// `queued`, `running` or `done`.
    pub status: String,
    /// Where the file is downloaded: a link that works for one hour without signing in. Give it
    /// to the user. Absent until the job is done.
    pub download_url: Option<String>,
    /// The name of the file.
    pub file_name: String,
    /// What to do next, and what the file lacks.
    pub message: String,
}

fn pending(job: &Job) -> String {
    let progress = if job.total > 0 { format!(" ({} of {})", job.done, job.total) } else { String::new() };
    format!(
        "The export is still {}{progress}. Call diapo_export again with the same deck_id and format and job_id \
         \"{}\" to read it; do not start another one.",
        job.status, job.id
    )
}

fn answer(ctx: &Context, job: Job) -> Result<Answer<ExportOut>, ToolError> {
    if job.status == "failed" {
        return Err(ToolError(format!("the export failed: {}. Nothing was exported.", job.error)));
    }
    let done = job.status == "done";
    let message = match (done, job.notice.is_empty()) {
        (false, _) => pending(&job),
        (true, true) => "The file is ready: give download_url to the user.".to_owned(),
        (true, false) => format!("The file is ready: give download_url to the user. {}", job.notice),
    };
    let download_url = done.then(|| format!("{}{}", ctx.origin, exports::signed(&ctx.state, job.id)));
    Ok(ExportOut { id: job.id.to_string(), status: job.status, download_url, file_name: job.file_name, message }.into())
}

/// `diapo_export`: builds a file from the deck as it is now, waits up to [`WAIT`] for it, and
/// answers a link to it or the pending job to read again.
pub async fn export(ctx: Context, call: Call, input: ExportIn) -> Result<Answer<ExportOut>, ToolError> {
    let (user, deck_id) = (user(&call)?, id(&input.deck_id).map_err(DeckError::Api)?);
    service::read(&ctx.state, user, deck_id).await.map_err(DeckError::Api)?;
    let started = match input.job_id.as_deref() {
        Some(job) => exports::read(&ctx.state, user, id(job).map_err(DeckError::Api)?).await,
        None => exports::create(&ctx.state, user, deck_id, input.format, input.steps.unwrap_or(false)).await,
    };
    let job =
        started.map_err(|error| tronc::mcp::failure(error, "Call diapo_export without job_id to start again."))?;
    let job = exports::settled(&ctx.state, job.id, WAIT).await.map_err(DeckError::Api)?;
    answer(&ctx, job)
}
