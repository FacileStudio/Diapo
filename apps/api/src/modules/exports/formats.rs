//! What the server can export right now, so a client asks instead of starting a job to find out.

use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use serde::Serialize;

use tronc::{Error, Result};

use super::Format;
use crate::state::AppState;

/// How long the check waits for Clap to answer.
pub const CLAP_WAIT: Duration = Duration::from_secs(2);

/// One format and whether an export of it would start now.
#[derive(Debug, Clone, Serialize)]
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
#[derive(Debug, Clone, Serialize)]
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

/// How long one answer is kept, so dialogs opening everywhere do not each ask Clap.
pub const KEPT: Duration = Duration::from_secs(30);

/// The last answer and when it was made.
#[derive(Debug, Clone, Default)]
pub struct Cache(Arc<Mutex<Option<(Instant, Formats)>>>);

impl Cache {
    /// Drops the kept answer, so the next request asks again.
    pub fn forget(&self) {
        *self.0.lock().unwrap_or_else(|poisoned| poisoned.into_inner()) = None;
    }
}

/// `GET /exports/formats`: HTML and PPTX always; PDF while the capture worker's heartbeat is
/// fresh; MP4 while Clap is configured and answers. Starts no job. One answer serves for
/// [`KEPT`].
pub async fn formats(state: &AppState) -> Formats {
    let kept = state.formats.0.lock().unwrap_or_else(|poisoned| poisoned.into_inner()).clone();
    if let Some((_, formats)) = kept.filter(|(at, _)| at.elapsed() < KEPT) {
        return formats;
    }
    let fresh = probe(state).await;
    *state.formats.0.lock().unwrap_or_else(|poisoned| poisoned.into_inner()) = Some((Instant::now(), fresh.clone()));
    fresh
}

async fn probe(state: &AppState) -> Formats {
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

/// `503` for a format the server cannot produce right now, checked again when a job is asked.
pub(super) async fn available(state: &AppState, format: Format) -> Result<()> {
    match format {
        Format::Html | Format::Pptx => Ok(()),
        Format::Pdf => state.captures.require().await,
        Format::Mp4 if state.clap.is_some() => Ok(()),
        Format::Mp4 => Err(Error::unavailable("exporting to mp4 is not available on this server")),
    }
}
