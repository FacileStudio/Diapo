//! The files a capture job is made of. The API writes `bundle/` then `job.json`; the worker
//! answers in `status.json` and writes the outputs beside it.

use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use time::format_description::well_known::Rfc3339;

pub use crate::layout::Layout;

/// The heartbeat file of the captures directory. It starts with a dot, so it is never a job.
pub const HEARTBEAT: &str = ".worker";

/// The oldest heartbeat a live worker has, in seconds.
pub const FRESH_SECONDS: i64 = 30;

/// What a job produces.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Kind {
    /// `thumbnail.jpg`: scene 0 at its last step, `width` wide.
    Thumbnail,
    /// `out.pdf`: one page per scene, or per step with `steps`.
    Pdf,
    /// `stills/<n>.png`: each scene of `scenes` at its last step, at the deck size.
    Stills,
}

/// `job.json`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Spec {
    /// The job id: the name of its directory.
    pub id: String,
    /// What to produce.
    pub kind: Kind,
    /// The stage width in pixels.
    pub width: u32,
    /// The stage height in pixels.
    pub height: u32,
    /// A PDF page per step instead of per scene.
    #[serde(default)]
    pub steps: bool,
    /// The scenes to shoot, from 0, for `stills`; `null` is every scene.
    #[serde(default)]
    pub scenes: Option<Vec<usize>>,
    /// When the API queued the job, RFC 3339. The oldest job runs first.
    pub created_at: String,
}

/// Where a job is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum State {
    /// The worker is on it.
    Running,
    /// The outputs are whole.
    Done,
    /// It ended with `error`.
    Failed,
    /// A `cancel` file stopped it.
    Cancelled,
}

/// `status.json`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Status {
    /// Where the job is.
    pub state: State,
    /// Pictures or pages made so far.
    pub done: u32,
    /// Pictures or pages the job makes; 0 until the deck has loaded.
    pub total: u32,
    /// Why the job failed.
    pub error: Option<String>,
    /// When this file was written, RFC 3339.
    pub updated_at: String,
}

/// Now, in UTC, to the second, RFC 3339.
pub fn now() -> String {
    let now = OffsetDateTime::now_utc().replace_nanosecond(0).unwrap_or(OffsetDateTime::UNIX_EPOCH);
    now.format(&Rfc3339).unwrap_or_default()
}

/// The instant `text` names.
pub fn parse(text: &str) -> Option<OffsetDateTime> {
    OffsetDateTime::parse(text.trim(), &Rfc3339).ok()
}

/// Whether `name` can be a job id: 1 to 64 letters, digits, `-` and `_`.
pub fn named(name: &str) -> bool {
    (1..=64).contains(&name.len())
        && name.bytes().all(|byte| byte.is_ascii_alphanumeric() || byte == b'-' || byte == b'_')
}

/// Whether the heartbeat text `written` is less than [`FRESH_SECONDS`] old.
pub fn fresh(written: &str) -> bool {
    parse(written)
        .map(|then| (OffsetDateTime::now_utc() - then).whole_seconds())
        .is_some_and(|age| (-FRESH_SECONDS..=FRESH_SECONDS).contains(&age))
}

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};

    use super::*;

    #[test]
    fn a_job_file_reads_back_and_names_are_plain() {
        let text = r#"{"id":"j1","kind":"stills","width":1920,"height":1080,"scenes":[1,3],"created_at":"2026-11-16T10:00:00Z"}"#;
        let spec: Spec = serde_json::from_str(text).unwrap();
        assert_eq!((spec.kind, spec.steps, spec.scenes.clone()), (Kind::Stills, false, Some(vec![1, 3])));
        assert!(serde_json::to_string(&spec).unwrap().contains("\"kind\":\"stills\""));
        assert!(named("a-b_9") && !named("") && !named("../x") && !named(".worker"));
        assert!(fresh(&now()) && !fresh("2020-01-01T00:00:00Z") && !fresh("yesterday"));
        let layout = Layout::of(Path::new("/data/captures"), "j1");
        assert_eq!(layout.still(3), PathBuf::from("/data/captures/j1/stills/3.png"));
        assert_eq!(layout.output(Kind::Pdf), PathBuf::from("/data/captures/j1/out.pdf"));
    }
}
