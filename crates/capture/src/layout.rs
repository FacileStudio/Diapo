//! Where the files of one job are.

use std::path::{Path, PathBuf};

use crate::job::Kind;

/// The files of the job directory `dir`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Layout {
    /// The job directory.
    pub dir: PathBuf,
}

impl Layout {
    /// The layout of the job `id` under the captures directory `captures`.
    pub fn of(captures: &Path, id: &str) -> Layout {
        Layout { dir: captures.join(id) }
    }

    /// `bundle/`: `index.html` and every file it loads, by relative path.
    pub fn bundle(&self) -> PathBuf {
        self.dir.join("bundle")
    }

    /// `job.json`.
    pub fn spec(&self) -> PathBuf {
        self.dir.join("job.json")
    }

    /// `status.json`.
    pub fn status(&self) -> PathBuf {
        self.dir.join("status.json")
    }

    /// `cancel`: any file with this name asks the worker to stop.
    pub fn cancel(&self) -> PathBuf {
        self.dir.join("cancel")
    }

    /// The output of a job of `kind`: a file, or the `stills` directory.
    pub fn output(&self, kind: Kind) -> PathBuf {
        self.dir.join(match kind {
            Kind::Thumbnail => "thumbnail.jpg",
            Kind::Pdf => "out.pdf",
            Kind::Stills => "stills",
        })
    }

    /// `stills/<scene>.png`.
    pub fn still(&self, scene: usize) -> PathBuf {
        self.output(Kind::Stills).join(format!("{scene}.png"))
    }
}
