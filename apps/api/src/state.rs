use std::path::PathBuf;
use std::sync::Arc;

use porte::exchange::Exchanger;
use sqlx::PgPool;
use tokio::sync::Semaphore;
use tronc::mcp::Confirmations;

use crate::env::Limits;
use crate::modules::assistant::Bulle;
use crate::modules::events::Events;
use crate::modules::frame::FrameKey;

/// What every handler shares.
#[derive(Clone)]
pub struct AppState {
    /// The database.
    pub pool: PgPool,
    /// The Registre issuer: the `provider` of every identity whose subject is a space member.
    pub issuer: Arc<str>,
    /// Where assets live.
    pub data_dir: Arc<PathBuf>,
    /// Tells open editors a deck changed.
    pub events: Events,
    /// Signs the asset links a sandboxed frame loads without a cookie, and export download links.
    pub frame_key: FrameKey,
    /// The byte caps.
    pub limits: Limits,
    /// The MCP confirmation flow, when `/mcp` is on.
    pub confirmations: Option<Confirmations>,
    /// Mints the person's tokens for other suite apps; `None` without Registre.
    pub exchanger: Option<Exchanger>,
    /// Where the person's assistant lives.
    pub bulle: Bulle,
    /// The origin people reach the app at, such as `https://diapo.facile.studio`: share links,
    /// oEmbed and `og:` tags are built on it.
    pub public_url: Arc<str>,
    /// The origins that may frame `/embed/{token}` besides the app itself.
    pub embed_ancestors: Arc<[String]>,
    /// The built client, whose `index.html` is the shell of the public pages.
    pub client_dir: Arc<PathBuf>,
    /// The one export slot: a single job builds at a time.
    pub exports: Arc<Semaphore>,
}
