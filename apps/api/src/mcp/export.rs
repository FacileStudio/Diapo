use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use tronc::mcp::{Answer, Call, ToolError};

use super::{Context, user};
use crate::extract::id;
use crate::modules::decks::error::DeckError;
use crate::modules::decks::service;

/// What a deck exports to.
#[derive(Debug, Clone, Copy, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum Format {
    /// One self-contained HTML file.
    Html,
    /// One page per slide.
    Pdf,
    /// An editable PowerPoint file.
    Pptx,
    /// A video, rendered by Clap.
    Mp4,
}

impl Format {
    fn name(self) -> &'static str {
        match self {
            Format::Html => "html",
            Format::Pdf => "pdf",
            Format::Pptx => "pptx",
            Format::Mp4 => "mp4",
        }
    }
}

/// `diapo_export`.
#[derive(Deserialize, JsonSchema)]
pub struct ExportIn {
    /// The deck id.
    pub deck_id: String,
    /// The file format: html, pdf, pptx or mp4.
    pub format: Format,
}

/// What `diapo_export` answers once exports exist.
#[derive(Serialize, JsonSchema)]
pub struct ExportOut {
    /// The export job id.
    pub id: String,
    /// `queued`, `running`, `done` or `failed`.
    pub status: String,
    /// Where the file is downloaded once the job is done.
    pub download_url: Option<String>,
}

/// `diapo_export`: checks the caller reads the deck, then says exports are not available yet.
pub async fn export(ctx: Context, call: Call, input: ExportIn) -> Result<Answer<ExportOut>, ToolError> {
    let deck_id = id(&input.deck_id).map_err(DeckError::Api)?;
    service::read(&ctx.state, user(&call)?, deck_id).await.map_err(DeckError::Api)?;
    Err(ToolError(format!(
        "exporting to {} is not available yet: nothing was exported. Tell the user that exports arrive in a \
         later release; do not retry.",
        input.format.name()
    )))
}
