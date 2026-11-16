//! Every HTTP call the CLI makes: the Diapo API under `/api`, and the identity provider's
//! device grant. A mismatch with the server is fixed in this module and nowhere else.

mod assets;
mod auth;
mod decks;
pub mod device;
mod wire;

use serde::de::DeserializeOwned;
use serde_json::Value;
use ureq::http::Request;

pub use assets::RemoteAsset;
pub use auth::{AuthConfig, Me};
pub use decks::{Deck, DeckSummary};
pub(crate) use wire::{agent, parse, run, text};

/// Why a call did not give its answer.
#[derive(Debug, Clone, PartialEq)]
pub enum ApiError {
    /// No HTTP answer came back.
    Unreachable(String),
    /// The server answered with an error status, in the suite envelope when it sent one.
    Refused {
        /// The HTTP status.
        status: u16,
        /// `error.code` of the envelope, or `http_<status>` without one.
        code: String,
        /// `error.message` of the envelope, or the start of the body.
        message: String,
    },
    /// A success answer the CLI cannot read.
    Malformed(String),
}

impl ApiError {
    /// Whether this is the `409 conflict` a stale `rev` gets.
    pub fn is_conflict(&self) -> bool {
        matches!(self, ApiError::Refused { status: 409, .. })
    }
}

/// What a request carries.
pub(crate) enum Payload<'a> {
    /// No body.
    Empty,
    /// A JSON body.
    Json(&'a Value),
    /// A body with its own content type.
    Raw(&'a str, Vec<u8>),
}

/// A Diapo server and the token to call it with.
pub struct Client {
    agent: ureq::Agent,
    server: String,
    token: Option<String>,
}

impl Client {
    /// A client for the server at `server`, a site root such as `https://diapo.facile.studio`.
    pub fn new(server: &str, token: Option<&str>) -> Client {
        Client { agent: agent(), server: server.trim_end_matches('/').to_owned(), token: token.map(str::to_owned) }
    }

    /// The site root this client calls.
    pub fn server(&self) -> &str {
        &self.server
    }

    pub(crate) fn call(&self, method: &str, path: &str, payload: Payload<'_>) -> Result<Vec<u8>, ApiError> {
        let mut request = Request::builder()
            .method(method)
            .uri(format!("{}/api{path}", self.server))
            .header("Accept", "application/json");
        if let Some(token) = &self.token {
            request = request.header("Authorization", format!("Bearer {token}"));
        }
        let (kind, body) = match payload {
            Payload::Empty => (None, Vec::new()),
            Payload::Json(value) => (Some("application/json"), value.to_string().into_bytes()),
            Payload::Raw(kind, bytes) => (Some(kind), bytes),
        };
        if let Some(kind) = kind {
            request = request.header("Content-Type", kind);
        }
        let request = request.body(body).map_err(|error| ApiError::Unreachable(error.to_string()))?;
        run(&self.agent, request)
    }

    pub(crate) fn json<T: DeserializeOwned>(&self, method: &str, path: &str, body: Payload<'_>) -> Result<T, ApiError> {
        parse(&self.call(method, path, body)?)
    }

    /// Reads a list the server may send bare or wrapped in an object.
    pub(crate) fn list<T: DeserializeOwned>(&self, path: &str) -> Result<Vec<T>, ApiError> {
        let items = match self.json::<Value>("GET", path, Payload::Empty)? {
            Value::Object(fields) => {
                fields.into_iter().map(|(_, value)| value).find(Value::is_array).unwrap_or_default()
            }
            other => other,
        };
        serde_json::from_value(items).map_err(|error| ApiError::Malformed(error.to_string()))
    }
}
