//! The sandboxed frame: a deck compiled to HTML, served with a policy that gives it an opaque
//! origin, and the signed links through which that cookie-less document loads the deck's assets.

mod key;
pub mod render;
mod router;
mod token;

use std::time::Duration;

pub use key::FrameKey;
pub use render::{POLICY, policy};
pub use router::{FrameQuery, routes};
pub use token::now;

/// How long the asset links of one frame work. Short, so a person who loses access to a deck
/// soon loses its assets too; the editor reloads its frame on every change, which signs again.
pub const TTL: Duration = Duration::from_secs(10 * 60);
