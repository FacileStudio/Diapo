//! Decks over REST: create, read, the `rev` rule, the parser's errors, revisions, the trash.

mod common;

use common::fixture::{self, SOURCE, projected};
use common::{App, Person};
use serde_json::{Value, json};

async fn save(app: &App, who: &Person, deck: &str, source: &str, rev: i64) -> common::Answer {
    let body = json!({"source": source, "rev": rev});
    app.call("PUT", &format!("/api/decks/{deck}/source"), Some(who), Some(body)).await
}

async fn revisions(app: &App, deck: &str) -> Vec<(i64, String, String)> {
    sqlx::query_as("SELECT rev, origin, source FROM deck_revisions WHERE deck_id = $1::uuid ORDER BY rev")
        .bind(deck)
        .fetch_all(&app.pool)
        .await
        .unwrap()
}

#[tokio::test]
async fn a_deck_is_created_read_and_listed_with_its_source_and_rev() {
    let Some(pool) = common::pool("diapo_decks_crud").await else { return };
    let app = common::app(pool).await;
    let roles = fixture::roles(&app).await;
    let deck = fixture::deck(&app, &roles.member, Some(&roles.space), SOURCE).await;
    assert_eq!((deck["title"].clone(), deck["rev"].clone()), (json!("Q3 review"), json!(1)));
    assert_eq!((deck["source"].clone(), deck["access"].clone()), (json!(projected(SOURCE)), json!("editor")));
    let id = fixture::id(&deck);
    let read = app.call("GET", &format!("/api/decks/{id}"), Some(&roles.owner), None).await.json();
    assert_eq!((read["source"].clone(), read["access"].clone()), (json!(projected(SOURCE)), json!("manager")));
    let mine = fixture::deck(&app, &roles.member, None, "# Private\n").await;
    assert_eq!((mine["title"].clone(), mine["space_id"].clone()), (json!("Untitled"), Value::Null));
    let all = app.call("GET", "/api/decks", Some(&roles.member), None).await.json();
    let ids: Vec<&str> = all.as_array().unwrap().iter().map(|d| d["id"].as_str().unwrap()).collect();
    assert_eq!(ids, [mine["id"].as_str().unwrap(), id.as_str()]);
    assert!(all[0].get("source").is_none(), "{all}");
    let path = format!("/api/decks?space_id={}", roles.space);
    let shared = app.call("GET", &path, Some(&roles.owner), None).await.json();
    assert_eq!(shared.as_array().unwrap().len(), 1);
    let starter = app.call("POST", "/api/decks", Some(&roles.member), Some(json!({"title": " Plan "}))).await.json();
    assert_eq!((starter["title"].clone(), starter["rev"].clone()), (json!("Plan"), json!(1)));
    assert_eq!(starter["source"], json!(scene::planche::STARTER));
    let untitled = app.call("POST", "/api/decks", Some(&roles.member), Some(json!({}))).await.json();
    assert_eq!(untitled["title"], json!("New deck"));
}

#[tokio::test]
async fn a_stale_rev_is_a_409_that_changes_nothing_and_names_the_current_rev() {
    let Some(pool) = common::pool("diapo_decks_stale").await else { return };
    let app = common::app(pool).await;
    let roles = fixture::roles(&app).await;
    let id = fixture::id(&fixture::deck(&app, &roles.member, Some(&roles.space), SOURCE).await);
    let saved = save(&app, &roles.member, &id, "# Two\n", 1).await;
    assert_eq!((saved.status.as_u16(), saved.json()["rev"].clone()), (200, json!(2)), "{}", saved.text());
    let stale = save(&app, &roles.owner, &id, "# Lost\n", 1).await;
    assert_eq!(stale.status, 409, "{}", stale.text());
    let body = stale.json();
    assert_eq!((body["error"]["code"].clone(), body["rev"].clone()), (json!("already_exists"), json!(2)));
    let read = app.call("GET", &format!("/api/decks/{id}"), Some(&roles.owner), None).await.json();
    assert_eq!((read["source"].clone(), read["rev"].clone()), (json!(projected("# Two\n")), json!(2)));
    assert_eq!(save(&app, &roles.owner, &id, "# Three\n", 2).await.json()["rev"], json!(3));
    let kept = revisions(&app, &id).await;
    let expected = [(1, "cli", SOURCE), (2, "cli", "# Two\n"), (3, "cli", "# Three\n")];
    assert_eq!(kept.len(), expected.len());
    for ((rev, origin, source), (want_rev, want_origin, want)) in kept.iter().zip(expected) {
        assert_eq!((*rev, origin.as_str(), source.clone()), (want_rev, want_origin, projected(want)));
    }
}

#[tokio::test]
async fn an_invalid_source_is_a_422_with_the_line_of_each_problem() {
    let Some(pool) = common::pool("diapo_decks_invalid").await else { return };
    let app = common::app(pool).await;
    let ada = app.person("ada").await;
    let id = fixture::id(&fixture::deck(&app, &ada, None, SOURCE).await);
    let bad = "# A\n\n---\ncolour: red\n---\n\n![](https://x.test/a.png)\n";
    let refused = save(&app, &ada, &id, bad, 1).await;
    assert_eq!(refused.status, 422, "{}", refused.text());
    let body = refused.json();
    assert_eq!(body["error"]["code"], json!("invalid_argument"));
    assert_eq!(body["errors"][0], json!({"line": 4, "message": "unknown key `colour`"}));
    assert_eq!(body["errors"][1]["line"], json!(7));
    let read = app.call("GET", &format!("/api/decks/{id}"), Some(&ada), None).await.json();
    assert_eq!((read["source"].clone(), read["rev"].clone()), (json!(projected(SOURCE)), json!(1)));
    let created = app.call("POST", "/api/decks", Some(&ada), Some(json!({"source": bad}))).await;
    assert_eq!((created.status.as_u16(), created.json()["errors"][0]["line"].clone()), (422, json!(4)));
    assert_eq!(app.call("POST", "/api/decks", Some(&ada), Some(json!({"source": ""}))).await.status, 422);
}

#[tokio::test]
async fn a_source_over_the_cap_is_a_413() {
    let Some(pool) = common::pool("diapo_decks_cap").await else { return };
    let app = common::app_with(pool, &[("SOURCE_MAX_BYTES", "256")]).await;
    let ada = app.person("ada").await;
    let id = fixture::id(&fixture::deck(&app, &ada, None, "# Small\n").await);
    let big = format!("# Big\n\n{}\n", "x".repeat(256));
    assert_eq!(save(&app, &ada, &id, &big, 1).await.status, 413);
    assert_eq!(app.call("POST", "/api/decks", Some(&ada), Some(json!({"source": big}))).await.status, 413);
    let grows = format!("# Medium\n\n{}\n", "y".repeat(80));
    assert!(grows.len() < 256 && projected(&grows).len() > 256);
    assert_eq!(save(&app, &ada, &id, &grows, 1).await.status, 413);
}

#[tokio::test]
async fn a_patch_renames_moves_and_trashes_and_only_a_trashed_deck_is_deleted() {
    let Some(pool) = common::pool("diapo_decks_patch").await else { return };
    let app = common::app(pool).await;
    let roles = fixture::roles(&app).await;
    let id = fixture::id(&fixture::deck(&app, &roles.owner, None, SOURCE).await);
    let path = format!("/api/decks/{id}");
    let renamed = app.call("PATCH", &path, Some(&roles.owner), Some(json!({"title": " Board deck "}))).await.json();
    assert_eq!((renamed["title"].clone(), renamed["rev"].clone()), (json!("Board deck"), json!(1)));
    assert_eq!(app.call("GET", &path, Some(&roles.member), None).await.status, 404);
    let foreign = json!({"space_id": roles.other_space});
    assert_eq!(app.call("PATCH", &path, Some(&roles.owner), Some(foreign)).await.status, 404);
    let moved = app.call("PATCH", &path, Some(&roles.owner), Some(json!({"space_id": roles.space}))).await.json();
    assert_eq!(moved["space_id"], json!(roles.space));
    assert_eq!(app.call("GET", &path, Some(&roles.member), None).await.status, 200);
    assert_eq!(app.call("DELETE", &path, Some(&roles.owner), None).await.status, 412);
    let trashed = app.call("PATCH", &path, Some(&roles.admin), Some(json!({"trashed": true}))).await.json();
    assert!(trashed["trashed_at"].is_string(), "{trashed}");
    assert_eq!(save(&app, &roles.owner, &id, "# Late\n", 1).await.status, 412);
    let bin = app.call("GET", "/api/decks?trashed=true", Some(&roles.member), None).await.json();
    assert_eq!(bin[0]["id"], json!(id));
    let restored = app.call("PATCH", &path, Some(&roles.admin), Some(json!({"trashed": false}))).await.json();
    assert_eq!(restored["trashed_at"], Value::Null);
    app.call("PATCH", &path, Some(&roles.owner), Some(json!({"trashed": true}))).await;
    assert_eq!(app.call("DELETE", &path, Some(&roles.owner), None).await.status, 204);
    assert_eq!(app.call("GET", &path, Some(&roles.owner), None).await.status, 404);
    assert!(revisions(&app, &id).await.is_empty());
}
