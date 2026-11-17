//! Live co-editing: one in-memory planche document per open deck, hosted by salon over y-sync.
//! Every update is checked under the `full` profile and must keep the deck's projection within
//! the source cap and compilable; the room stores the Yjs state, the projection and a revision.

mod check;
mod host;
mod hub;
mod live;
mod load;
mod persist;
mod router;
mod space;

use std::time::Duration;

pub use hub::{Hub, Rewrite};
pub use router::routes;
pub use space::Live;

/// How often a socket re-checks its session in production: a logout, a back-channel logout or
/// an expiry closes it within this long.
pub const SOCKET_RECHECK: Duration = Duration::from_secs(30);
