//! MP4 through Clap, against a mock that follows Clap's documented `POST /api/renders` contract.

mod common;

use std::sync::Arc;
use std::time::Duration;

use api::modules::exports::clap::Clap;
use api::modules::exports::tokens::Tokens;
use common::clap_mock::{VIDEO, mock};
use common::fixture::{self, NOTED};
use common::upload;
use serde_json::{Value, json};

struct Fixed;

#[async_trait::async_trait]
impl Tokens for Fixed {
    async fn token(&self, user_id: i64) -> tronc::Result<String> {
        Ok(format!("clap-token-{user_id}"))
    }
}

async fn started(prefix: &str, clap: Option<&str>) -> Option<common::App> {
    let mut app = common::app(common::pool(prefix).await?).await;
    app.state.clap =
        clap.map(|url| Clap { poll: Duration::from_millis(20), ..Clap::at(url, Arc::new(Fixed)).unwrap() });
    app.router = api::router::build_router(app.state.clone(), &app.kits, None).unwrap();
    Some(app)
}

async fn exported(app: &common::App, who: &common::Person, deck: &str) -> Value {
    let body = Some(json!({"format": "mp4"}));
    let queued = app.call("POST", &format!("/api/decks/{deck}/exports"), Some(who), body).await;
    assert_eq!(queued.status, 201, "{}", queued.text());
    let path = format!("/api/exports/{}", queued.json()["id"].as_str().unwrap());
    let mut job = queued.json();
    for _ in 0..200 {
        if matches!(job["status"].as_str(), Some("done" | "failed")) {
            break;
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
        job = app.call("GET", &path, Some(who), None).await.json();
    }
    job
}

#[tokio::test]
async fn an_mp4_export_posts_the_bundle_to_clap_as_the_person_and_brings_the_video_back() {
    let (url, seen) = mock(false).await;
    let Some(app) = started("diapo_clap", Some(&url)).await else { return };
    let ada = app.person("ada").await;
    let id = fixture::id(&fixture::deck(&app, &ada, None, NOTED).await);
    upload::png(&app, &ada, &id, "assets/dot.png").await;
    let job = exported(&app, &ada, &id).await;
    assert_eq!((job["status"].clone(), job["file_name"].clone()), (json!("done"), json!("Launch-plan-b.mp4")), "{job}");
    assert_eq!(
        (job["done"].clone(), job["total"].clone(), job["size"].clone()),
        (json!(10), json!(10), json!(VIDEO.len()))
    );
    let file = app.call("GET", job["download_url"].as_str().unwrap(), Some(&ada), None).await;
    assert_eq!((file.bytes.as_slice(), file.headers["content-type"].to_str().unwrap()), (VIDEO, "video/mp4"));
    let seen = seen.lock().unwrap();
    let expected = format!("Bearer clap-token-{}", ada.user_id);
    assert!(seen.bearers.len() >= 3 && seen.bearers.iter().all(|sent| *sent == expected), "{:?}", seen.bearers);
    assert_eq!(serde_json::from_str::<Value>(&seen.spec).unwrap(), json!({"width": 1920, "height": 1080, "fps": 30}));
    assert_eq!(seen.files, ["index.html", "assets/dot.png"]);
}

#[tokio::test]
async fn a_render_clap_fails_fails_the_export_and_no_clap_is_a_503() {
    let (url, _) = mock(true).await;
    let Some(app) = started("diapo_clap_fail", Some(&url)).await else { return };
    let ada = app.person("ada").await;
    let id = fixture::id(&fixture::deck(&app, &ada, None, NOTED).await);
    let job = exported(&app, &ada, &id).await;
    assert_eq!(job["status"], json!("failed"));
    assert!(job["error"].as_str().unwrap().contains("the scene threw"), "{job}");
    assert_eq!(
        app.call("GET", &format!("/api/exports/{}/file", job["id"].as_str().unwrap()), Some(&ada), None).await.status,
        412
    );
    let Some(off) = started("diapo_clap_off", None).await else { return };
    let bob = off.person("bob").await;
    let deck = fixture::id(&fixture::deck(&off, &bob, None, NOTED).await);
    let refused =
        off.call("POST", &format!("/api/decks/{deck}/exports"), Some(&bob), Some(json!({"format": "mp4"}))).await;
    assert_eq!((refused.status.as_u16(), refused.json()["error"]["code"].clone()), (503, json!("unavailable")));
    let gone = Clap { poll: Duration::from_millis(20), ..Clap::at("http://127.0.0.1:1", Arc::new(Fixed)).unwrap() };
    let mut down = off;
    down.state.clap = Some(gone);
    down.router = api::router::build_router(down.state.clone(), &down.kits, None).unwrap();
    let job = exported(&down, &bob, &deck).await;
    assert!(
        job["status"] == json!("failed") && job["error"].as_str().unwrap().contains("could not be reached"),
        "{job}"
    );
}
