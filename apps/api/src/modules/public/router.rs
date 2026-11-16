use axum::Router;
use axum::extract::{Path, Request, State};
use axum::response::Response;
use axum::routing::get;
use tronc::Result;
use tronc::http::{rate_limit, security_override};
use tronc::json::Json;

use super::guard::{framable, limiter, loose};
use super::info::{self, PublicDeck, shared};
use crate::extract::Params;
use crate::modules::assets::serve::send;
use crate::modules::decks::error::DeckError;
use crate::modules::frame::FrameQuery;
use crate::modules::frame::render::{Links, compiled, cookieless, policy, respond};
use crate::state::AppState;

async fn deck(State(state): State<AppState>, Path(token): Path<String>) -> Result<Json<PublicDeck>> {
    info::deck(&state, &token).await.map(Json)
}

async fn frame(
    State(state): State<AppState>,
    Path(token): Path<String>,
    Params(query): Params<FrameQuery>,
) -> std::result::Result<Response, DeckError> {
    let deck = shared(&state, &token).await?;
    let html = compiled(&state, deck.id, query.scene, Links::Shared(&token)).await?;
    Ok(respond(html, &policy(&state.embed_ancestors)))
}

async fn asset(
    State(state): State<AppState>,
    Path((token, name)): Path<(String, String)>,
    request: Request,
) -> Result<Response> {
    let deck = crate::modules::share::service::resolve(&state, &token).await?;
    send(&state, deck, &name, true, request).await
}

/// The public routes under `/api`: the deck's facts, its sandboxed frame and the assets that
/// frame loads. No session, no cookie; 5 requests a second per client with a burst of 60 on
/// the first two. The frame may sit inside
/// `ancestors`, since the embed page that shows it does.
pub fn api_routes(ancestors: &[String]) -> std::result::Result<Router<AppState>, String> {
    let open = axum::middleware::from_fn_with_state(framable(ancestors)?, security_override);
    Ok(Router::new()
        .route("/public/decks/{token}", get(deck))
        .route("/public/decks/{token}/frame", get(frame).layer(open))
        .route_layer(axum::middleware::from_fn_with_state(limiter(), rate_limit))
        .route(
            "/public/decks/{token}/assets/{*name}",
            get(asset).layer(axum::middleware::from_fn_with_state(loose(), rate_limit)),
        )
        .layer(axum::middleware::map_response(cookieless)))
}
