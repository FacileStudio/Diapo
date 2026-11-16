//! The device sign-in: the person approves a link in any browser, on any machine, and the
//! CLI polls the identity provider. Nothing listens here, so it works over SSH.

use std::thread;
use std::time::{Duration, Instant};

use crate::api::device::{self, Answer, Authorization, GRANT_TYPE, Provider};
use crate::api::{ApiError, Client};
use crate::failure::{Failure, Outcome};
use crate::{browser, ui};

const RETRY: &str = "run `diapo login` again";

/// What one answer of the token endpoint means.
#[derive(Debug, Clone, PartialEq)]
pub enum Poll {
    /// Approved, with the provider's access token.
    Approved(String),
    /// Not approved yet.
    Pending,
    /// Not approved yet, and the CLI must poll five seconds slower.
    SlowDown,
}

/// Reads one answer of the token endpoint: approved, keep waiting, or the reason to stop.
pub fn read(answer: Answer) -> Outcome<Poll> {
    if let Some(token) = answer.access_token.filter(|token| !token.is_empty()) {
        return Ok(Poll::Approved(token));
    }
    let stop = |message: String| Err(Failure::new("login_refused", message));
    match answer.error.as_deref() {
        Some("authorization_pending") => Ok(Poll::Pending),
        Some("slow_down") => Ok(Poll::SlowDown),
        Some("access_denied") => {
            stop(format!("the sign-in was refused in the browser — {RETRY} if that was a mistake"))
        }
        Some("expired_token") => stop(format!("the sign-in link expired before it was approved — {RETRY}")),
        other => {
            let reason = answer.error_description.as_deref().or(other).unwrap_or("no reason given");
            stop(format!("the identity provider refused the sign-in: {reason}"))
        }
    }
}

fn endpoint(provider: &Provider, issuer: &str) -> Outcome<String> {
    let offered = provider.grant_types_supported.iter().any(|grant| grant == GRANT_TYPE);
    provider.device_authorization_endpoint.clone().filter(|_| offered).ok_or_else(|| {
        Failure::new(
            "no_device_login",
            format!("{issuer} does not offer device sign-in — run `diapo login` without --device"),
        )
    })
}

fn wait(token_endpoint: &str, started: &Authorization) -> Outcome<String> {
    let mut interval = Duration::from_secs(started.interval.unwrap_or(5));
    let deadline = Instant::now() + Duration::from_secs(started.expires_in.unwrap_or(600));
    while Instant::now() < deadline {
        thread::sleep(interval);
        let Ok(answer) = device::poll(token_endpoint, &started.device_code) else { continue };
        match read(answer)? {
            Poll::Approved(token) => return Ok(token),
            Poll::SlowDown => interval += Duration::from_secs(5),
            Poll::Pending => {}
        }
    }
    Err(Failure::new("timeout", format!("the sign-in link expired before it was approved — {RETRY}")))
}

fn exchange(client: &Client, access_token: &str) -> Outcome<String> {
    client.device_exchange(access_token).map_err(|error| match error {
        ApiError::Refused { status: 401, .. } => Failure::new(
            "unknown_account",
            format!(
                "the server does not know this account yet — sign in once at {} in a browser, then {RETRY}",
                client.server()
            ),
        ),
        other => other.into(),
    })
}

/// Signs in with the device grant at `issuer` and answers the session token. The provider's
/// access token is traded at once and never stored. With `open`, the link also opens in the
/// browser of this machine when it has one.
pub fn sign_in(client: &Client, issuer: &str, open: bool) -> Outcome<String> {
    let provider = device::provider(issuer)?;
    let started = device::authorize(&endpoint(&provider, issuer)?)?;
    let link = started.verification_uri_complete.as_deref().unwrap_or(&started.verification_uri);
    if !link.starts_with("https://") && !link.starts_with("http://127.0.0.1") {
        return Err(Failure::new("bad_link", format!("the identity provider sent an unusable sign-in link: {link}")));
    }
    ui::step(&format!("Approve this sign-in at {link} (code {})", started.user_code));
    if open {
        browser::open(link);
    }
    exchange(client, &wait(&provider.token_endpoint, &started)?)
}
