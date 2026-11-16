//! Exports: a deck turned into a file. Each export is a job row and, once done, a file under
//! `DATA_DIR/exports` that is kept one day. HTML and PPTX are built in the process, one job at a
//! time; PDF and MP4 are named and answer `unavailable` until the capture worker exists.

pub mod build;
mod download;
pub mod purge;
mod router;
pub mod service;

use std::path::{Path, PathBuf};
use std::time::Duration;

use schemars::JsonSchema;
use serde::Deserialize;
use uuid::Uuid;

pub use router::{download_routes, routes};

/// How long a signed download link works.
pub const LINK_TTL: Duration = Duration::from_secs(60 * 60);

/// The scope of the tokens that sign download links.
pub const SCOPE: &str = "export";

/// What a deck exports to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum Format {
    /// One self-contained HTML file that opens offline.
    Html,
    /// One page per slide.
    Pdf,
    /// An editable PowerPoint file.
    Pptx,
    /// A video, rendered by Clap.
    Mp4,
}

impl Format {
    /// The name as the API and the file extension spell it.
    pub fn name(self) -> &'static str {
        match self {
            Format::Html => "html",
            Format::Pdf => "pdf",
            Format::Pptx => "pptx",
            Format::Mp4 => "mp4",
        }
    }

    /// Whether this build can produce the format.
    pub fn ready(self) -> bool {
        matches!(self, Format::Html | Format::Pptx)
    }

    /// The media type of the download. An HTML export is a download, never a page of this
    /// origin: it is answered as bytes.
    pub fn mime(self) -> &'static str {
        match self {
            Format::Html => "application/octet-stream",
            Format::Pdf => "application/pdf",
            Format::Pptx => "application/vnd.openxmlformats-officedocument.presentationml.presentation",
            Format::Mp4 => "video/mp4",
        }
    }
}

/// `DATA_DIR/exports/<job id>`: the file of one finished job.
pub fn path(data_dir: &Path, job: Uuid) -> PathBuf {
    data_dir.join("exports").join(job.to_string())
}

/// A file name for a download: the title's letters and digits, then the extension.
pub fn file_name(title: &str, format: Format) -> String {
    let kept = |c: char| if c.is_ascii_alphanumeric() { c } else { '-' };
    let slug: String = title.chars().map(kept).collect();
    let words: Vec<&str> = slug.split('-').filter(|word| !word.is_empty()).collect();
    let stem: String = words.join("-").chars().take(80).collect();
    format!("{}.{}", if stem.is_empty() { "deck" } else { &stem }, format.name())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_download_is_named_after_the_title_in_plain_letters() {
        assert_eq!(file_name("Q3 review: été \"2026\"", Format::Pptx), "Q3-review-t-2026.pptx");
        assert_eq!(file_name("../../etc", Format::Html), "etc.html");
        assert_eq!(file_name("", Format::Html), "deck.html");
    }
}
