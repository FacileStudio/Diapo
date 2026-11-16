use axum::Router;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::routing::{get, post};
use porte::Identity;
use porte::session::Manager;
use serde::Deserialize;
use tronc::Result;
use tronc::json::Json;

use super::Format;
use super::download::download;
use super::service::{self, Job};
use crate::extract::id;
use crate::state::AppState;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ExportRequest {
    format: Format,
    #[serde(default)]
    steps: bool,
}

async fn create(
    State(state): State<AppState>,
    who: Identity,
    Path(raw): Path<String>,
    Json(body): Json<ExportRequest>,
) -> Result<(StatusCode, Json<Job>)> {
    Ok((StatusCode::CREATED, Json(service::create(&state, who.user_id, id(&raw)?, body.format, body.steps).await?)))
}

async fn show(State(state): State<AppState>, who: Identity, Path(raw): Path<String>) -> Result<Json<Job>> {
    service::read(&state, who.user_id, id(&raw)?).await.map(Json)
}

/// `POST /decks/{id}/exports` and `GET /exports/{id}`, behind `require_auth`.
pub fn routes() -> Router<AppState> {
    Router::new().route("/decks/{id}/exports", post(create)).route("/exports/{id}", get(show))
}

/// `GET /exports/{id}/file`: a session or a signed link, so it sits behind the optional layer.
pub fn download_routes(sessions: &Manager) -> Router<AppState> {
    Router::new().route("/exports/{id}/file", get(download).route_layer(sessions.optional()))
}
