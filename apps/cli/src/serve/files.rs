//! The deck folder's files over HTTP.

use std::fs;
use std::path::{Component, Path, PathBuf};

use tiny_http::{Header, Request, Response};

use crate::mime;
use crate::url::decode;

/// Answers `request` with `bytes`, never cached.
pub fn respond(request: Request, status: u16, kind: &str, bytes: Vec<u8>) {
    let mut response = Response::from_data(bytes).with_status_code(status);
    for (name, value) in [("Content-Type", kind), ("Cache-Control", "no-store")] {
        if let Ok(header) = Header::from_bytes(name, value) {
            response.add_header(header);
        }
    }
    let _ = request.respond(response);
}

/// The file a request path names inside `dir`, or `None` for a path that leaves the folder
/// or names a hidden file.
pub fn resolve(dir: &Path, path: &str) -> Option<PathBuf> {
    let decoded = decode(path.trim_start_matches('/'));
    let relative = Path::new(&decoded);
    let visible =
        |part: Component<'_>| matches!(part, Component::Normal(name) if !name.to_string_lossy().starts_with('.'));
    (!decoded.is_empty() && relative.components().all(visible)).then(|| dir.join(relative))
}

/// Answers `request` with the file at `path`, or 404.
pub fn serve(dir: &Path, path: &str, request: Request) {
    let found = resolve(dir, path).and_then(|file| fs::read(&file).ok().map(|bytes| (mime::of(&file), bytes)));
    match found {
        Some((kind, bytes)) => respond(request, 200, kind, bytes),
        None => respond(request, 404, "text/plain; charset=utf-8", b"not found".to_vec()),
    }
}
