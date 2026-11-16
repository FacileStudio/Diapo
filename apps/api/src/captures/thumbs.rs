//! Thumbnails: after a save, the first slide of the deck's new revision is captured and kept at
//! `DATA_DIR/thumbnails/<deck id>/<rev>.jpg`. One job per deck at a time; the latest revision wins.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use capture::job::{Kind, State};
use uuid::Uuid;

use super::Ask;
use crate::modules::decks::service::fetch;
use crate::modules::public::thumbnail;
use crate::state::AppState;

/// How long a deck must stay unsaved before its thumbnail is captured.
pub const DEBOUNCE: Duration = Duration::from_secs(3);

/// How long one thumbnail job may take.
pub const LIMIT: Duration = Duration::from_secs(60);

/// The decks whose thumbnail is being made. The flag says a save arrived meanwhile.
#[derive(Debug, Clone)]
pub struct Thumbs {
    busy: Arc<Mutex<HashMap<Uuid, bool>>>,
    delay: Duration,
}

impl Default for Thumbs {
    fn default() -> Self {
        Thumbs::after(DEBOUNCE)
    }
}

/// Captures the thumbnail of the revision `deck` is at, unless it has one, the deck is in the
/// trash or the worker is down. Answers whether a file was written.
pub async fn make(state: &AppState, deck: Uuid) -> bool {
    let Ok(row) = fetch(state, deck).await else { return false };
    let wanted = row.trashed_at.is_none() && !thumbnail::exists(state, deck, row.rev).await;
    if !wanted || !state.captures.alive().await {
        return false;
    }
    let Ok(job) = state.captures.submit(state, &row, Kind::Thumbnail, Ask::default()).await else { return false };
    let done = state.captures.wait(&job, LIMIT, |_| {}).await.is_ok_and(|status| status.state == State::Done);
    let kept = done && keep(state, &job, deck, row.rev).await.is_ok();
    state.captures.remove(&job).await;
    kept
}

async fn keep(state: &AppState, job: &str, deck: Uuid, rev: i64) -> std::io::Result<()> {
    let target = thumbnail::path(&state.data_dir, deck, rev);
    let folder = target.parent().unwrap_or(&state.data_dir).to_owned();
    tokio::fs::create_dir_all(&folder).await?;
    let partial = folder.join(format!(".{rev}.tmp"));
    tokio::fs::copy(state.captures.layout(job).output(Kind::Thumbnail), &partial).await?;
    tokio::fs::rename(&partial, &target).await?;
    let mut others = tokio::fs::read_dir(&folder).await?;
    while let Some(entry) = others.next_entry().await? {
        if entry.path() != target {
            tokio::fs::remove_file(entry.path()).await.ok();
        }
    }
    Ok(())
}

impl Thumbs {
    /// Thumbnails captured `delay` after the last save.
    pub fn after(delay: Duration) -> Self {
        Thumbs { busy: Default::default(), delay }
    }

    fn claim(&self, deck: Uuid) -> bool {
        let mut busy = self.busy.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
        busy.insert(deck, true).is_none()
    }

    fn settle(&self, deck: Uuid) -> bool {
        let mut busy = self.busy.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
        match busy.get_mut(&deck) {
            Some(dirty) if *dirty => {
                *dirty = false;
                false
            }
            _ => {
                busy.remove(&deck);
                true
            }
        }
    }

    /// Asks for the thumbnail of `deck` at its newest revision. A deck already being captured is
    /// only flagged: its task captures again once it is done, so one job per deck is in flight.
    pub fn request(&self, state: &AppState, deck: Uuid) {
        if !self.claim(deck) {
            return;
        }
        tokio::spawn(follow(self.clone(), state.clone(), deck));
    }
}

async fn follow(thumbs: Thumbs, state: AppState, deck: Uuid) {
    loop {
        tokio::time::sleep(thumbs.delay).await;
        if thumbs.settle(deck) {
            return;
        }
        make(&state, deck).await;
    }
}
