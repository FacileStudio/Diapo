//! porte wiring: sessions and Registre sign-in.

mod picture;
mod routes;

use porte::oidc::{self, Kit};
use porte::pg;
use porte::session::{self, Manager};
use sqlx::PgPool;

pub use picture::Pictures;
pub use routes::routes;

use crate::env::Config;

/// Every auth kit the app builds once at boot.
#[derive(Clone)]
pub struct Kits {
    /// Sessions, cookies and the auth middleware.
    pub sessions: Manager,
    /// Registre sign-in routes.
    pub oidc: Kit,
}

type BootError = Box<dyn std::error::Error + Send + Sync>;

impl Kits {
    /// Builds the kits. A wrong issuer fails here, so the boot fails.
    pub async fn new(cfg: &Config, pool: &PgPool) -> Result<Kits, BootError> {
        let store = pg::Store::new(pool.clone()).with_token_sealer(pg::TokenSealer::parse(&cfg.token_key)?);
        let sessions = Manager::new(&cfg.porte, session::Deps { sessions: store.sessions(), now: None });
        let deps = oidc::Deps {
            users: Pictures::wrap(store.users()),
            identities: store.identities(),
            codes: store.login_codes(),
            sessions: sessions.clone(),
            config_extra: None,
            http: None,
            now: None,
        };
        let oidc = Kit::new(&cfg.porte, deps).await?;
        Ok(Kits { sessions, oidc })
    }
}
