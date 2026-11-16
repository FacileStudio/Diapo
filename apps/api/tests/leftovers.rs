//! Files that outlive their rows: export files and thumbnails go with their deck or their person.

mod common;

use api::auth::AppData;
use api::modules::exports::{leftovers, path};
use api::modules::public::thumbnail;
use common::fixture::{self, NOTED};
use porte::directory::PersonDeleted;
use serde_json::{Value, json};
use uuid::Uuid;

async fn exported(app: &common::App, who: &common::Person, deck: &str) -> Uuid {
    let body = Some(json!({"format": "html"}));
    let job: Value = app.call("POST", &format!("/api/decks/{deck}/exports"), Some(who), body).await.json();
    assert_eq!(job["status"], json!("done"), "{job}");
    job["id"].as_str().unwrap().parse().unwrap()
}

fn pictured(app: &common::App, deck: &str) -> std::path::PathBuf {
    let file = thumbnail::path(&app.state.data_dir, deck.parse().unwrap(), 1);
    std::fs::create_dir_all(file.parent().unwrap()).unwrap();
    std::fs::write(&file, b"\xff\xd8\xff").unwrap();
    file
}

#[tokio::test]
async fn erasing_a_person_removes_their_export_files_and_thumbnails_at_once() {
    let Some(pool) = common::pool("diapo_leftovers_erase").await else { return };
    let app = common::app(pool.clone()).await;
    let roles = fixture::roles(&app).await;
    let personal = fixture::id(&fixture::deck(&app, &roles.member, None, NOTED).await);
    let shared = fixture::id(&fixture::deck(&app, &roles.owner, Some(&roles.space), NOTED).await);
    let (mine, theirs) = (exported(&app, &roles.member, &personal).await, exported(&app, &roles.member, &shared).await);
    let owners = exported(&app, &roles.owner, &shared).await;
    let (thumb, kept) = (pictured(&app, &personal), pictured(&app, &shared));
    let data = app.state.data_dir.as_ref().clone();
    AppData::new(pool, data.clone()).on_person_deleted(common::ISSUER, &roles.member.subject).await.unwrap();
    assert!(!path(&data, mine).exists() && !path(&data, theirs).exists() && !thumb.exists());
    assert!(path(&data, owners).exists() && kept.exists());
}

#[tokio::test]
async fn deleting_a_deck_for_good_removes_its_files_and_the_sweep_takes_the_rest() {
    let Some(pool) = common::pool("diapo_leftovers_delete").await else { return };
    let app = common::app(pool.clone()).await;
    let ada = app.person("ada").await;
    let deck = fixture::id(&fixture::deck(&app, &ada, None, NOTED).await);
    let (job, thumb) = (exported(&app, &ada, &deck).await, pictured(&app, &deck));
    let path_of = format!("/api/decks/{deck}");
    app.call("PATCH", &path_of, Some(&ada), Some(json!({"trashed": true}))).await;
    assert_eq!(app.call("DELETE", &path_of, Some(&ada), None).await.status, 204);
    assert!(!path(&app.state.data_dir, job).exists() && !thumb.exists());
    let other = fixture::id(&fixture::deck(&app, &ada, None, NOTED).await);
    let (kept, live) = (exported(&app, &ada, &other).await, pictured(&app, &other));
    let orphan = path(&app.state.data_dir, Uuid::new_v4());
    std::fs::write(&orphan, "left").unwrap();
    let stray = thumbnail::path(&app.state.data_dir, Uuid::new_v4(), 1);
    std::fs::create_dir_all(stray.parent().unwrap()).unwrap();
    std::fs::write(&stray, "left").unwrap();
    assert_eq!(leftovers::sweep(&pool, &app.state.data_dir).await.unwrap(), 2);
    assert!(!orphan.exists() && !stray.exists() && path(&app.state.data_dir, kept).exists() && live.exists());
}
