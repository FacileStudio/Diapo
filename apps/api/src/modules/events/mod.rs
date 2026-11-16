//! Deck change events: one in-process broadcast, read by the SSE route of each open editor.
//! It lives in this process, so the app runs as one replica.

mod router;

use tokio::sync::broadcast;
use uuid::Uuid;

pub use router::routes;

/// How many events wait for a slow reader before it is told it lagged.
pub const BUFFER: usize = 256;

/// One deck changed: it is now at `rev`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Changed {
    /// The deck.
    pub deck: Uuid,
    /// The revision it is at.
    pub rev: i64,
}

/// The broadcast every write tells and every open editor listens to.
#[derive(Debug, Clone)]
pub struct Events {
    sender: broadcast::Sender<Changed>,
}

impl Default for Events {
    fn default() -> Self {
        Events { sender: broadcast::channel(BUFFER).0 }
    }
}

impl Events {
    /// Tells every listener that `deck` is at `rev`. Nobody listening is not an error.
    pub fn changed(&self, deck: Uuid, rev: i64) {
        self.sender.send(Changed { deck, rev }).ok();
    }

    /// A receiver of every event told from now on.
    pub fn listen(&self) -> broadcast::Receiver<Changed> {
        self.sender.subscribe()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn a_listener_hears_what_is_told_after_it_subscribed() {
        let events = Events::default();
        let deck = Uuid::new_v4();
        events.changed(deck, 1);
        let mut listener = events.listen();
        events.changed(deck, 2);
        assert_eq!(listener.recv().await.unwrap(), Changed { deck, rev: 2 });
    }
}
