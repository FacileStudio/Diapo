use axum::body::Body;
use axum::extract::{Path, Request, State};
use axum::http::{HeaderValue, StatusCode, header};
use axum::response::Response;
use porte::Identity;
use serde::Deserialize;
use tower::ServiceExt;
use tower_http::services::ServeFile;
use tronc::{Error, Result};

use super::service::{self, Job};
use super::{SCOPE, path};
use crate::extract::{Params, id};
use crate::modules::decks::access::{self, Access};
use crate::modules::frame;
use crate::state::AppState;

/// `?token=`: the signature of a download link.
#[derive(Deserialize)]
pub struct TokenQuery {
    token: Option<String>,
}

async fn allowed(state: &AppState, who: Option<Identity>, job: &Job, token: Option<&str>) -> Result<()> {
    let missing = || Error::not_found("export not found");
    match (token, who) {
        (Some(token), _) if state.frame_key.grants(SCOPE, job.id, token, frame::now()) => Ok(()),
        (Some(_), _) => Err(missing()),
        (None, Some(who)) => match access::access(state, who.user_id, job.deck_id).await? {
            Access::None => Err(missing()),
            _ => Ok(()),
        },
        (None, None) => Err(Error::unauthorized("missing auth")),
    }
}

fn dress(response: &mut Response, job: &Job, mime: &'static str) {
    let headers = response.headers_mut();
    headers.insert(header::CONTENT_TYPE, HeaderValue::from_static(mime));
    headers.insert(header::CONTENT_SECURITY_POLICY, HeaderValue::from_static("sandbox; default-src 'none'"));
    headers.insert(header::X_CONTENT_TYPE_OPTIONS, HeaderValue::from_static("nosniff"));
    headers.insert(header::CACHE_CONTROL, HeaderValue::from_static("private, no-store"));
    headers.remove(header::SET_COOKIE);
    if let Ok(value) = HeaderValue::from_str(&format!("attachment; filename=\"{}\"", job.file_name)) {
        headers.insert(header::CONTENT_DISPOSITION, value);
    }
}

fn mime(format: &str) -> &'static str {
    super::Format::named(format).unwrap_or(super::Format::Html).mime()
}

/// `GET /exports/{id}/file`: the file of a finished job, as a download, for a caller who reads
/// its deck or holds a signed link. An HTML export is answered as bytes, never as a page.
pub async fn download(
    State(state): State<AppState>,
    who: Option<Identity>,
    Path(raw): Path<String>,
    Params(query): Params<TokenQuery>,
    request: Request,
) -> Result<Response> {
    let job = service::find(&state, id(&raw)?).await?;
    allowed(&state, who, &job, query.token.as_deref()).await?;
    if job.status != "done" {
        return Err(Error::failed("the export has no file"));
    }
    let file = ServeFile::new(path(&state.data_dir, job.id));
    let answer = file.oneshot(request).await.map_err(|error| Error::internal("failed to read an export", error))?;
    if answer.status() == StatusCode::NOT_FOUND {
        return Err(Error::not_found("the export has expired"));
    }
    let mut response = answer.map(Body::new);
    dress(&mut response, &job, mime(&job.format));
    Ok(response)
}
