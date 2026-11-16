use axum::Router;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::routing::post;
use porte::Identity;
use serde::Deserialize;
use tronc::Result;
use tronc::json::Json;

use super::service::{self, Link};
use crate::extract::{Params, id};
use crate::state::AppState;

#[derive(Deserialize)]
struct ShareQuery {
    #[serde(default)]
    rotate: bool,
}

async fn create(
    State(state): State<AppState>,
    who: Identity,
    Path(raw): Path<String>,
    Params(query): Params<ShareQuery>,
) -> Result<(StatusCode, Json<Link>)> {
    let (link, created) = service::create(&state, who.user_id, id(&raw)?, query.rotate).await?;
    Ok((if created { StatusCode::CREATED } else { StatusCode::OK }, Json(link)))
}

async fn show(State(state): State<AppState>, who: Identity, Path(raw): Path<String>) -> Result<Json<Link>> {
    service::state(&state, who.user_id, id(&raw)?).await.map(Json)
}

async fn revoke(State(state): State<AppState>, who: Identity, Path(raw): Path<String>) -> Result<StatusCode> {
    service::revoke(&state, who.user_id, id(&raw)?).await.map(|()| StatusCode::NO_CONTENT)
}

/// `POST`, `GET` and `DELETE /decks/{id}/share`, behind `require_auth`, for an editor.
pub fn routes() -> Router<AppState> {
    Router::new().route("/decks/{id}/share", post(create).get(show).delete(revoke))
}
