//! The RFC 8628 device grant at the identity provider (Registre), with the public client the
//! suite's command lines share.

use serde::Deserialize;
use serde::de::DeserializeOwned;
use ureq::http::Request;

use super::{ApiError, agent, parse};
use crate::url::encode;

/// The identity provider of the hosted suite.
pub const ISSUER: &str = "https://sso.facile.studio";

/// The public client Registre registers for the suite's command lines.
pub const CLIENT_ID: &str = "facile-cli";

/// The grant identifier, spelled as RFC 8628 section 3.4 does.
pub const GRANT_TYPE: &str = "urn:ietf:params:oauth:grant-type:device_code";

const SCOPES: &str = "openid email profile";

/// The parts of the provider's OpenID configuration the device grant needs.
#[derive(Debug, Clone, Deserialize)]
pub struct Provider {
    /// Where a device sign-in starts, when the provider has one.
    pub device_authorization_endpoint: Option<String>,
    /// Where the CLI polls for the token.
    pub token_endpoint: String,
    /// The grants the provider offers.
    #[serde(default)]
    pub grant_types_supported: Vec<String>,
}

/// A started device sign-in.
#[derive(Debug, Clone, Deserialize)]
pub struct Authorization {
    /// The secret the CLI polls with.
    pub device_code: String,
    /// The short code the person confirms.
    pub user_code: String,
    /// Where the person approves.
    pub verification_uri: String,
    /// The same, with the code filled in.
    pub verification_uri_complete: Option<String>,
    /// Seconds between two polls.
    pub interval: Option<u64>,
    /// Seconds until the sign-in expires.
    pub expires_in: Option<u64>,
}

/// One answer of the token endpoint.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct Answer {
    /// The access token, once approved.
    pub access_token: Option<String>,
    /// `authorization_pending`, `slow_down`, `access_denied`, `expired_token`, or another error.
    pub error: Option<String>,
    /// The provider's own words for the error.
    pub error_description: Option<String>,
}

fn form<T: DeserializeOwned>(url: &str, fields: &[(&str, &str)]) -> Result<T, ApiError> {
    let pairs: Vec<String> = fields.iter().map(|(key, value)| format!("{key}={}", encode(value))).collect();
    let request = Request::post(url)
        .header("Content-Type", "application/x-www-form-urlencoded")
        .header("Accept", "application/json")
        .body(pairs.join("&").into_bytes())
        .map_err(|error| ApiError::Unreachable(error.to_string()))?;
    let response = agent().run(request).map_err(|error| ApiError::Unreachable(error.to_string()))?;
    let body = response.into_body().read_to_vec().map_err(|error| ApiError::Unreachable(error.to_string()))?;
    parse(&body)
}

/// Reads `<issuer>/.well-known/openid-configuration`.
pub fn provider(issuer: &str) -> Result<Provider, ApiError> {
    let url = format!("{}/.well-known/openid-configuration", issuer.trim_end_matches('/'));
    let request = Request::get(url).body(Vec::new()).map_err(|error| ApiError::Unreachable(error.to_string()))?;
    parse(&super::run(&agent(), request)?)
}

/// Starts a device sign-in at the provider's device authorization endpoint.
pub fn authorize(endpoint: &str) -> Result<Authorization, ApiError> {
    form(endpoint, &[("client_id", CLIENT_ID), ("scope", SCOPES)])
}

/// Asks the token endpoint once whether the person approved. A pending sign-in is an answer
/// with an `error`, not a failure of the call.
pub fn poll(token_endpoint: &str, device_code: &str) -> Result<Answer, ApiError> {
    form(token_endpoint, &[("grant_type", GRANT_TYPE), ("device_code", device_code), ("client_id", CLIENT_ID)])
}
