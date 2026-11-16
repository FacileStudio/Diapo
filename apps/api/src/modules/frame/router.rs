use axum::Router;
use axum::extract::{Path, State};
use axum::http::{HeaderValue, header};
use axum::response::Response;
use axum::routing::get;
use porte::Identity;
use porte::session::Manager;
use serde::Deserialize;

use super::render::{Links, POLICY, compiled, cookieless, respond};
use crate::extract::{Params, id};
use crate::modules::decks::access::{self, Access};
use crate::modules::decks::error::DeckError;
use crate::state::AppState;

/// `?scene=n`: one scene alone, from 0.
#[derive(Deserialize)]
pub struct FrameQuery {
    /// The scene index; the whole deck when absent.
    pub scene: Option<usize>,
}

async fn frame(
    State(state): State<AppState>,
    who: Identity,
    Path(raw): Path<String>,
    Params(query): Params<FrameQuery>,
) -> Result<Response, DeckError> {
    let deck = id(&raw)?;
    access::check(&state, who.user_id, deck, Access::Viewer).await?;
    let mut response = respond(compiled(&state, deck, query.scene, Links::Signed).await?, POLICY);
    response.headers_mut().insert(header::X_FRAME_OPTIONS, HeaderValue::from_static("SAMEORIGIN"));
    Ok(response)
}

/// `GET /decks/{id}/frame?scene=n`, behind `require_auth`: the deck compiled to one HTML
/// document, or its scene `n` (from 0), with [`POLICY`]. With the frame of a shared deck, the
/// only routes that answer user HTML as HTML. The response never carries a cookie, whatever a
/// layer inside it set.
pub fn routes(sessions: &Manager) -> Router<AppState> {
    Router::new()
        .route("/decks/{id}/frame", get(frame).route_layer(sessions.require_auth()))
        .layer(axum::middleware::map_response(cookieless))
}
