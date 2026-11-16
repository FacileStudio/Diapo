//! What a share link opens, with no session and no cookie: the deck's public facts, its
//! sandboxed frame, the `/p/{token}` and `/embed/{token}` page shells, oEmbed and the thumbnail.
//! Every route answers `404` for a token that is unknown, revoked or on a trashed deck, and
//! none answers the source or a presenter note.

mod guard;
pub mod info;
pub mod oembed;
pub mod pages;
mod router;
mod site;
pub mod thumbnail;

pub use router::api_routes;
pub use site::root_routes;
