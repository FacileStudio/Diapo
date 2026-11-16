//! porte's sign-in routes: discovery, the one-time code exchange, the device exchange, the
//! account behind a token, and sign-out.

use serde::{Deserialize, Serialize};
use serde_json::json;

use super::{ApiError, Client, Payload};
use crate::url::encode;

/// `GET /api/auth/config`: how this server signs people in.
#[derive(Debug, Clone, Copy, PartialEq, Deserialize)]
pub struct AuthConfig {
    /// Whether single sign-on is the only way in.
    #[serde(default)]
    pub sso_only: bool,
    /// Whether single sign-on is configured.
    #[serde(default)]
    pub oidc_enabled: bool,
}

/// `GET /api/auth/me`: the account behind the token.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct Me {
    /// The account's email.
    pub email: String,
    /// The account's display name.
    #[serde(default)]
    pub name: String,
}

#[derive(Deserialize)]
struct Session {
    token: String,
}

fn token(session: Session) -> Result<String, ApiError> {
    match session.token.is_empty() {
        true => Err(ApiError::Malformed("the server returned an empty token".to_owned())),
        false => Ok(session.token),
    }
}

impl Client {
    /// `GET /api/auth/config`.
    pub fn auth_config(&self) -> Result<AuthConfig, ApiError> {
        self.json("GET", "/auth/config", Payload::Empty)
    }

    /// The URL that starts a browser sign-in ending on `127.0.0.1:port` with `state` echoed.
    pub fn login_url(&self, port: u16, state: &str) -> String {
        format!("{}/api/auth/oidc?flow=cli&port={port}&cli_state={}", self.server(), encode(state))
    }

    /// `POST /api/auth/oidc/exchange`: trades the one-time code for a session token.
    pub fn exchange_code(&self, code: &str) -> Result<String, ApiError> {
        token(self.json("POST", "/auth/oidc/exchange", Payload::Json(&json!({ "code": code })))?)
    }

    /// Whether the server mounts the device exchange: porte answers an empty request with
    /// 400 when it does and 404 when it does not.
    pub fn device_mounted(&self) -> Result<bool, ApiError> {
        match self.call("POST", "/auth/oidc/device/exchange", Payload::Json(&json!({}))) {
            Err(ApiError::Refused { status: 404, .. }) => Ok(false),
            Err(ApiError::Unreachable(detail)) => Err(ApiError::Unreachable(detail)),
            _ => Ok(true),
        }
    }

    /// `POST /api/auth/oidc/device/exchange`: trades the identity provider's access token for
    /// a session token.
    pub fn device_exchange(&self, access_token: &str) -> Result<String, ApiError> {
        let body = json!({ "access_token": access_token });
        token(self.json("POST", "/auth/oidc/device/exchange", Payload::Json(&body))?)
    }

    /// `GET /api/auth/me`.
    pub fn me(&self) -> Result<Me, ApiError> {
        self.json("GET", "/auth/me", Payload::Empty)
    }

    /// `POST /api/auth/logout`: ends the session the token names.
    pub fn logout(&self) -> Result<(), ApiError> {
        self.call("POST", "/auth/logout", Payload::Empty).map(drop)
    }
}
