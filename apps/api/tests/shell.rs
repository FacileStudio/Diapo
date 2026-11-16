//! The wave A shell against a real Postgres: probes, `/version`, sessions, the API reference.

use api::auth::Kits;
use api::env::Config;
use api::state::AppState;
use axum::Router;
use axum::body::Body;
use http::{Request, StatusCode, header};
use http_body_util::BodyExt;
use sqlx::PgPool;
use tower::ServiceExt;
use tronc::env::Vars;

struct App {
    router: Router,
    kits: Kits,
    pool: PgPool,
}

async fn app(prefix: &str) -> Option<App> {
    let Some(pool) = tronc::testdb::pool(prefix, &api::MIGRATOR).await else {
        eprintln!("{}", tronc::testdb::skip_reason("mise run db-up"));
        return None;
    };
    let config = Config::from_vars(&Vars::from_pairs(vec![("DATABASE_URL", "postgres://unused/x")])).unwrap();
    let kits = Kits::new(&config, &pool).await.unwrap();
    let router = api::router::build_router(AppState { pool: pool.clone() }, &kits).unwrap();
    Some(App { router, kits, pool })
}

async fn call(app: &App, method: &str, path: &str, token: Option<&str>) -> (StatusCode, serde_json::Value) {
    let mut request = Request::builder().method(method).uri(path).header(header::HOST, "diapo.test");
    if let Some(token) = token {
        request = request.header(header::AUTHORIZATION, format!("Bearer {token}"));
    }
    let response = app.router.clone().oneshot(request.body(Body::empty()).unwrap()).await.unwrap();
    let status = response.status();
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    (status, serde_json::from_slice(&bytes).unwrap_or(serde_json::Value::Null))
}

#[tokio::test]
async fn probes_and_version_answer() {
    let Some(app) = app("diapo_shell_probes").await else { return };
    assert_eq!(call(&app, "GET", "/health", None).await.1["status"], "ok");
    assert_eq!(call(&app, "GET", "/api/ready", None).await.1["status"], "ready");
    assert_eq!(call(&app, "GET", "/version", None).await.1["version"], api::version::VERSION);
    assert_eq!(call(&app, "GET", "/api/nope", None).await.0, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn a_session_names_its_person_and_an_anonymous_one_names_nobody() {
    let Some(app) = app("diapo_shell_session").await else { return };
    assert_eq!(call(&app, "GET", "/api/auth/session", None).await.1["user"], serde_json::Value::Null);
    assert_eq!(call(&app, "GET", "/api/auth/me", None).await.0, StatusCode::UNAUTHORIZED);
    let user_id: i64 = sqlx::query_scalar("INSERT INTO porte_users (email, name) VALUES ($1, 'Ada') RETURNING id")
        .bind("ada@example.test")
        .fetch_one(&app.pool)
        .await
        .unwrap();
    let (token, _) = app.kits.sessions.issue(user_id, "test").await.unwrap();
    let (status, body) = call(&app, "GET", "/api/auth/session", Some(&token)).await;
    assert_eq!((status, body["user"]["email"].as_str()), (StatusCode::OK, Some("ada@example.test")));
    assert_eq!(call(&app, "GET", "/api/auth/me", Some(&token)).await.1["name"], "Ada");
}

#[tokio::test]
async fn every_mounted_route_is_in_the_api_reference() {
    let Some(app) = app("diapo_shell_apiref").await else { return };
    let missing = tronc::apiref::undocumented(&app.router, &api::apiref::document(), &api::apiref::config(), &[]);
    assert!(missing.is_empty(), "routes missing from the API document: {missing:?}");
    assert_eq!(call(&app, "GET", "/docs/openapi.json", None).await.1["info"]["title"], "Diapo API");
}

#[test]
fn the_first_migration_is_portes_schema_verbatim() {
    assert_eq!(include_str!("../migrations/0001_porte.sql"), porte::pg::SCHEMA);
}
