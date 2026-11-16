//! Export jobs: one under way per person, and an MP4 waiting on Clap holds up no other format.

mod common;

use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::Duration;

use api::modules::exports::clap::Clap;
use api::modules::exports::tokens::Tokens;
use common::clap_mock::{Mode, mock};
use common::fixture::{self, NOTED};
use serde_json::{Value, json};

#[derive(Default)]
struct Counting(AtomicU32);

#[async_trait::async_trait]
impl Tokens for Counting {
    async fn token(&self, user_id: i64) -> tronc::Result<String> {
        Ok(format!("clap-token-{user_id}-{}", self.0.fetch_add(1, Ordering::Relaxed)))
    }
}

async fn served(prefix: &str, mode: Mode) -> Option<(common::App, common::clap_mock::Mock)> {
    let (url, seen) = mock(mode).await;
    let mut app = common::app(common::pool(prefix).await?).await;
    let clap = Clap::at(&url, Arc::new(Counting::default())).unwrap();
    app.state.clap = Some(Clap { poll: Duration::from_millis(20), ..clap });
    app.router = api::router::build_router(app.state.clone(), &app.kits, None).unwrap();
    Some((app, seen))
}

async fn export(app: &common::App, who: &common::Person, deck: &str, format: &str) -> common::Answer {
    app.call("POST", &format!("/api/decks/{deck}/exports"), Some(who), Some(json!({"format": format}))).await
}

#[tokio::test]
async fn a_person_runs_one_export_at_a_time_and_a_video_blocks_nobody_else() {
    let Some((app, _)) = served("diapo_queue", Mode::Hold).await else { return };
    let (ada, bob) = (app.person("ada").await, app.person("bob").await);
    let (mine, his) = (
        fixture::id(&fixture::deck(&app, &ada, None, NOTED).await),
        fixture::id(&fixture::deck(&app, &bob, None, NOTED).await),
    );
    let video: Value = export(&app, &ada, &mine, "mp4").await.json();
    assert!(matches!(video["status"].as_str(), Some("queued" | "running")), "{video}");
    let again: Value = export(&app, &ada, &mine, "mp4").await.json();
    assert_eq!(again["id"], video["id"]);
    let other = export(&app, &ada, &mine, "html").await;
    assert_eq!((other.status.as_u16(), other.json()["error"]["code"].clone()), (429, json!("rate_limited")));
    let quick: Value = export(&app, &bob, &his, "html").await.json();
    assert_eq!(quick["status"], json!("done"), "an html export waited on a video: {quick}");
}

#[tokio::test]
async fn the_clap_bearer_is_asked_for_again_on_every_call() {
    let Some((app, seen)) = served("diapo_queue_tokens", Mode::Done).await else { return };
    let ada = app.person("ada").await;
    let deck = fixture::id(&fixture::deck(&app, &ada, None, NOTED).await);
    let mut job: Value = export(&app, &ada, &deck, "mp4").await.json();
    for _ in 0..100 {
        if job["status"] == json!("done") {
            break;
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
        job = app.call("GET", &format!("/api/exports/{}", job["id"].as_str().unwrap()), Some(&ada), None).await.json();
    }
    assert_eq!(job["status"], json!("done"), "{job}");
    let bearers = seen.lock().unwrap().bearers.clone();
    let distinct: std::collections::HashSet<&String> = bearers.iter().collect();
    assert!(bearers.len() >= 3 && distinct.len() == bearers.len(), "{bearers:?}");
}
