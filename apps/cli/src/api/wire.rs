//! One request and its answer: the agent, the suite error envelope, and lenient reading.

use std::time::Duration;

use serde::de::DeserializeOwned;
use serde::{Deserialize, Deserializer};
use serde_json::Value;
use ureq::http::Request;

use super::ApiError;

const TIMEOUT: Duration = Duration::from_secs(120);
const BODY_LIMIT: u64 = 1 << 30;

/// The HTTP agent: a timeout, and error statuses answered instead of raised.
pub(crate) fn agent() -> ureq::Agent {
    ureq::Agent::config_builder()
        .timeout_global(Some(TIMEOUT))
        .http_status_as_error(false)
        .user_agent(concat!("diapo/", env!("CARGO_PKG_VERSION")))
        .build()
        .into()
}

fn refusal(status: u16, body: &[u8]) -> ApiError {
    #[derive(Deserialize)]
    struct Envelope {
        error: Detail,
    }
    #[derive(Deserialize)]
    struct Detail {
        #[serde(default)]
        code: String,
        #[serde(default)]
        message: String,
    }
    match serde_json::from_slice::<Envelope>(body) {
        Ok(Envelope { error }) if !error.code.is_empty() => {
            ApiError::Refused { status, code: error.code, message: error.message }
        }
        _ => {
            let text: String = String::from_utf8_lossy(body).chars().take(200).collect();
            ApiError::Refused { status, code: format!("http_{status}"), message: text.trim().to_owned() }
        }
    }
}

/// Sends `request` and answers the body of a success, or the error.
pub(crate) fn run(agent: &ureq::Agent, request: Request<Vec<u8>>) -> Result<Vec<u8>, ApiError> {
    let mut response = agent.run(request).map_err(|error| ApiError::Unreachable(error.to_string()))?;
    let status = response.status().as_u16();
    let body = response
        .body_mut()
        .with_config()
        .limit(BODY_LIMIT)
        .read_to_vec()
        .map_err(|error| ApiError::Unreachable(error.to_string()))?;
    if (200..300).contains(&status) { Ok(body) } else { Err(refusal(status, &body)) }
}

/// Reads a JSON answer.
pub(crate) fn parse<T: DeserializeOwned>(body: &[u8]) -> Result<T, ApiError> {
    serde_json::from_slice(body).map_err(|error| ApiError::Malformed(error.to_string()))
}

/// Reads an id the server may send as a string or as a number.
pub(crate) fn text<'de, D: Deserializer<'de>>(deserializer: D) -> Result<String, D::Error> {
    Ok(match Value::deserialize(deserializer)? {
        Value::String(text) => text,
        other => other.to_string(),
    })
}
