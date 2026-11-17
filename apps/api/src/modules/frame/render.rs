use axum::http::{HeaderValue, header};
use axum::response::{IntoResponse, Response};
use tronc::Error;
use uuid::Uuid;

use super::{TTL, now};
use crate::modules::assets::{name, service as assets};
use crate::modules::decks::error::DeckError;
use crate::modules::decks::service::fetch;
use crate::state::AppState;

/// The `Content-Security-Policy` of a frame only the app's own pages show. `sandbox` without
/// `allow-same-origin` gives the document an opaque origin: it runs the author's scripts, and
/// cannot read the app's cookies, storage or API. No `connect-src`, so nothing in it reaches the
/// network from script.
pub const POLICY: &str = "sandbox allow-scripts; default-src 'none'; img-src 'self' data: blob:; \
    media-src 'self' blob:; style-src 'unsafe-inline'; script-src 'unsafe-inline'; font-src 'self' data:; \
    base-uri 'none'; form-action 'none'; frame-ancestors 'self'";

/// [`POLICY`], with `ancestors` allowed to hold the frame besides the app itself: what the frame
/// of a shared deck carries, since the embed page that shows it sits inside another site.
pub fn policy(ancestors: &[String]) -> String {
    ancestors.iter().fold(POLICY.to_owned(), |policy, origin| format!("{policy} {origin}"))
}

/// Where a frame loads the deck's assets from.
#[derive(Debug, Clone, Copy)]
pub enum Links<'a> {
    /// The asset route of the deck, each link signed for [`TTL`]: the frame of a signed-in reader.
    Signed,
    /// The public asset route of the share link with this token: the frame of a shared deck,
    /// which never shows the deck id.
    Shared(&'a str),
}

impl Links<'_> {
    fn link(&self, state: &AppState, deck: Uuid, name: &str) -> String {
        match self {
            Links::Signed => {
                let token = state.frame_key.token(deck, now() + TTL.as_secs());
                format!("{}?frame={token}", assets::url(deck, name))
            }
            Links::Shared(token) => format!("/api/public/decks/{token}/assets/{name}"),
        }
    }
}

/// The deck compiled to one HTML document, or its scene `scene` (from 0), with every asset the
/// deck has resolved through `links`. Whoever asks: callers check access first.
pub async fn compiled(
    state: &AppState,
    deck: Uuid,
    scene: Option<usize>,
    links: Links<'_>,
) -> Result<String, DeckError> {
    let row = fetch(state, deck).await?;
    let project = scene::planche::project(&row.source).map_err(DeckError::Invalid)?;
    let names = assets::names(state, deck).await?;
    let resolve = |path: &str| {
        let (name, fragment) = name::referenced(path);
        names.contains(name).then(|| format!("{}{fragment}", links.link(state, deck, name)))
    };
    let options = scene::Options::new().assets(&resolve);
    match scene {
        None => Ok(scene::compile(&project, &options)),
        Some(index) => scene::compile_scene(&project, index, &options)
            .ok_or_else(|| Error::not_found(format!("the deck has no scene {index}")).into()),
    }
}

/// `html` as a frame response under `policy`: never cached, never sniffed, no referrer.
pub fn respond(html: String, policy: &str) -> Response {
    let policy = HeaderValue::from_str(policy).unwrap_or(HeaderValue::from_static(POLICY));
    let headers = [
        (header::CONTENT_TYPE, HeaderValue::from_static("text/html; charset=utf-8")),
        (header::CONTENT_SECURITY_POLICY, policy),
        (header::X_CONTENT_TYPE_OPTIONS, HeaderValue::from_static("nosniff")),
        (header::CACHE_CONTROL, HeaderValue::from_static("no-store")),
        (header::REFERRER_POLICY, HeaderValue::from_static("no-referrer")),
    ];
    (headers, html).into_response()
}

/// Marks a public answer that names no caching as not to be stored. Without it a CDN in front
/// of the app caches a `404` for a path that looks static, such as a thumbnail that is not
/// rendered yet, and keeps serving it after the file exists.
pub async fn uncached(mut response: Response) -> Response {
    let headers = response.headers_mut();
    if !headers.contains_key(header::CACHE_CONTROL) {
        headers.insert(header::CACHE_CONTROL, HeaderValue::from_static("no-store"));
    }
    response
}

/// Removes any cookie a layer inside set: a response to a sandboxed or public reader carries none.
pub async fn cookieless(mut response: Response) -> Response {
    response.headers_mut().remove(header::SET_COOKIE);
    response
}

#[cfg(test)]
mod tests {
    #[test]
    fn a_shared_frame_names_its_extra_ancestors_and_stays_sandboxed() {
        let policy = super::policy(&["https://prisme.facile.studio".to_owned()]);
        assert!(policy.starts_with("sandbox allow-scripts; default-src 'none';"), "{policy}");
        assert!(policy.ends_with("frame-ancestors 'self' https://prisme.facile.studio"), "{policy}");
        assert_eq!(super::policy(&[]), super::POLICY);
    }
}
