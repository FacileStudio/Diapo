use std::sync::Arc;

use hmac::{Hmac, KeyInit, Mac};
use sha2::Sha256;
use uuid::Uuid;

use super::token::{opened, sealed};

const LABEL: &[u8] = b"diapo/frame-assets/v1";

/// The key that signs the links a caller without a session follows: the asset links of a frame
/// and the download links of an export.
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

    fn signing(&self, message: String) -> Hmac<Sha256> {
        let mut mac = mac(&self.0);
        mac.update(message.as_bytes());
        mac
    }

    /// A token that reads the assets of `deck` until `expires`, in seconds since the epoch:
    /// `<expires>.<hex hmac>`.
    pub fn token(&self, deck: Uuid, expires: u64) -> String {
        sealed(expires, self.signing(format!("{deck}:{expires}")))
    }

    /// Whether `token` was signed by this key for `deck` and still holds at `now`.
    pub fn allows(&self, deck: Uuid, token: &str, now: u64) -> bool {
        opened(token, now).is_some_and(|(expires, signature)| {
            self.signing(format!("{deck}:{expires}")).verify_slice(&signature).is_ok()
        })
    }

    /// A token for `id` in `scope` (such as `export`) until `expires`. A token of one scope never
    /// passes for another, nor for a frame.
    pub fn grant(&self, scope: &str, id: Uuid, expires: u64) -> String {
        sealed(expires, self.signing(format!("{scope}:{id}:{expires}")))
    }

    /// Whether `token` was granted by this key for `id` in `scope` and still holds at `now`.
    pub fn grants(&self, scope: &str, id: Uuid, token: &str, now: u64) -> bool {
        opened(token, now).is_some_and(|(expires, signature)| {
            self.signing(format!("{scope}:{id}:{expires}")).verify_slice(&signature).is_ok()
        })
    }
}
