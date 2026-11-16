//! The bearer Clap accepts for one person.

use tronc::{Error, Result};

/// Mints the bearer Clap accepts for one person: a Registre token for the audience `clap` with
/// the scope `clap.write`, exchanged from that person's grant.
#[async_trait::async_trait]
pub trait Tokens: Send + Sync {
    /// The token for the porte user `user_id`.
    async fn token(&self, user_id: i64) -> Result<String>;
}

/// porte's token exchange as [`Tokens`].
pub struct Exchanged(pub porte::exchange::Exchanger);

#[async_trait::async_trait]
impl Tokens for Exchanged {
    async fn token(&self, user_id: i64) -> Result<String> {
        let target = porte::exchange::Target::new("clap", ["clap.write"]);
        self.0.token(user_id, &target).await.map_err(Error::from)
    }
}
