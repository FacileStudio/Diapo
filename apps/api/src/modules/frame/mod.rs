//! The sandboxed frame: a deck compiled to HTML, served with a policy that gives it an opaque
//! origin, and the signed links through which that cookie-less document loads the deck's assets.

mod router;

use std::sync::Arc;
use std::time::Duration;

use hmac::{Hmac, KeyInit, Mac};
use sha2::Sha256;
use uuid::Uuid;

pub use router::{POLICY, routes};

/// How long the asset links of one frame work.
pub const TTL: Duration = Duration::from_secs(60 * 60);

const LABEL: &[u8] = b"diapo/frame-assets/v1";

/// The key that signs frame asset links.
#[derive(Clone)]
pub struct FrameKey(Arc<[u8]>);

fn mac(key: &[u8]) -> Hmac<Sha256> {
    Hmac::<Sha256>::new_from_slice(key).expect("HMAC takes a key of any length")
}

impl FrameKey {
    /// The key derived from `secret` (the app's `PORTE_TOKEN_KEY`) under a label of its own, so
    /// links survive a restart. Without a secret, a random key that lasts as long as the process.
    pub fn derive(secret: &str) -> Self {
        let seed = match secret.is_empty() {
            true => porte::new_token().unwrap_or_else(|_| Uuid::new_v4().to_string()),
            false => secret.to_owned(),
        };
        let mut mac = mac(seed.as_bytes());
        mac.update(LABEL);
        FrameKey(mac.finalize().into_bytes().to_vec().into())
    }

    fn signing(&self, deck: Uuid, expires: u64) -> Hmac<Sha256> {
        let mut mac = mac(&self.0);
        mac.update(format!("{deck}:{expires}").as_bytes());
        mac
    }

    /// A token that reads the assets of `deck` until `expires`, in seconds since the epoch:
    /// `<expires>.<hex hmac>`.
    pub fn token(&self, deck: Uuid, expires: u64) -> String {
        let signature = self.signing(deck, expires).finalize().into_bytes();
        let hex: String = signature.iter().map(|byte| format!("{byte:02x}")).collect();
        format!("{expires}.{hex}")
    }

    /// Whether `token` was signed by this key for `deck` and still holds at `now`.
    pub fn allows(&self, deck: Uuid, token: &str, now: u64) -> bool {
        let Some((expires, hex)) = token.split_once('.') else { return false };
        let Ok(expires) = expires.parse::<u64>() else { return false };
        let Some(signature) = unhex(hex) else { return false };
        expires > now && self.signing(deck, expires).verify_slice(&signature).is_ok()
    }
}

fn unhex(hex: &str) -> Option<Vec<u8>> {
    if hex.len() != 64 || !hex.is_ascii() {
        return None;
    }
    (0..32).map(|index| u8::from_str_radix(&hex[index * 2..index * 2 + 2], 16).ok()).collect()
}

/// Seconds since the epoch.
pub fn now() -> u64 {
    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_or(0, |elapsed| elapsed.as_secs())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_token_reads_one_deck_until_it_expires_and_a_forged_one_is_refused() {
        let key = FrameKey::derive("secret");
        let (deck, other) = (Uuid::new_v4(), Uuid::new_v4());
        let token = key.token(deck, 1000);
        assert!(key.allows(deck, &token, 999));
        assert!(!key.allows(deck, &token, 1000));
        assert!(!key.allows(other, &token, 999));
        assert!(!FrameKey::derive("another").allows(deck, &token, 999));
        assert_eq!(FrameKey::derive("secret").token(deck, 1000), token);
        let key = FrameKey::derive("");
        let token = key.token(deck, 1000);
        let signature = token.split_once('.').unwrap().1;
        let garbage = format!("1000.{}", "zz".repeat(32));
        for forged in [format!("9999.{signature}"), "1000.".to_owned(), "1000".to_owned(), garbage] {
            assert!(!key.allows(deck, &forged, 0), "{forged}");
        }
    }
}
