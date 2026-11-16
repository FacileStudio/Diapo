use axum::Router;
use axum::extract::{Path, State};
use axum::http::{HeaderValue, header};
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use porte::Identity;
use porte::session::Manager;
use serde::Deserialize;
use tronc::Error;
use uuid::Uuid;

use super::{TTL, now};
use crate::extract::{Params, id};
use crate::modules::assets::{name, service as assets};
use crate::modules::decks::access::{self, Access};
use crate::modules::decks::error::DeckError;
use crate::modules::decks::service::fetch;
use crate::state::AppState;

/// The `Content-Security-Policy` of every frame. `sandbox` without `allow-same-origin` gives the
/// document an opaque origin: it runs the author's scripts, and cannot read the app's cookies,
/// storage or API. No `connect-src`, so nothing in it reaches the network from script.
pub const POLICY: &str = "sandbox allow-scripts; default-src 'none'; img-src 'self' data: blob:; \
    media-src 'self' blob:; style-src 'unsafe-inline'; script-src 'unsafe-inline'; font-src 'self' data:; \
    base-uri 'none'; form-action 'none'; frame-ancestors 'self'";

#[derive(Deserialize)]
struct FrameQuery {
    scene: Option<usize>,
}

async fn compiled(state: &AppState, deck: Uuid, scene: Option<usize>) -> Result<String, DeckError> {
    let row = fetch(state, deck).await?;
    let project = scene::parse(&row.source).map_err(DeckError::Invalid)?;
    let names = assets::names(state, deck).await?;
    let token = state.frame_key.token(deck, now() + TTL.as_secs());
    let resolve = |path: &str| {
        let (name, fragment) = name::referenced(path);
        names.contains(name).then(|| format!("{}?frame={token}{fragment}", assets::url(deck, name)))
    };
    let options = scene::Options::new().assets(&resolve);
    match scene {
        None => Ok(scene::compile(&project, &options)),
        Some(index) => scene::compile_scene(&project, index, &options)
            .ok_or_else(|| Error::not_found(format!("the deck has no scene {index}")).into()),
    }
}

async fn frame(
    State(state): State<AppState>,
    who: Identity,
    Path(raw): Path<String>,
    Params(query): Params<FrameQuery>,
) -> Result<Response, DeckError> {
    let deck = id(&raw)?;
    access::check(&state, who.user_id, deck, Access::Viewer).await?;
    let html = compiled(&state, deck, query.scene).await?;
    let headers = [
        (header::CONTENT_TYPE, HeaderValue::from_static("text/html; charset=utf-8")),
        (header::CONTENT_SECURITY_POLICY, HeaderValue::from_static(POLICY)),
        (header::X_FRAME_OPTIONS, HeaderValue::from_static("SAMEORIGIN")),
        (header::X_CONTENT_TYPE_OPTIONS, HeaderValue::from_static("nosniff")),
        (header::CACHE_CONTROL, HeaderValue::from_static("no-store")),
        (header::REFERRER_POLICY, HeaderValue::from_static("no-referrer")),
    ];
    Ok((headers, html).into_response())
}

async fn cookieless(mut response: Response) -> Response {
    response.headers_mut().remove(header::SET_COOKIE);
    response
}

/// `GET /decks/{id}/frame?scene=n`, behind `require_auth`: the deck compiled to one HTML
/// document, or its scene `n` (from 0), with [`POLICY`]. The only route that answers user HTML
/// as HTML. The response never carries a cookie, whatever a layer inside it set.
pub fn routes(sessions: &Manager) -> Router<AppState> {
    Router::new()
        .route("/decks/{id}/frame", get(frame).route_layer(sessions.require_auth()))
        .layer(axum::middleware::map_response(cookieless))
}
