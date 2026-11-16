//! A capture worker that plants symbolic links: the API reads none of them, and never writes into
//! a tree the worker can change before it is whole.

mod common;

use common::fixture::{self, NOTED};
use common::worker;
use serde_json::{Value, json};

const RAW: &str = "---\nlayout: raw\nduration: 2\n---\n\n<div>Animated</div>\n";

const SECRET: &str = "SECRET_TOKEN=do-not-leak";

async fn finished(app: &common::App, who: &common::Person, deck: &str, format: &str) -> Value {
    let body = Some(json!({"format": format}));
    let mut job = app.call("POST", &format!("/api/decks/{deck}/exports"), Some(who), body).await.json();
    let path = format!("/api/exports/{}", job["id"].as_str().unwrap());
    for _ in 0..200 {
        if matches!(job["status"].as_str(), Some("done" | "failed")) {
            return job;
        }
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
        job = app.call("GET", &path, Some(who), None).await.json();
    }
    panic!("the export did not finish: {job}");
}

#[tokio::test]
async fn outputs_planted_as_links_are_refused_and_nothing_leaks() {
    let Some(pool) = common::pool("diapo_hostile").await else { return };
    let app = common::app(pool).await;
    let ada = app.person("ada").await;
    let outside = tempfile::tempdir().unwrap();
    let secret = outside.path().join("environ");
    std::fs::write(&secret, SECRET).unwrap();
    std::fs::write(outside.path().join("1.png"), SECRET).unwrap();
    let _worker = worker::start_hostile(&app, secret.clone());
    let text = fixture::id(&fixture::deck(&app, &ada, None, NOTED).await);
    let pdf = finished(&app, &ada, &text, "pdf").await;
    assert_eq!(pdf["status"], json!("failed"), "{pdf}");
    assert!(!pdf.to_string().contains("SECRET"), "{pdf}");
    let raw = fixture::id(&fixture::deck(&app, &ada, None, &format!("# One\n\n{RAW}")).await);
    let pptx = finished(&app, &ada, &raw, "pptx").await;
    assert_eq!(pptx["status"], json!("done"), "{pptx}");
    let file = app.call("GET", pptx["download_url"].as_str().unwrap(), Some(&ada), None).await;
    assert!(!file.bytes.windows(SECRET.len()).any(|window| window == SECRET.as_bytes()));
    let (deck, _) = fixture::shared(&app, &ada, None).await;
    app.state.thumbs.request(&app.state, deck.parse().unwrap());
    tokio::time::sleep(std::time::Duration::from_millis(800)).await;
    assert!(!app.state.data_dir.join("thumbnails").join(&deck).join("1.jpg").exists());
    assert_eq!(std::fs::read_to_string(&secret).unwrap(), SECRET);
}

#[tokio::test]
async fn a_job_reaches_the_worker_whole_and_leaves_nothing_in_staging() {
    let Some(pool) = common::pool("diapo_hostile_stage").await else { return };
    let app = common::app(pool).await;
    let ada = app.person("ada").await;
    let id = fixture::id(&fixture::deck(&app, &ada, None, NOTED).await);
    common::upload::png(&app, &ada, &id, "assets/dot.png").await;
    let row = api::modules::decks::service::fetch(&app.state, id.parse().unwrap()).await.unwrap();
    let kind = capture::job::Kind::Pdf;
    let job = app.state.captures.submit(&app.state, &row, kind, Default::default()).await.unwrap();
    let layout = app.state.captures.layout(&job);
    assert!(layout.spec().is_file() && layout.bundle().join("index.html").is_file());
    assert_eq!(std::fs::read(layout.bundle().join("assets/dot.png")).unwrap(), common::upload::PNG);
    assert_eq!(std::fs::read_dir(app.state.captures.staging()).unwrap().count(), 0);
    let mine = std::fs::symlink_metadata(layout.dir.clone()).unwrap();
    assert!(mine.is_dir());
    std::fs::remove_dir_all(&layout.dir).unwrap();
    std::os::unix::fs::symlink(app.state.data_dir.as_path(), &layout.dir).unwrap();
    app.state.captures.cancel(&job).await;
    assert!(!app.state.data_dir.join("cancel").exists(), "a cancel followed a planted link");
}
