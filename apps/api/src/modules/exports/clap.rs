//! MP4 through Clap: the deck's bundle is posted to Clap's render API with a token exchanged for
//! the person, the render is followed, and the video is copied here once it is done.

use std::sync::Arc;
use std::time::{Duration, Instant};

use reqwest::multipart::{Form, Part};
use serde::Deserialize;
use tokio::io::AsyncWriteExt;
use tronc::{Error, Result};
use uuid::Uuid;

use super::tokens::Tokens;
use super::work::{Built, progress};
use super::{build, path};
use crate::extract::db;
use crate::modules::decks::types::DeckRow;
use crate::state::AppState;

/// The frame rate of an exported video.
pub const FPS: u32 = 30;

/// How long a render may take on Clap's side before the export gives up.
pub const LIMIT: Duration = Duration::from_secs(60 * 60);

/// Where Clap is and how to reach it as a person.
#[derive(Clone)]
pub struct Clap {
    /// `CLAP_URL`: Clap's origin, without a trailing slash.
    pub url: String,
    /// The HTTP client. It follows no redirect, so the bearer never leaves Clap's origin.
    pub http: reqwest::Client,
    /// Mints the bearer.
    pub tokens: Arc<dyn Tokens>,
    /// The pause between two reads of the render.
    pub poll: Duration,
}

impl Clap {
    /// Clap at `url`, with bearers from `tokens`.
    pub fn at(url: &str, tokens: Arc<dyn Tokens>) -> std::result::Result<Self, String> {
        let http = reqwest::Client::builder()
            .redirect(reqwest::redirect::Policy::none())
            .connect_timeout(Duration::from_secs(10))
            .build()
            .map_err(|error| error.to_string())?;
        Ok(Clap { url: url.trim_end_matches('/').to_owned(), http, tokens, poll: Duration::from_secs(2) })
    }
}

/// A render as Clap answers it: the fields the export follows.
#[derive(Debug, Deserialize)]
struct Render {
    id: String,
    state: String,
    #[serde(default)]
    frame: u32,
    #[serde(default)]
    frames: u32,
    error: Option<String>,
}

fn down(error: reqwest::Error) -> Error {
    Error::unavailable(format!("Clap could not be reached: {}", error.without_url()))
}

async fn answered(response: reqwest::Response) -> Result<reqwest::Response> {
    let status = response.status();
    if status.is_success() {
        return Ok(response);
    }
    let body: serde_json::Value = response.json().await.unwrap_or_default();
    let said = body["error"]["message"].as_str().unwrap_or("no reason given");
    match status.as_u16() {
        401 | 403 => Err(Error::failed(format!("Clap refused the request ({status}): sign in again, then retry"))),
        429 | 502..=504 => Err(Error::unavailable(format!("Clap is busy or down: {said}"))),
        _ => Err(Error::failed(format!("Clap refused the video: {said}"))),
    }
}

async fn form(state: &AppState, deck: &DeckRow) -> Result<Form> {
    let project = scene::parse(&deck.source).map_err(|errors| Error::failed(errors.to_string()))?;
    let size = project.config().size;
    let spec = serde_json::json!({"width": size.width, "height": size.height, "fps": FPS});
    let html = scene::compile(&project, &scene::Options::new());
    let mut form = Form::new().text("spec", spec.to_string()).part("file", Part::text(html).file_name("index.html"));
    for (name, (_, bytes)) in build::bundle(state, deck.id).await? {
        form = form.part("file", Part::bytes(bytes).file_name(name));
    }
    Ok(form)
}

async fn followed(clap: &Clap, state: &AppState, job: Uuid, (render, token): (&str, &str)) -> Result<()> {
    let deadline = Instant::now() + LIMIT;
    loop {
        let url = format!("{}/api/renders/{render}", clap.url);
        let read: Render = answered(clap.http.get(url).bearer_auth(token).send().await.map_err(down)?)
            .await?
            .json()
            .await
            .map_err(down)?;
        progress(state, job, read.frame, read.frames).await;
        match read.state.as_str() {
            "done" => return Ok(()),
            "failed" | "cancelled" => {
                let reason = read.error.unwrap_or_else(|| read.state.clone());
                return Err(Error::failed(format!("Clap could not render the video: {reason}")));
            }
            _ if Instant::now() > deadline => return Err(Error::unavailable("Clap did not finish the video in time")),
            _ => tokio::time::sleep(clap.poll).await,
        }
    }
}

async fn copied(clap: &Clap, state: &AppState, job: Uuid, (render, token): (&str, &str)) -> Result<i64> {
    let url = format!("{}/api/renders/{render}/file", clap.url);
    let mut response = answered(clap.http.get(url).bearer_auth(token).send().await.map_err(down)?).await?;
    let failed = |error: std::io::Error| Error::internal("failed to store an export", error);
    let target = path(&state.data_dir, job);
    tokio::fs::create_dir_all(target.parent().unwrap_or(&state.data_dir)).await.map_err(failed)?;
    let mut file = tokio::fs::File::create(&target).await.map_err(failed)?;
    let mut size = 0i64;
    while let Some(chunk) = response.chunk().await.map_err(down)? {
        size += chunk.len() as i64;
        file.write_all(&chunk).await.map_err(failed)?;
    }
    file.flush().await.map_err(failed)?;
    Ok(size)
}

/// Renders `deck` to an MP4 on Clap as the person who asked for the export `job`, and writes
/// the video to the export's file. `503` when Clap is not configured, busy or down.
pub async fn render(state: &AppState, job: Uuid, deck: &DeckRow) -> Result<Built> {
    let clap =
        state.clap.as_ref().ok_or_else(|| Error::unavailable("exporting to mp4 is not available on this server"))?;
    let user: Option<i64> = sqlx::query_scalar("SELECT created_by FROM exports WHERE id = $1")
        .bind(job)
        .fetch_one(&state.pool)
        .await
        .map_err(db)?;
    let token = clap.tokens.token(user.ok_or_else(|| Error::failed("the account behind this export is gone"))?).await?;
    let posted =
        clap.http.post(format!("{}/api/renders", clap.url)).bearer_auth(&token).multipart(form(state, deck).await?);
    let queued: Render = answered(posted.send().await.map_err(down)?).await?.json().await.map_err(down)?;
    followed(clap, state, job, (&queued.id, &token)).await?;
    let size = copied(clap, state, job, (&queued.id, &token)).await?;
    Ok(Built { bytes: None, size, notice: String::new() })
}
