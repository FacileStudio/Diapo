//! The shell against a real Postgres: probes, `/version`, sessions, the API reference.

mod common;

use http::StatusCode;
use serde_json::Value;

#[tokio::test]
async fn probes_and_version_answer() {
    let Some(pool) = common::pool("diapo_shell_probes").await else { return };
    let app = common::app(pool).await;
    assert_eq!(app.call("GET", "/health", None, None).await.json()["status"], "ok");
    assert_eq!(app.call("GET", "/api/ready", None, None).await.json()["status"], "ready");
    assert_eq!(app.call("GET", "/version", None, None).await.json()["version"], api::version::VERSION);
    assert_eq!(app.call("GET", "/api/nope", None, None).await.status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn a_session_names_its_person_and_an_anonymous_one_names_nobody() {
    let Some(pool) = common::pool("diapo_shell_session").await else { return };
    let app = common::app(pool).await;
    let anonymous = app.call("GET", "/api/auth/session", None, None).await.json();
    assert_eq!(
        (anonymous["user"].clone(), anonymous["bulle_url"].as_str()),
        (Value::Null, Some("https://bulle.facile.studio"))
    );
    assert_eq!(app.call("GET", "/api/auth/me", None, None).await.status, StatusCode::UNAUTHORIZED);
    let ada = app.person("Ada").await;
    let session = app.call("GET", "/api/auth/session", Some(&ada), None).await;
    assert_eq!(session.status, StatusCode::OK);
    assert!(session.json()["user"]["email"].as_str().unwrap().starts_with("Ada-"));
    assert_eq!(app.call("GET", "/api/auth/me", Some(&ada), None).await.json()["name"], "Ada");
    let assistant = app.call("GET", "/api/assistant", Some(&ada), None).await;
    assert_eq!(assistant.json()["name"], "Bulle");
    assert_eq!(app.call("GET", "/api/assistant", None, None).await.status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn every_mounted_route_is_in_the_api_reference() {
    let Some(served) = common::mcp::served("diapo_shell_apiref").await else { return };
    let router = &served.app.router;
    let beside = ["/mcp", "/.well-known"];
    let missing = tronc::apiref::undocumented(router, &api::apiref::document(), &api::apiref::config(), &beside);
    assert!(missing.is_empty(), "routes missing from the API document: {missing:?}");
    let document = served.app.call("GET", "/docs/openapi.json", None, None).await.json();
    assert_eq!(document["info"]["title"], "Diapo API");
    assert!(document["paths"]["/decks/{id}/source"]["put"].is_object(), "{document}");
}

#[test]
fn the_migrations_are_portes_schema_and_troncs_pending_table_verbatim() {
    assert_eq!(include_str!("../migrations/0001_porte.sql"), porte::pg::SCHEMA);
    assert_eq!(include_str!("../migrations/0003_mcp_actions.sql"), tronc::mcp::PENDING_SCHEMA);
}
