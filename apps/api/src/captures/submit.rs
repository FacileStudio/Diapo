use capture::job::{Kind, Layout, Spec, now};
use tronc::{Error, Result};
use uuid::Uuid;

use super::{Ask, Captures, bundle, failed};
use crate::modules::decks::types::DeckRow;
use crate::state::AppState;

impl Captures {
    /// Queues a job of `kind` for `deck`. The whole job directory, bundle and `job.json`, is built
    /// in the staging directory, then renamed into the captures directory in one step: the worker
    /// never sees half a job, and the API never writes into a tree the worker can change. A name
    /// the worker planted in the way makes the rename fail. Answers the job id.
    pub async fn submit(&self, state: &AppState, deck: &DeckRow, kind: Kind, ask: Ask) -> Result<String> {
        let id = Uuid::new_v4().simple().to_string();
        let staged = Layout::of(self.staging(), &id);
        let built = self.stage(state, deck, &staged, (kind, ask)).await;
        let moved = match built {
            Ok(()) => tokio::fs::rename(&staged.dir, self.layout(&id).dir).await.map_err(failed),
            Err(error) => Err(error),
        };
        if moved.is_err() {
            tokio::fs::remove_dir_all(&staged.dir).await.ok();
        }
        moved.map(|()| id)
    }

    async fn stage(&self, state: &AppState, deck: &DeckRow, staged: &Layout, (kind, ask): (Kind, Ask)) -> Result<()> {
        let project = scene::planche::project(&deck.source).map_err(|errors| Error::failed(errors.to_string()))?;
        tokio::fs::create_dir_all(self.staging()).await.map_err(failed)?;
        tokio::fs::create_dir_all(self.dir()).await.map_err(failed)?;
        tokio::fs::create_dir(&staged.dir).await.map_err(failed)?;
        bundle::write(state, deck.id, &project, &staged.bundle()).await?;
        let size = project.config().size;
        let id = staged.dir.file_name().unwrap_or_default().to_string_lossy().into_owned();
        let (width, height, created_at) = (size.width, size.height, now());
        let spec = Spec { id, kind, width, height, steps: ask.steps, scenes: ask.scenes, created_at };
        let text = serde_json::to_vec(&spec).map_err(|error| Error::internal("job.json", error))?;
        tokio::fs::write(staged.spec(), text).await.map_err(failed)
    }
}
