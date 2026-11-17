//! Diapo's handle on its salon rooms, with the calls the rest of the API makes.

use planche::Document;
use salon::{Change, Due, Refusal};
use uuid::Uuid;

use super::space::Live;
use crate::modules::decks::error::DeckError;
use crate::modules::decks::types::Author;
use crate::state::AppState;

/// Every open deck room. One replica only: two would split a room.
#[derive(Clone, Default)]
pub struct Hub(pub(super) salon::Hub<AppState>);

/// A whole new document for a deck, written through REST or MCP.
pub struct Rewrite {
    /// The document the deck must hold.
    pub next: Document,
    /// The rev the writer read.
    pub rev: i64,
    /// Who writes, and through what.
    pub author: Author,
}

fn replaced(rewrite: Rewrite) -> impl FnOnce(&mut Live) -> Result<Option<Vec<u8>>, DeckError> {
    let Rewrite { next, rev, author } = rewrite;
    move |live| {
        if live.trashed {
            return Err(tronc::Error::failed("the deck is in the trash").into());
        }
        if live.rev != rev {
            return Err(DeckError::Stale { rev: live.rev, sent: rev });
        }
        let update = live.replace(&next).map_err(|Refusal(_, reason)| tronc::Error::invalid(reason))?;
        live.origin = Some(author.origin);
        Ok(Some(update))
    }
}

fn stored(live: &Live, saved: bool) -> Result<i64, DeckError> {
    match saved {
        true => Ok(live.rev),
        false => Err(tronc::Error::unavailable("the change is applied but not stored yet: read the deck again").into()),
    }
}

impl Hub {
    /// Makes deck `id` hold `rewrite.next` inside its room, opening it when needed, while the
    /// room is still at `rewrite.rev`: one transaction open editors receive live, stored before the answer.
    /// Answers the new rev. A stale `rev` is refused and changes nothing.
    pub async fn save(&self, state: &AppState, id: Uuid, rewrite: Rewrite) -> Result<i64, DeckError> {
        let author = Some(rewrite.author.user_id);
        let change = Change { due: Due::Snapshot, author, edit: replaced(rewrite), answer: stored };
        self.0.write(state, id, change).await?
    }

    /// Stores and unloads the rooms of `ids`, closing their members with 1001: they reconnect
    /// to a room that reads the deck's new place (trashed, restored, moved).
    pub async fn close(&self, ids: &[Uuid]) {
        self.0.close(ids, 1001, "deck changed").await;
    }

    /// Stores and unloads the rooms of `ids`, closing their members with 1008: the decks are
    /// deleted for good.
    pub async fn delete(&self, ids: &[Uuid]) {
        self.0.close(ids, 1008, "deck deleted").await;
    }

    /// Stores and unloads every room, for a graceful stop.
    pub async fn shutdown(&self) {
        self.0.shutdown(1001, "server restarting").await;
    }

    /// Closes, in every open room, the connections of `users` that lost access.
    pub async fn evict(&self, users: &[i64]) {
        self.0.evict(users).await;
    }
}
