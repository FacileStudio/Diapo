//! Deck rooms: the legacy conversion on load, the checked updates, the debounced save, and REST
//! writes that reach open editors.

mod common;

use std::time::Duration;

use common::fixture::{self, SOURCE, projected};
use common::ysync::Peer;
use common::{App, Person};
use planche::{Edit, Value};
use serde_json::json;

type Stored = (Option<Vec<u8>>, String, i64);

async fn stored(app: &App, id: &str) -> Stored {
    sqlx::query_as("SELECT state, source, rev FROM decks WHERE id = $1::uuid")
        .bind(id)
        .fetch_one(&app.pool)
        .await
        .unwrap()
}

async fn revisions(app: &App, id: &str) -> Vec<(i64, String, String)> {
    sqlx::query_as("SELECT rev, origin, source FROM deck_revisions WHERE deck_id = $1::uuid ORDER BY rev")
        .bind(id)
        .fetch_all(&app.pool)
        .await
        .unwrap()
}

async fn legacy(app: &App, who: &Person) -> String {
    let id: uuid::Uuid = sqlx::query_scalar(
        "INSERT INTO decks (owner_id, title, source, created_by, updated_by) VALUES ($1, 'Q3', $2, $1, $1) RETURNING id",
    )
    .bind(who.user_id)
    .bind(SOURCE)
    .fetch_one(&app.pool)
    .await
    .unwrap();
    id.to_string()
}

async fn put(app: &App, who: &Person, id: &str, source: &str, rev: i64) -> common::Answer {
    let body = json!({"source": source, "rev": rev});
    app.call("PUT", &format!("/api/decks/{id}/source"), Some(who), Some(body)).await
}

#[tokio::test]
async fn a_legacy_deck_compiles_then_converts_on_its_first_load_and_keeps_its_old_source() {
    let Some(pool) = common::pool("diapo_rooms_legacy").await else { return };
    let app = common::app(pool).await;
    let ada = app.person("ada").await;
    let id = legacy(&app, &ada).await;
    let frame = app.call("GET", &format!("/api/decks/{id}/frame"), Some(&ada), None).await;
    assert_eq!(frame.status, 200, "{}", frame.text());
    let mut peer = Peer::new();
    peer.connect(&format!("{}/api/decks/{id}/ws", fixture::serve(&app).await), &ada.token).await;
    assert_eq!(peer.projection(), projected(SOURCE));
    let (state, source, rev) = stored(&app, &id).await;
    assert_eq!((state.is_some(), source, rev), (true, projected(SOURCE), 2));
    let kept = revisions(&app, &id).await;
    let want = [(1, "web".to_owned(), SOURCE.to_owned()), (2, "web".to_owned(), projected(SOURCE))];
    assert_eq!(kept, want);
    let frame = app.call("GET", &format!("/api/decks/{id}/frame"), Some(&ada), None).await;
    assert!(frame.status == 200 && frame.text().contains("Revenue"), "{}", frame.text());
    peer.disconnect();
    app.state.hub.close(&[id.parse().unwrap()]).await;
    let saved = put(&app, &ada, &id, "# After\n", 2).await;
    assert_eq!((saved.status.as_u16(), saved.json()["rev"].clone()), (200, json!(3)), "{}", saved.text());
    assert_eq!(stored(&app, &id).await.1, projected("# After\n"));
}

#[tokio::test]
async fn an_invalid_update_closes_its_sender_with_1008_and_a_valid_one_is_saved() {
    let Some(pool) = common::pool("diapo_rooms_checked").await else { return };
    let app = common::app(pool).await;
    let ada = app.person("ada").await;
    let id = fixture::id(&fixture::deck(&app, &ada, None, scene::planche::STARTER).await);
    let url = format!("{}/api/decks/{id}/ws", fixture::serve(&app).await);
    let mut bad = Peer::new();
    bad.connect(&url, &ada.token).await;
    bad.edit(&[Edit::Set("title1".into(), "x".into(), Value::Str("far".into()))]).await;
    bad.pump(Duration::from_millis(800)).await;
    assert_eq!(bad.closed, Some(1008));
    let mut good = Peer::new();
    good.connect(&url, &ada.token).await;
    assert_eq!(good.projection(), scene::planche::STARTER);
    good.edit(&[Edit::Set("title1".into(), "x".into(), Value::Number(200.0))]).await;
    good.pump(Duration::from_millis(2600)).await;
    assert_eq!(good.closed, None);
    let (_, source, rev) = stored(&app, &id).await;
    assert!(source.contains("x=\"200\"") && rev == 2, "{rev} {source}");
    assert_eq!(
        revisions(&app, &id).await.last().map(|(rev, origin, _)| (*rev, origin.clone())),
        Some((2, "web".into()))
    );
}

#[tokio::test]
async fn a_rest_save_reaches_an_open_editor_live_and_a_stale_rev_still_changes_nothing() {
    let Some(pool) = common::pool("diapo_rooms_rest").await else { return };
    let app = common::app(pool).await;
    let ada = app.person("ada").await;
    let id = fixture::id(&fixture::deck(&app, &ada, None, scene::planche::STARTER).await);
    let mut editor = Peer::new();
    editor.connect(&format!("{}/api/decks/{id}/ws", fixture::serve(&app).await), &ada.token).await;
    let mut events = app.state.events.listen();
    let saved = put(&app, &ada, &id, "# Live\n", 1).await;
    assert_eq!((saved.status.as_u16(), saved.json()["rev"].clone()), (200, json!(2)), "{}", saved.text());
    assert_eq!(events.recv().await.unwrap().rev, 2);
    editor.pump(Duration::from_millis(500)).await;
    assert_eq!(editor.projection(), projected("# Live\n"));
    let stale = put(&app, &ada, &id, "# Lost\n", 1).await;
    assert_eq!((stale.status.as_u16(), stale.json()["rev"].clone()), (409, json!(2)));
    let (_, source, rev) = stored(&app, &id).await;
    assert_eq!((source, rev), (projected("# Live\n"), 2));
    let kept = revisions(&app, &id).await.into_iter().map(|(rev, origin, _)| (rev, origin)).collect::<Vec<_>>();
    assert_eq!(kept, [(1, "cli".to_owned()), (2, "cli".to_owned())]);
}

#[tokio::test]
async fn a_stranger_cannot_open_a_room() {
    let Some(pool) = common::pool("diapo_rooms_access").await else { return };
    let app = common::app(pool).await;
    let ada = app.person("ada").await;
    let eve = app.person("eve").await;
    let id = fixture::id(&fixture::deck(&app, &ada, None, scene::planche::STARTER).await);
    assert_eq!(app.call("GET", &format!("/api/decks/{id}/ws"), Some(&eve), None).await.status, 404);
}
