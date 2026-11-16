//! The API's side of the capture service, against a worker that follows the file protocol:
//! PDF exports, the pictures of animated slides in a PPTX, thumbnails after a save.

mod common;

use common::fixture::{self, NOTED};
use common::worker;
use serde_json::{Value, json};

const RAW: &str = "# One\n\n---\nlayout: raw\nduration: 2\n---\n\n<div id=\"box\">Animated</div>\n";

async fn export(app: &common::App, who: &common::Person, deck: &str, body: Value) -> common::Answer {
    app.call("POST", &format!("/api/decks/{deck}/exports"), Some(who), Some(body)).await
}

async fn finished(app: &common::App, who: &common::Person, job: &Value) -> Value {
    let path = format!("/api/exports/{}", job["id"].as_str().unwrap());
    let mut last = job.clone();
    for _ in 0..200 {
        if matches!(last["status"].as_str(), Some("done" | "failed")) {
            return last;
        }
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
        last = app.call("GET", &path, Some(who), None).await.json();
    }
    panic!("the export did not finish: {last}");
}

#[tokio::test]
async fn a_pdf_export_needs_the_worker_and_comes_back_through_the_job() {
    let Some(pool) = common::pool("diapo_capture_pdf").await else { return };
    let app = common::app(pool).await;
    let ada = app.person("ada").await;
    let id = fixture::id(&fixture::deck(&app, &ada, None, NOTED).await);
    let down = export(&app, &ada, &id, json!({"format": "pdf"})).await;
    assert_eq!((down.status.as_u16(), down.json()["error"]["code"].clone()), (503, json!("unavailable")));
    let _worker = worker::start(&app);
    let queued = export(&app, &ada, &id, json!({"format": "pdf", "steps": true})).await;
    assert_eq!(queued.status, 201, "{}", queued.text());
    let job = finished(&app, &ada, &queued.json()).await;
    assert_eq!(
        (job["status"].clone(), job["steps"].clone(), job["notice"].clone()),
        (json!("done"), json!(true), json!(""))
    );
    let file = app.call("GET", job["download_url"].as_str().unwrap(), Some(&ada), None).await;
    assert_eq!(file.text(), "%PDF-1.7 steps=true bundle=true");
    assert_eq!(file.headers["content-type"], "application/pdf");
    assert_eq!(file.headers["content-disposition"], "attachment; filename=\"Launch-plan-b.pdf\"");
    let left =
        std::fs::read_dir(app.state.captures.dir()).unwrap().flatten().filter(|entry| entry.path().is_dir()).count();
    assert_eq!(left, 0, "a collected job leaves no directory");
}

#[tokio::test]
async fn a_pptx_gets_pictures_of_its_animated_slides_or_says_it_has_none() {
    let Some(pool) = common::pool("diapo_capture_pptx").await else { return };
    let app = common::app(pool).await;
    let ada = app.person("ada").await;
    let id = fixture::id(&fixture::deck(&app, &ada, None, RAW).await);
    let plain = finished(&app, &ada, &export(&app, &ada, &id, json!({"format": "pptx"})).await.json()).await;
    assert_eq!(plain["status"], json!("done"));
    assert!(plain["notice"].as_str().unwrap().contains("placeholders"), "{plain}");
    let _worker = worker::start(&app);
    let pictured = finished(&app, &ada, &export(&app, &ada, &id, json!({"format": "pptx"})).await.json()).await;
    assert_eq!((pictured["status"].clone(), pictured["notice"].clone()), (json!("done"), json!("")));
    assert_ne!(pictured["size"], plain["size"]);
    let text = fixture::id(&fixture::deck(&app, &ada, None, NOTED).await);
    let quick = export(&app, &ada, &text, json!({"format": "pptx"})).await.json();
    assert_eq!((quick["status"].clone(), quick["notice"].clone()), (json!("done"), json!("")));
}

#[tokio::test]
async fn a_save_makes_the_thumbnail_of_the_new_revision_and_drops_the_old_one() {
    let Some(pool) = common::pool("diapo_capture_thumb").await else { return };
    let app = common::app(pool).await;
    let ada = app.person("ada").await;
    let _worker = worker::start(&app);
    let (id, token) = fixture::shared(&app, &ada, None).await;
    let picture = format!("/p/{token}/thumbnail.jpg");
    let served = || async { app.call("GET", &picture, None, None).await.status == 200 };
    worker::until("the first thumbnail", served).await;
    assert_eq!(app.call("GET", &picture, None, None).await.bytes, worker::JPEG);
    let facts = app.call("GET", &format!("/api/public/decks/{token}"), None, None).await.json();
    assert_eq!(facts["thumbnail"], json!(picture));
    for rev in [1, 2] {
        let save = json!({"source": format!("# Rev {rev}\n"), "rev": rev});
        assert_eq!(app.call("PUT", &format!("/api/decks/{id}/source"), Some(&ada), Some(save)).await.status, 200);
    }
    let folder = app.state.data_dir.join("thumbnails").join(&id);
    let kept = || async {
        let names: Vec<String> = std::fs::read_dir(&folder)
            .unwrap()
            .flatten()
            .map(|entry| entry.file_name().to_string_lossy().into_owned())
            .collect();
        names == ["3.jpg"]
    };
    worker::until("the thumbnail of rev 3 alone", kept).await;
}
