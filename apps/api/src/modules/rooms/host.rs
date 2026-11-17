//! Diapo's side of the rooms: how a deck loads, saves, and who may stay in its room.

use std::time::Duration;

use salon::{Caller, Due, Fault, Kind, Policy, Saved};
use tokio::time::Instant;
use tronc::Code;
use uuid::Uuid;

use super::live::alive;
use super::load::load;
use super::persist::persist;
use super::space::Live;
use crate::modules::decks::access::{self, Access};
use crate::modules::decks::error::DeckError;
use crate::state::AppState;

/// How long after the last change a room saves.
const DEBOUNCE: Duration = Duration::from_secs(2);

fn code(kind: Kind) -> Code {
    match kind {
        Kind::NotFound => Code::NotFound,
        Kind::Invalid => Code::InvalidArgument,
        Kind::Failed => Code::FailedPrecondition,
        Kind::Unavailable => Code::Unavailable,
        Kind::Internal => Code::Internal,
    }
}

impl salon::Host for AppState {
    type Space = Live;
    type Error = DeckError;

    fn rooms(&self) -> (&salon::Hub<Self>, Policy) {
        let policy = Policy {
            flush: DEBOUNCE,
            debounce: true,
            retry: DEBOUNCE,
            snapshot_every: usize::MAX,
            snapshot_bytes: usize::MAX,
            snapshot_after: None,
            idle: Duration::from_secs(60),
            recheck: self.socket_recheck,
            blocking: true,
            noun: "deck",
        };
        (&self.hub.0, policy)
    }

    fn fault(fault: Fault) -> DeckError {
        tronc::Error::new(code(fault.kind), fault.message).into()
    }

    async fn open(&self, key: Uuid) -> Result<Live, Fault> {
        match load(self, key).await {
            Ok(Some(live)) => Ok(live),
            Ok(None) => Err(Fault::new(Kind::NotFound, "deck not found")),
            Err(error) => {
                tracing::error!(deck = %key, error = %error, "rooms: failed to load a room");
                Err(Fault::new(Kind::Internal, "failed to load a room"))
            }
        }
    }

    async fn save(&self, key: Uuid, live: &mut Live, _due: Due, author: Option<i64>) -> Saved {
        match persist(&self.pool, key, live, author).await {
            Ok(Some(row)) => {
                (live.rev, live.origin) = (row.rev, None);
                self.events.changed(key, row.rev);
                self.thumbs.request(self, key);
                Saved::Stored
            }
            Ok(None) => Saved::Gone,
            Err(error) => {
                tracing::error!(deck = %key, error = %error, "rooms: save failed, retrying");
                Saved::Retry
            }
        }
    }

    async fn alive(&self, caller: &Caller, opened: Instant) -> Option<(u16, String)> {
        alive(self, caller, opened).await
    }

    async fn lost(&self, key: Uuid, user: i64) -> bool {
        matches!(access::access(self, user, key).await, Ok(Access::None))
    }
}
