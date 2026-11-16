use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use tronc::mcp::{Answer, Call, ToolError};

use super::{Context, user};
use crate::extract::id;
use crate::modules::decks::error::DeckError;
use crate::modules::decks::service;
use crate::modules::exports::{Format, service as exports};

/// `diapo_export`.
#[derive(Deserialize, JsonSchema)]
pub struct ExportIn {
    /// The deck id.
    pub deck_id: String,
    /// The file format: html, pdf, pptx or mp4.
    pub format: Format,
}

/// What `diapo_export` answers.
#[derive(Serialize, JsonSchema)]
pub struct ExportOut {
    /// The export job id.
    pub id: String,
    /// `queued`, `running`, `done` or `failed`.
    pub status: String,
    /// Where the file is downloaded: a link that works for one hour without signing in. Give it
    /// to the user. Absent when the job failed.
    pub download_url: Option<String>,
    /// The name of the file.
    pub file_name: String,
}

/// `diapo_export`: builds an HTML or a PPTX file from the deck as it is now and answers a link
/// to it. PDF and MP4 answer that they are not available yet.
pub async fn export(ctx: Context, call: Call, input: ExportIn) -> Result<Answer<ExportOut>, ToolError> {
    let deck_id = id(&input.deck_id).map_err(DeckError::Api)?;
    service::read(&ctx.state, user(&call)?, deck_id).await.map_err(DeckError::Api)?;
    if !input.format.ready() {
        return Err(ToolError(format!(
            "exporting to {} is not available yet: nothing was exported. html and pptx work today; tell the \
             user, do not retry.",
            input.format.name()
        )));
    }
    let job = exports::create(&ctx.state, user(&call)?, deck_id, input.format).await.map_err(DeckError::Api)?;
    if job.status != "done" {
        return Err(ToolError(format!("the export failed: {}. Nothing was exported.", job.error)));
    }
    let download_url = Some(format!("{}{}", ctx.origin, exports::signed(&ctx.state, job.id)));
    Ok(ExportOut { id: job.id.to_string(), status: job.status, download_url, file_name: job.file_name }.into())
}
