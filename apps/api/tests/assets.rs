//! Assets: typed by their bytes, named by safe relative paths, capped, listed and deleted.

mod common;

use common::fixture::{self, SOURCE};
use common::upload::{File, PNG, png, upload};
use serde_json::json;

#[tokio::test]
async fn an_upload_is_typed_by_its_bytes_named_and_listed() {
    let Some(pool) = common::pool("diapo_assets_upload").await else { return };
    let app = common::app(pool).await;
    let roles = fixture::roles(&app).await;
    let id = fixture::id(&fixture::deck(&app, &roles.member, Some(&roles.space), SOURCE).await);
    let stored = png(&app, &roles.member, &id, "assets/chart.png").await;
    assert_eq!(stored.status, 201, "{}", stored.text());
    let body = stored.json();
    assert_eq!((body["name"].clone(), body["mime"].clone()), (json!("assets/chart.png"), json!("image/png")));
    assert_eq!(body["url"], json!(format!("/api/decks/{id}/assets/assets/chart.png")));
    let unnamed = File { name: None, file_name: "../../Capture d'écran.png", bytes: PNG };
    assert_eq!(upload(&app, &roles.member, &id, unnamed).await.json()["name"], json!("assets/Capture-d--cran.png"));
    let list = app.call("GET", &format!("/api/decks/{id}/assets"), Some(&roles.owner), None).await.json();
    let names: Vec<&str> = list["assets"].as_array().unwrap().iter().map(|a| a["name"].as_str().unwrap()).collect();
    assert_eq!(names, ["assets/Capture-d--cran.png", "assets/chart.png"]);
    let served = app.call("GET", body["url"].as_str().unwrap(), Some(&roles.admin), None).await;
    assert_eq!((served.status.as_u16(), served.bytes.as_slice()), (200, PNG));
    let gif = File { name: Some("assets/chart.png"), file_name: "x", bytes: b"GIF89a-replaced" };
    assert_eq!(upload(&app, &roles.member, &id, gif).await.json()["mime"], json!("image/gif"));
    let again = app.call("GET", &format!("/api/decks/{id}/assets"), Some(&roles.owner), None).await.json();
    assert_eq!(again["assets"].as_array().unwrap().len(), 2);
    let path = format!("/api/decks/{id}/assets/assets/chart.png");
    assert_eq!(app.call("DELETE", &path, Some(&roles.member), None).await.status, 204);
    assert_eq!(app.call("DELETE", &path, Some(&roles.member), None).await.status, 404);
    assert_eq!(app.call("GET", &path, Some(&roles.member), None).await.status, 404);
}

#[tokio::test]
async fn traversal_and_unsafe_names_are_refused() {
    let Some(pool) = common::pool("diapo_assets_names").await else { return };
    let app = common::app(pool).await;
    let ada = app.person("ada").await;
    let id = fixture::id(&fixture::deck(&app, &ada, None, SOURCE).await);
    let refused = ["../secret.png", "assets/../../etc/passwd", "/etc/passwd", "assets\\a.png", "a b.png", ".env"];
    for name in refused.into_iter().chain(["https://x.test/a.png", "assets//a.png", "a/./b.png"]) {
        let answer = png(&app, &ada, &id, name).await;
        assert_eq!(answer.status, 400, "{name}: {}", answer.text());
        assert_eq!(answer.json()["error"]["code"], json!("invalid_argument"), "{name}");
    }
    let list = app.call("GET", &format!("/api/decks/{id}/assets"), Some(&ada), None).await.json();
    assert_eq!(list, json!({"assets": []}));
    assert!(!app.state.data_dir.join("assets").exists());
    for path in ["assets/..%2F..%2Fetc%2Fpasswd", "..%2Fx.png", "%2e%2e/%2e%2e/etc/passwd"] {
        let answer = app.call("GET", &format!("/api/decks/{id}/assets/{path}"), Some(&ada), None).await;
        assert_eq!(answer.status, 404, "{path}");
    }
}

#[tokio::test]
async fn html_scripts_and_unknown_bytes_are_refused_whatever_they_claim() {
    let Some(pool) = common::pool("diapo_assets_sniff").await else { return };
    let app = common::app(pool).await;
    let ada = app.person("ada").await;
    let id = fixture::id(&fixture::deck(&app, &ada, None, SOURCE).await);
    for bytes in [&b"<!doctype html><script>alert(1)</script>"[..], b"alert(1)", b"%PDF-1.7", b"PK\x03\x04zip"] {
        let file = File { name: Some("assets/x.png"), file_name: "x.png", bytes };
        let answer = upload(&app, &ada, &id, file).await;
        assert_eq!(answer.status, 400, "{}", answer.text());
    }
    let empty = app.call("POST", &format!("/api/decks/{id}/assets"), Some(&ada), None).await;
    assert_eq!(empty.status, 400, "{}", empty.text());
}

#[tokio::test]
async fn an_asset_and_the_assets_of_a_deck_are_capped() {
    let Some(pool) = common::pool("diapo_assets_caps").await else { return };
    let app = common::app_with(pool, &[("ASSET_MAX_BYTES", "64"), ("DECK_ASSETS_MAX_BYTES", "100")]).await;
    let ada = app.person("ada").await;
    let id = fixture::id(&fixture::deck(&app, &ada, None, SOURCE).await);
    let big = [PNG, &[0u8; 64]].concat();
    let file = File { name: Some("assets/big.png"), file_name: "big.png", bytes: &big };
    assert_eq!(upload(&app, &ada, &id, file).await.status, 413);
    let sized = [PNG, &[0u8; 20]].concat();
    for (name, status) in [("a.png", 201), ("a.png", 201), ("b.png", 413)] {
        let file = File { name: Some(name), file_name: name, bytes: &sized };
        assert_eq!(upload(&app, &ada, &id, file).await.status, status, "{name}");
    }
    app.call("PATCH", &format!("/api/decks/{id}"), Some(&ada), Some(json!({"trashed": true}))).await;
    let late = File { name: Some("c.png"), file_name: "c.png", bytes: PNG };
    assert_eq!(upload(&app, &ada, &id, late).await.status, 412);
}

#[tokio::test]
async fn the_sweep_removes_only_old_files_no_row_names() {
    let Some(pool) = common::pool("diapo_assets_sweep").await else { return };
    let app = common::app(pool).await;
    let ada = app.person("ada").await;
    let id = fixture::id(&fixture::deck(&app, &ada, None, SOURCE).await);
    png(&app, &ada, &id, "kept.png").await;
    let dir = &app.state.data_dir;
    let orphan = api::modules::assets::files::store(dir, b"GIF89a-orphan").await.unwrap();
    let young = api::modules::assets::files::store(dir, b"GIF89a-young").await.unwrap();
    let old = std::time::SystemTime::now() - std::time::Duration::from_secs(2 * 60 * 60);
    for sha in [&orphan, &api::modules::assets::files::digest(PNG)] {
        let path = api::modules::assets::files::path(dir, sha);
        std::fs::File::options().write(true).open(path).unwrap().set_modified(old).unwrap();
    }
    assert_eq!(api::modules::assets::sweep::sweep(&app.state).await.unwrap(), 1);
    assert!(!api::modules::assets::files::path(dir, &orphan).exists());
    assert!(api::modules::assets::files::path(dir, &young).exists());
    assert!(api::modules::assets::files::path(dir, &api::modules::assets::files::digest(PNG)).exists());
}
