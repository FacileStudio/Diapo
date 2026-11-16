//! Exports: a job, a file kept one day, a download for a reader of the deck or a signed link.

mod common;

use common::fixture::{self, NOTED};
use common::upload;
use serde_json::json;

async fn export(app: &common::App, who: &common::Person, deck: &str, format: &str) -> common::Answer {
    app.call("POST", &format!("/api/decks/{deck}/exports"), Some(who), Some(json!({"format": format}))).await
}

#[tokio::test]
async fn an_html_export_is_one_offline_file_downloaded_never_served_as_a_page() {
    let Some(pool) = common::pool("diapo_exports_html").await else { return };
    let app = common::app(pool).await;
    let roles = fixture::roles(&app).await;
    let id = fixture::id(&fixture::deck(&app, &roles.member, Some(&roles.space), NOTED).await);
    upload::png(&app, &roles.member, &id, "assets/dot.png").await;
    let made = export(&app, &roles.member, &id, "html").await;
    assert_eq!(made.status, 201, "{}", made.text());
    let job = made.json();
    assert_eq!(
        (job["status"].clone(), job["format"].clone(), job["rev"].clone()),
        (json!("done"), json!("html"), json!(1))
    );
    let url = job["download_url"].as_str().unwrap().to_owned();
    assert_eq!(url, format!("/api/exports/{}/file", job["id"].as_str().unwrap()));
    let file = app.call("GET", &url, Some(&roles.owner), None).await;
    assert_eq!(file.headers["content-type"], "application/octet-stream", "{}", file.text());
    assert_eq!(file.headers["content-disposition"], "attachment; filename=\"Launch-plan-b.html\"");
    assert!(file.headers["content-security-policy"].to_str().unwrap().starts_with("sandbox"));
    let html = file.text();
    assert!(html.starts_with("<!doctype html>") && html.contains("src=\"data:image/png;base64,"), "{html}");
    assert!(!html.contains("/api/") && html.contains("window.__scene"));
    let read =
        app.call("GET", &format!("/api/exports/{}", job["id"].as_str().unwrap()), Some(&roles.admin), None).await;
    assert_eq!(read.json()["size"], json!(html.len()));
    for path in [url.clone(), url.trim_end_matches("/file").to_owned()] {
        assert_eq!(app.call("GET", &path, Some(&roles.outsider), None).await.status, 404, "{path}");
        assert_eq!(app.call("GET", &path, None, None).await.status, 401, "{path}");
    }
    assert_eq!(export(&app, &roles.outsider, &id, "html").await.status, 404);
}

#[tokio::test]
async fn a_pptx_export_downloads_and_a_signed_link_needs_no_session() {
    let Some(pool) = common::pool("diapo_exports_pptx").await else { return };
    let app = common::app(pool).await;
    let ada = app.person("ada").await;
    let id = fixture::id(&fixture::deck(&app, &ada, None, NOTED).await);
    let job = export(&app, &ada, &id, "pptx").await.json();
    assert_eq!((job["status"].clone(), job["file_name"].clone()), (json!("done"), json!("Launch-plan-b.pptx")));
    let job_id: uuid::Uuid = job["id"].as_str().unwrap().parse().unwrap();
    let signed = api::modules::exports::service::signed(&app.state, job_id);
    let file = app.call("GET", &signed, None, None).await;
    assert_eq!(file.status, 200, "{}", file.text());
    assert!(file.bytes.starts_with(b"PK") && file.bytes.len() as u64 == job["size"].as_u64().unwrap());
    let kind = "application/vnd.openxmlformats-officedocument.presentationml.presentation";
    assert_eq!(file.headers["content-type"], kind);
    assert!(file.headers.get(http::header::SET_COOKIE).is_none());
    let other = export(&app, &ada, &id, "html").await.json();
    let stolen = signed.replace(&job_id.to_string(), other["id"].as_str().unwrap());
    assert_eq!(app.call("GET", &stolen, None, None).await.status, 404);
    let frame = app.state.frame_key.token(job_id, api::modules::frame::now() + 60);
    assert_eq!(app.call("GET", &format!("/api/exports/{job_id}/file?token={frame}"), None, None).await.status, 404);
    assert_eq!(app.call("GET", &format!("/api/exports/{job_id}/file?token=1.00"), None, None).await.status, 404);
}

#[tokio::test]
async fn pdf_and_mp4_are_unavailable_and_old_exports_are_purged() {
    let Some(pool) = common::pool("diapo_exports_rest").await else { return };
    let app = common::app(pool).await;
    let ada = app.person("ada").await;
    let id = fixture::id(&fixture::deck(&app, &ada, None, NOTED).await);
    for format in ["pdf", "mp4"] {
        let refused = export(&app, &ada, &id, format).await;
        assert_eq!((refused.status.as_u16(), refused.json()["error"]["code"].clone()), (503, json!("unavailable")));
        assert!(refused.json()["error"]["message"].as_str().unwrap().contains("not"));
    }
    assert_eq!(export(&app, &ada, &id, "gif").await.status, 400);
    let jobs: i64 = sqlx::query_scalar("SELECT count(*) FROM exports WHERE deck_id = $1::uuid")
        .bind(&id)
        .fetch_one(&app.pool)
        .await
        .unwrap();
    assert_eq!(jobs, 0);
    let job = export(&app, &ada, &id, "html").await.json();
    let job_id: uuid::Uuid = job["id"].as_str().unwrap().parse().unwrap();
    let file = api::modules::exports::path(&app.state.data_dir, job_id);
    assert!(file.exists());
    sqlx::query("UPDATE exports SET created_at = now() - interval '2 days' WHERE id = $1")
        .bind(job_id)
        .execute(&app.pool)
        .await
        .unwrap();
    assert!(api::modules::exports::purge::purge(&app.state).await.unwrap() >= 1);
    assert!(!file.exists());
    assert_eq!(app.call("GET", &format!("/api/exports/{job_id}"), Some(&ada), None).await.status, 404);
}
