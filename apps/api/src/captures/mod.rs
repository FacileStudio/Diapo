//! The API's side of the capture service: it writes a job directory, reads the worker's status
//! and heartbeat, and collects the outputs. It never starts a browser. See `docs/capture.md`.

pub mod bundle;
pub mod safe;
mod submit;
pub mod thumbs;
mod watch;

use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use capture::job::{HEARTBEAT, Layout, fresh};
use tronc::{Error, Result};

/// How often a waiting task reads `status.json`.
pub const POLL: Duration = Duration::from_millis(250);

/// What one job asks for besides its deck.
#[derive(Debug, Clone, Default)]
pub struct Ask {
    /// A PDF page per step instead of per scene.
    pub steps: bool,
    /// The scenes to shoot, for stills; every scene when absent.
    pub scenes: Option<Vec<usize>>,
}

/// The captures directory, `DATA_DIR/captures`: the one thing the API shares with the worker.
/// Jobs are built in `DATA_DIR/staging`, which the worker cannot see, and moved in whole.
#[derive(Debug, Clone)]
pub struct Captures {
    dir: Arc<PathBuf>,
    staging: Arc<PathBuf>,
}

pub(crate) fn failed(error: std::io::Error) -> Error {
    Error::internal("the captures directory could not be written", error)
}

impl Captures {
    /// The captures directory under `data_dir`.
    pub fn under(data_dir: &std::path::Path) -> Self {
        Captures { dir: Arc::new(data_dir.join("captures")), staging: Arc::new(data_dir.join("staging")) }
    }

    /// The directory itself.
    pub fn dir(&self) -> &std::path::Path {
        &self.dir
    }

    /// Where jobs are built before they are moved into [`Self::dir`].
    pub fn staging(&self) -> &std::path::Path {
        &self.staging
    }

    /// Reads a file the worker wrote: no link followed, a regular file of at most `max` bytes,
    /// inside the captures directory.
    pub async fn read(&self, path: &std::path::Path, max: u64) -> std::io::Result<Vec<u8>> {
        safe::read(&self.dir, path, max).await
    }

    /// The files of the job `id`.
    pub fn layout(&self, id: &str) -> Layout {
        Layout::of(&self.dir, id)
    }

    /// Whether the worker wrote its heartbeat in the last half minute.
    pub async fn alive(&self) -> bool {
        let written = self.read(&self.dir.join(HEARTBEAT), safe::SMALL).await;
        written.is_ok_and(|bytes| fresh(&String::from_utf8_lossy(&bytes)))
    }

    /// `503` unless the worker is alive.
    pub async fn require(&self) -> Result<()> {
        match self.alive().await {
            true => Ok(()),
            false => Err(Error::unavailable("the capture service is not running; try again in a moment")),
        }
    }
}
