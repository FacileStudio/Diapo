//! A deck's assets: list, upload, download, delete.

use serde::Deserialize;

use super::{ApiError, Client, Payload};
use crate::sync::sha256;
use crate::url::{encode, encode_path};

/// An asset as the server lists it.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct RemoteAsset {
    /// The name: the relative path the source writes, such as `assets/chart.png`.
    pub name: String,
    /// The size in bytes.
    #[serde(default)]
    pub size: u64,
    /// The SHA-256 of the content, in hex.
    #[serde(default)]
    pub sha256: String,
}

fn part(boundary: &str, disposition: &str) -> Vec<u8> {
    format!("--{boundary}\r\nContent-Disposition: form-data; {disposition}\r\n").into_bytes()
}

fn multipart(name: &str, kind: &str, bytes: &[u8]) -> (String, Vec<u8>) {
    let boundary = format!("diapo-{}", &sha256(bytes)[..32]);
    let file_name = name.rsplit('/').next().unwrap_or(name).replace(['"', '\r', '\n'], "_");
    let mut body = part(&boundary, "name=\"name\"");
    body.extend_from_slice(format!("\r\n{name}\r\n").as_bytes());
    body.extend_from_slice(&part(&boundary, &format!("name=\"file\"; filename=\"{file_name}\"")));
    body.extend_from_slice(format!("Content-Type: {kind}\r\n\r\n").as_bytes());
    body.extend_from_slice(bytes);
    body.extend_from_slice(format!("\r\n--{boundary}--\r\n").as_bytes());
    (format!("multipart/form-data; boundary={boundary}"), body)
}

fn route(id: &str, name: Option<&str>) -> String {
    match name {
        Some(name) => format!("/decks/{}/assets/{}", encode(id), encode_path(name)),
        None => format!("/decks/{}/assets", encode(id)),
    }
}

impl Client {
    /// `GET /api/decks/{id}/assets`.
    pub fn assets(&self, id: &str) -> Result<Vec<RemoteAsset>, ApiError> {
        self.list(&route(id, None))
    }

    /// `POST /api/decks/{id}/assets`: a multipart form with the asset's `name` and its `file`.
    /// The server types the file by its bytes and replaces an asset of the same name.
    pub fn upload_asset(&self, id: &str, name: &str, kind: &str, bytes: &[u8]) -> Result<(), ApiError> {
        let (content_type, body) = multipart(name, kind, bytes);
        self.call("POST", &route(id, None), Payload::Raw(&content_type, body)).map(drop)
    }

    /// `GET /api/decks/{id}/assets/{name}`: the bytes of one asset.
    pub fn download_asset(&self, id: &str, name: &str) -> Result<Vec<u8>, ApiError> {
        self.call("GET", &route(id, Some(name)), Payload::Empty)
    }

    /// `DELETE /api/decks/{id}/assets/{name}`.
    pub fn delete_asset(&self, id: &str, name: &str) -> Result<(), ApiError> {
        self.call("DELETE", &route(id, Some(name)), Payload::Empty).map(drop)
    }
}
