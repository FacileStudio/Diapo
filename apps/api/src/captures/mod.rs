//! The API's side of the capture service: it writes a job directory, reads the worker's status
//! and heartbeat, and collects the outputs. It never starts a browser. See `docs/capture.md`.

pub mod bundle;
pub mod thumbs;
mod watch;

use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use capture::job::{HEARTBEAT, Kind, Layout, Spec, fresh, now};
use tronc::{Error, Result};
use uuid::Uuid;

use crate::modules::decks::types::DeckRow;
use crate::state::AppState;

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
#[derive(Debug, Clone)]
pub struct Captures {
    dir: Arc<PathBuf>,
}

fn failed(error: std::io::Error) -> Error {
    Error::internal("the captures directory could not be written", error)
}

impl Captures {
    /// The captures directory under `data_dir`.
    pub fn under(data_dir: &std::path::Path) -> Self {
        Captures { dir: Arc::new(data_dir.join("captures")) }
    }

    /// The directory itself.
    pub fn dir(&self) -> &std::path::Path {
        &self.dir
    }

    /// The files of the job `id`.
    pub fn layout(&self, id: &str) -> Layout {
        Layout::of(&self.dir, id)
    }

    /// Whether the worker wrote its heartbeat in the last half minute.
    pub async fn alive(&self) -> bool {
        tokio::fs::read_to_string(self.dir.join(HEARTBEAT)).await.is_ok_and(|written| fresh(&written))
    }

    /// `503` unless the worker is alive.
    pub async fn require(&self) -> Result<()> {
        match self.alive().await {
            true => Ok(()),
            false => Err(Error::unavailable("the capture service is not running; try again in a moment")),
        }
    }

    /// Queues a job of `kind` for `deck`: the bundle first, then `job.json` by rename, so the
    /// worker never sees half a job. Answers the job id.
    pub async fn submit(&self, state: &AppState, deck: &DeckRow, kind: Kind, ask: Ask) -> Result<String> {
        let project = scene::parse(&deck.source).map_err(|errors| Error::failed(errors.to_string()))?;
        let id = Uuid::new_v4().simple().to_string();
        let layout = self.layout(&id);
        bundle::write(state, deck.id, &project, &layout.bundle()).await?;
        let size = project.config().size;
        let (width, height, created_at) = (size.width, size.height, now());
        let spec = Spec { id: id.clone(), kind, width, height, steps: ask.steps, scenes: ask.scenes, created_at };
        let text = serde_json::to_vec(&spec).map_err(|error| Error::internal("job.json", error))?;
        let partial = layout.dir.join("job.json.tmp");
        tokio::fs::write(&partial, text).await.map_err(failed)?;
        tokio::fs::rename(&partial, layout.spec()).await.map_err(failed)?;
        Ok(id)
    }
}
