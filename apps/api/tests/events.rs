//! The SSE stream: where the deck is, then every change, whichever door wrote it.

mod common;

use axum::body::Body;
use common::fixture::{self, SOURCE};
use http::{Request, header};
use http_body_util::BodyExt;
use serde_json::json;
use tower::ServiceExt;

type Stream = axum::body::Body;

async fn open(app: &common::App, who: &common::Person, deck: &str) -> axum::response::Response {
    let request = Request::get(format!("/api/decks/{deck}/events"))
        .header(header::HOST, "diapo.test")
        .header(header::AUTHORIZATION, format!("Bearer {}", who.token))
        .body(Body::empty())
        .unwrap();
    app.router.clone().oneshot(request).await.unwrap()
}

async fn next(stream: &mut Stream) -> Option<String> {
    let wait = std::time::Duration::from_secs(5);
    let frame = tokio::time::timeout(wait, stream.frame()).await.expect("no event within 5 s")?;
    Some(String::from_utf8(frame.unwrap().into_data().unwrap().to_vec()).unwrap())
}

#[tokio::test]
async fn an_open_stream_hears_where_the_deck_is_then_every_write() {
    let Some(pool) = common::pool("diapo_events").await else { return };
    let app = common::app(pool).await;
    let roles = fixture::roles(&app).await;
    let id = fixture::id(&fixture::deck(&app, &roles.owner, Some(&roles.space), SOURCE).await);
    let other = fixture::id(&fixture::deck(&app, &roles.owner, Some(&roles.space), SOURCE).await);
    let response = open(&app, &roles.member, &id).await;
    assert_eq!(response.status(), 200);
    assert_eq!(response.headers()["content-type"], "text/event-stream");
    let mut stream = response.into_body();
    assert_eq!(next(&mut stream).await.unwrap(), "event: changed\ndata: {\"rev\":1}\n\n");
    let write = |deck: &str, rev: i64| {
        let path = format!("/api/decks/{deck}/source");
        let body = json!({"source": "# Changed\n", "rev": rev});
        let (app, who) = (&app, &roles.owner);
        async move { app.call("PUT", &path, Some(who), Some(body)).await.status }
    };
    assert_eq!(write(&other, 1).await, 200);
    assert_eq!(write(&id, 1).await, 200);
    assert_eq!(next(&mut stream).await.unwrap(), "event: changed\ndata: {\"rev\":2}\n\n");
    assert_eq!(write(&id, 1).await, 409);
    let rename = Some(json!({"title": "Renamed"}));
    assert_eq!(app.call("PATCH", &format!("/api/decks/{id}"), Some(&roles.owner), rename).await.status, 200);
    assert_eq!(next(&mut stream).await.unwrap(), "event: changed\ndata: {\"rev\":2}\n\n");
}

#[tokio::test]
async fn a_stream_is_refused_to_strangers_and_ends_when_access_is_lost() {
    let Some(pool) = common::pool("diapo_events_access").await else { return };
    let app = common::app(pool).await;
    let roles = fixture::roles(&app).await;
    let id = fixture::id(&fixture::deck(&app, &roles.owner, Some(&roles.space), SOURCE).await);
    assert_eq!(open(&app, &roles.outsider, &id).await.status(), 404);
    let mut stream = open(&app, &roles.member, &id).await.into_body();
    assert!(next(&mut stream).await.unwrap().contains("\"rev\":1"));
    sqlx::query("DELETE FROM space_members WHERE subject = $1")
        .bind(&roles.member.subject)
        .execute(&app.pool)
        .await
        .unwrap();
    let body = json!({"source": "# Changed\n", "rev": 1});
    app.call("PUT", &format!("/api/decks/{id}/source"), Some(&roles.owner), Some(body)).await;
    assert_eq!(next(&mut stream).await, None);
}
