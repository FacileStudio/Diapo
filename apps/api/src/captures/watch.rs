use std::time::{Duration, Instant};

use capture::job::{State, Status};
use tronc::{Error, Result};

use super::{Captures, POLL};

impl Captures {
    /// The worker's status of the job `id`, once it has written one.
    pub async fn status(&self, id: &str) -> Option<Status> {
        let text = self.read(&self.layout(id).status(), super::safe::SMALL).await.ok()?;
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
        self.cancel(id).await;
        Err(Error::unavailable("the capture service did not finish in time"))
    }

    /// Asks the worker to stop the job `id`, when its directory is still a real one inside the
    /// captures directory. The file is created new: an existing name, a link included, is left.
    pub async fn cancel(&self, id: &str) {
        let layout = self.layout(id);
        if !super::safe::real_dir(self.dir(), &layout.dir) {
            return;
        }
        let mut create = tokio::fs::OpenOptions::new();
        create.write(true).create_new(true);
        create.open(layout.cancel()).await.ok();
    }

    /// Removes the job directory `id`. The worker never does. A link is removed, not followed.
    pub async fn remove(&self, id: &str) {
        tokio::fs::remove_dir_all(self.layout(id).dir).await.ok();
    }
}
