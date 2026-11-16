use axum::Router;
use axum::extract::{DefaultBodyLimit, Multipart, Path, State};
use axum::http::StatusCode;
use axum::routing::{MethodRouter, delete, get};
use porte::Identity;
use porte::session::Manager;
use tronc::json::Json;
use tronc::{Error, Result};

use super::intake::Upload;
use super::serve::serve;
use super::service::{self, Asset, Assets};
use crate::extract::id;
use crate::state::AppState;

#[derive(Default)]
struct Parts {
    name: Option<String>,
    file: Option<(String, Vec<u8>)>,
}

async fn parts(mut multipart: Multipart) -> Result<Parts> {
    let invalid = |error: axum::extract::multipart::MultipartError| match error.status() {
        StatusCode::PAYLOAD_TOO_LARGE => Error::too_large("the upload is too large"),
        _ => Error::invalid(error.body_text()),
    };
    let mut parts = Parts::default();
    while let Some(field) = multipart.next_field().await.map_err(invalid)? {
        match field.name() {
            Some("name") => parts.name = Some(field.text().await.map_err(invalid)?),
            Some("file") => {
                let file_name = field.file_name().unwrap_or_default().to_owned();
                parts.file = Some((file_name, field.bytes().await.map_err(invalid)?.to_vec()));
            }
            _ => {}
        }
    }
    Ok(parts)
}

async fn upload(
    State(state): State<AppState>,
    who: Identity,
    Path(raw): Path<String>,
    multipart: Multipart,
) -> Result<(StatusCode, Json<Asset>)> {
    let deck = id(&raw)?;
    let parts = parts(multipart).await?;
    let Some((file_name, bytes)) = parts.file else {
        return Err(Error::invalid("the multipart body has no `file` part"));
    };
    let file = Upload { name: parts.name.as_deref(), file_name: &file_name, bytes: &bytes };
    Ok((StatusCode::CREATED, Json(service::upload(&state, who.user_id, deck, file).await?)))
}

async fn list(State(state): State<AppState>, who: Identity, Path(raw): Path<String>) -> Result<Json<Assets>> {
    service::list(&state, who.user_id, id(&raw)?).await.map(Json)
}

async fn remove(
    State(state): State<AppState>,
    who: Identity,
    Path((raw, name)): Path<(String, String)>,
) -> Result<StatusCode> {
    service::remove(&state, who.user_id, id(&raw)?, &name).await.map(|()| StatusCode::NO_CONTENT)
}

/// The asset routes. Upload, list and delete sit behind `require_auth`; the upload body is
/// capped 1 MiB over `asset_max`, for the multipart framing. Reading an asset takes a session
/// or a frame link, so it sits behind the optional session layer.
pub fn routes(sessions: &Manager, asset_max: usize) -> Router<AppState> {
    let limit = DefaultBodyLimit::max(asset_max + (1 << 20));
    let collection = get(list).post(upload).layer(limit).route_layer(sessions.require_auth());
    let one = get(serve).route_layer(sessions.optional()).merge(delete_one(sessions));
    Router::new().route("/decks/{id}/assets", collection).route("/decks/{id}/assets/{*name}", one)
}

fn delete_one(sessions: &Manager) -> MethodRouter<AppState> {
    delete(remove).route_layer(sessions.require_auth())
}
