//! The first slide as a picture. Nothing writes one yet: the capture worker will put a JPEG at
//! [`path`] for each revision it renders, and every route here starts answering it.

use std::path::{Path, PathBuf};

use axum::body::Body;
use axum::http::{HeaderValue, header};
use axum::response::Response;
use tronc::{Error, Result};
use uuid::Uuid;

use crate::state::AppState;

/// The width of a thumbnail in pixels; its height follows the deck's stage ratio.
pub const WIDTH: u32 = 1280;

/// `DATA_DIR/thumbnails/<deck id>`: the thumbnails of one deck.
pub fn folder(data_dir: &Path, deck: Uuid) -> PathBuf {
    data_dir.join("thumbnails").join(deck.to_string())
}

/// `DATA_DIR/thumbnails/<deck id>/<rev>.jpg`: one file per deck revision.
pub fn path(data_dir: &Path, deck: Uuid, rev: i64) -> PathBuf {
    folder(data_dir, deck).join(format!("{rev}.jpg"))
}

/// The public route of the thumbnail behind the link `token`.
pub fn route(token: &str) -> String {
    format!("/p/{token}/thumbnail.jpg")
}

/// The height of a thumbnail for a stage of `width` by `height`.
pub fn height(width: u32, height: u32) -> u32 {
    (u64::from(WIDTH) * u64::from(height) / u64::from(width.max(1))) as u32
}

/// Whether the revision `rev` of `deck` has its thumbnail.
pub async fn exists(state: &AppState, deck: Uuid, rev: i64) -> bool {
    tokio::fs::try_exists(path(&state.data_dir, deck, rev)).await.unwrap_or(false)
}

/// The thumbnail of the revision `rev` of `deck`, or `404` while none exists.
pub async fn serve(state: &AppState, deck: Uuid, rev: i64) -> Result<Response> {
    let bytes = tokio::fs::read(path(&state.data_dir, deck, rev)).await.map_err(|_| Error::not_found("not found"))?;
    let mut response = Response::new(Body::from(bytes));
    let headers = response.headers_mut();
    headers.insert(header::CONTENT_TYPE, HeaderValue::from_static("image/jpeg"));
    headers.insert(header::X_CONTENT_TYPE_OPTIONS, HeaderValue::from_static("nosniff"));
    headers.insert(header::CACHE_CONTROL, HeaderValue::from_static("private, max-age=60"));
    Ok(response)
}

#[cfg(test)]
mod tests {
    #[test]
    fn a_thumbnail_keeps_the_stage_ratio() {
        assert_eq!(super::height(1920, 1080), 720);
        assert_eq!(super::height(1024, 768), 960);
    }
}
