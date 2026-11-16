use std::sync::{Arc, Mutex};

use axum::Router;
use axum::extract::{Multipart, Path, State};
use axum::http::{HeaderMap, StatusCode};
use axum::routing::{get, post};
use serde_json::{Value, json};

pub const VIDEO: &[u8] = b"\0\0\0\x18ftypisom-a-video";

#[derive(Default)]
pub struct Seen {
    pub bearers: Vec<String>,
    pub spec: String,
    pub files: Vec<String>,
    reads: u32,
    fails: bool,
}

pub type Mock = Arc<Mutex<Seen>>;

fn bearer(mock: &Mock, headers: &HeaderMap) {
    let sent = headers.get("authorization").and_then(|value| value.to_str().ok()).unwrap_or_default();
    mock.lock().unwrap().bearers.push(sent.to_owned());
}

async fn queue(State(mock): State<Mock>, headers: HeaderMap, mut body: Multipart) -> (StatusCode, axum::Json<Value>) {
    bearer(&mock, &headers);
    while let Some(part) = body.next_field().await.unwrap() {
        let (name, file) = (part.name().unwrap_or_default().to_owned(), part.file_name().map(str::to_owned));
        let text = String::from_utf8_lossy(&part.bytes().await.unwrap()).into_owned();
        let mut seen = mock.lock().unwrap();
        match (name.as_str(), file) {
            ("spec", _) => seen.spec = text,
            (_, Some(file)) => seen.files.push(file),
            _ => {}
        }
    }
    (StatusCode::ACCEPTED, axum::Json(json!({"id": "r1", "state": "queued", "frame": 0, "frames": 0, "error": null})))
}

async fn read(State(mock): State<Mock>, headers: HeaderMap, Path(id): Path<String>) -> axum::Json<Value> {
    bearer(&mock, &headers);
    let mut seen = mock.lock().unwrap();
    seen.reads += 1;
    let state = match (seen.reads, seen.fails) {
        (1, _) => json!({"id": id, "state": "running", "frame": 3, "frames": 10, "error": null}),
        (_, true) => json!({"id": id, "state": "failed", "frame": 3, "frames": 10, "error": "the scene threw"}),
        (_, false) => json!({"id": id, "state": "done", "frame": 10, "frames": 10, "error": null}),
    };
    axum::Json(state)
}

pub async fn mock(fails: bool) -> (String, Mock) {
    let seen: Mock = Arc::new(Mutex::new(Seen { fails, ..Seen::default() }));
    let router = Router::new()
        .route("/api/renders", post(queue))
        .route("/api/renders/{id}", get(read))
        .route("/api/renders/{id}/file", get(|| async { VIDEO }))
        .with_state(seen.clone());
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let url = format!("http://{}", listener.local_addr().unwrap());
    tokio::spawn(async move { axum::serve(listener, router).await.unwrap() });
    (url, seen)
}
