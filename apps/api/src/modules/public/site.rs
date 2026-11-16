use axum::Router;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::Response;
use axum::routing::get;
use tronc::Result;
use tronc::http::{rate_limit, security_override};

use super::guard::{framable, limiter};
use super::info::shared;
use super::oembed::{self, Embed, Query};
use super::{pages, thumbnail};
use crate::extract::Params;
use crate::modules::frame::render::cookieless;
use crate::state::AppState;

async fn present(State(state): State<AppState>, Path(token): Path<String>) -> Response {
    pages::present(&state, &token).await
}

async fn embed(State(state): State<AppState>, Path(token): Path<String>) -> Response {
    pages::embed(&state, &token).await
}

async fn picture(State(state): State<AppState>, Path(token): Path<String>) -> Result<Response> {
    let deck = shared(&state, &token).await?;
    thumbnail::serve(&state, deck.id, deck.rev).await
}

async fn card(State(state): State<AppState>, Params(query): Params<Query>) -> Result<Response> {
    if query.format.as_deref().is_some_and(|format| format.eq_ignore_ascii_case("xml")) {
        let body = tronc::errors::envelope(tronc::Code::Unavailable.as_str(), "oEmbed is served as JSON only", None);
        return Ok(tronc::json::write(StatusCode::NOT_IMPLEMENTED, &body));
    }
    let card: Embed = oembed::embed(&state, &query).await?;
    Ok(tronc::json::write(StatusCode::OK, &card))
}

/// The public routes at the root: the present page, its thumbnail, the embed page that
/// `ancestors` may frame, and oEmbed. Same limits, no cookie.
pub fn root_routes(ancestors: &[String]) -> std::result::Result<Router<AppState>, String> {
    let open = axum::middleware::from_fn_with_state(framable(ancestors)?, security_override);
    Ok(Router::new()
        .route("/p/{token}", get(present))
        .route("/p/{token}/thumbnail.jpg", get(picture))
        .route("/embed/{token}", get(embed).layer(open))
        .route("/oembed", get(card))
        .route_layer(axum::middleware::from_fn_with_state(limiter(), rate_limit))
        .layer(axum::middleware::map_response(cookieless)))
}
