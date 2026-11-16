//! What the server can export right now, so a client asks instead of starting a job to find out.

use std::time::Duration;

use serde::Serialize;

use super::Format;
use crate::state::AppState;

/// How long the check waits for Clap to answer.
pub const CLAP_WAIT: Duration = Duration::from_secs(2);

/// One format and whether an export of it would start now.
#[derive(Debug, Serialize)]
pub struct Capability {
    pub format: &'static str,
    pub available: bool,
    /// Why the format is not available.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<&'static str>,
    /// What the file holds that depends on the server's state.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<&'static str>,
}

/// What `GET /exports/formats` answers.
#[derive(Debug, Serialize)]
pub struct Formats {
    pub formats: Vec<Capability>,
}

fn capability(format: Format, reason: Option<&'static str>, notes: Option<&'static str>) -> Capability {
    Capability { format: format.name(), available: reason.is_none(), reason, notes }
}

async fn clap(state: &AppState) -> Option<&'static str> {
    let Some(clap) = state.clap.as_ref() else { return Some("video export is not set up on this server") };
    let health = clap.http.get(format!("{}/health", clap.url)).timeout(CLAP_WAIT).send().await;
    match health.is_ok_and(|answer| answer.status().is_success()) {
        true => None,
        false => Some("the video service is not answering"),
    }
}

/// `GET /exports/formats`: HTML and PPTX always; PDF while the capture worker's heartbeat is
/// fresh; MP4 while Clap is configured and answers. Starts no job.
pub async fn formats(state: &AppState) -> Formats {
    let capturing = state.captures.alive().await;
    let pictures = if capturing { "raw scenes as pictures" } else { "raw scenes as placeholders" };
    let pdf = (!capturing).then_some("the capture service is not running");
    Formats {
        formats: vec![
            capability(Format::Html, None, None),
            capability(Format::Pptx, None, Some(pictures)),
            capability(Format::Pdf, pdf, None),
            capability(Format::Mp4, clap(state).await, None),
        ],
    }
}
