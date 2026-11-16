use hmac::{Hmac, Mac};
use sha2::Sha256;

/// `<expires>.<hex hmac>`: the token `mac` signs, good until `expires`.
pub fn sealed(expires: u64, mac: Hmac<Sha256>) -> String {
    let hex: String = mac.finalize().into_bytes().iter().map(|byte| format!("{byte:02x}")).collect();
    format!("{expires}.{hex}")
}

/// The expiry and the signature `token` carries, when it is well formed and not expired at `now`.
pub fn opened(token: &str, now: u64) -> Option<(u64, Vec<u8>)> {
    let (expires, hex) = token.split_once('.')?;
    let expires = expires.parse::<u64>().ok().filter(|expires| *expires > now)?;
    Some((expires, unhex(hex)?))
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
    use uuid::Uuid;

    use crate::modules::frame::FrameKey;

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
        let signature = token.split_once('.').unwrap().1;
        let garbage = format!("1000.{}", "zz".repeat(32));
        for forged in [format!("9999.{signature}"), "1000.".to_owned(), "1000".to_owned(), garbage] {
            assert!(!key.allows(deck, &forged, 0), "{forged}");
        }
    }

    #[test]
    fn a_grant_holds_for_its_scope_and_id_only() {
        let key = FrameKey::derive("");
        let (deck, other) = (Uuid::new_v4(), Uuid::new_v4());
        let export = key.grant("export", deck, 1000);
        assert!(key.grants("export", deck, &export, 999) && !key.grants("export", deck, &export, 1000));
        assert!(!key.allows(deck, &export, 999) && !key.grants("export", deck, &key.token(deck, 1000), 999));
        assert!(!key.grants("other", deck, &export, 999) && !key.grants("export", other, &export, 999));
    }
}
