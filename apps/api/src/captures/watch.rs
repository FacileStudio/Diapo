use std::time::{Duration, Instant};

use capture::job::{State, Status};
use tronc::{Error, Result};

use super::{Captures, POLL};

impl Captures {
    /// The worker's status of the job `id`, once it has written one.
    pub async fn status(&self, id: &str) -> Option<Status> {
        let text = tokio::fs::read(self.layout(id).status()).await.ok()?;
        serde_json::from_slice(&text).ok()
    }

    /// Waits until the job `id` is over, telling `progress` each status on the way. A job that
    /// is not over within `limit`, or whose worker stops beating, is cancelled and answers `503`.
    pub async fn wait(&self, id: &str, limit: Duration, mut progress: impl FnMut(&Status)) -> Result<Status> {
        let deadline = Instant::now() + limit;
        loop {
            let status = self.status(id).await;
            if let Some(status) = status.as_ref() {
                progress(status);
            }
            match status {
                Some(status) if status.state != State::Running => return Ok(status),
                _ if Instant::now() > deadline || !self.alive().await => break,
                _ => tokio::time::sleep(POLL).await,
            }
        }
        tokio::fs::write(self.layout(id).cancel(), b"").await.ok();
        Err(Error::unavailable("the capture service did not finish in time"))
    }

    /// Removes the job directory `id`. The worker never does.
    pub async fn remove(&self, id: &str) {
        tokio::fs::remove_dir_all(self.layout(id).dir).await.ok();
    }
}
