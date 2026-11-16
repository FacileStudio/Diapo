#![allow(dead_code)]

pub mod clap_mock;
pub mod directory;
pub mod fixture;
pub mod http;
pub mod mcp;
pub mod upload;
pub mod worker;

use std::path::PathBuf;
use std::sync::Arc;

use api::auth::Kits;
use api::env::Config;
use api::state::AppState;
use axum::Router;
use sqlx::PgPool;
use tronc::env::Vars;

pub use http::{Answer, send};

pub const ISSUER: &str = "https://sso.test";

pub const HOST: &str = "https://diapo.test";

pub struct App {
    pub router: Router,
    pub state: AppState,
    pub kits: Kits,
    pub pool: PgPool,
}

pub struct Person {
    pub user_id: i64,
    pub subject: String,
    pub token: String,
}

pub async fn pool(prefix: &str) -> Option<PgPool> {
    let pool = tronc::testdb::pool(prefix, &api::MIGRATOR).await;
    if pool.is_none() {
        eprintln!("{}", tronc::testdb::skip_reason("mise run db-up"));
    }
    pool
}

pub fn data_dir() -> PathBuf {
    let dir = std::env::temp_dir().join(format!("diapo-test-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

pub const SHELL: &str =
    "<!doctype html>\n<html><head><title>Diapo</title></head><body><div id=\"app\"></div></body></html>\n";

pub fn client_dir() -> PathBuf {
    let dir = data_dir();
    std::fs::write(dir.join("index.html"), SHELL).unwrap();
    dir
}

pub async fn app(pool: PgPool) -> App {
    app_with(pool, &[]).await
}

pub async fn app_with(pool: PgPool, vars: &[(&str, &str)]) -> App {
    let mut pairs = vec![("DATABASE_URL", "postgres://unused/x"), ("PUBLIC_URL", HOST)];
    pairs.extend_from_slice(vars);
    let config = Config::from_vars(&Vars::from_pairs(pairs)).unwrap();
    let kits = Kits::new(&config, &pool).await.unwrap();
    let data = data_dir();
    let state = AppState {
        issuer: Arc::from(ISSUER),
        captures: api::captures::Captures::under(&data),
        thumbs: api::captures::thumbs::Thumbs::after(std::time::Duration::from_millis(50)),
        data_dir: Arc::new(data),
        client_dir: Arc::new(client_dir()),
        ..api::server::state(&config, pool.clone(), &kits)
    };
    let router = api::router::build_router(state.clone(), &kits, None).unwrap();
    App { router, state, kits, pool }
}

impl App {
    pub async fn person(&self, name: &str) -> Person {
        let subject = format!("{name}-{}", uuid::Uuid::new_v4());
        let user_id: i64 = sqlx::query_scalar("INSERT INTO porte_users (email, name) VALUES ($1, $2) RETURNING id")
            .bind(format!("{subject}@example.test"))
            .bind(name)
            .fetch_one(&self.pool)
            .await
            .unwrap();
        sqlx::query("INSERT INTO porte_identities (user_id, provider, subject) VALUES ($1, $2, $3)")
            .bind(user_id)
            .bind(ISSUER)
            .bind(&subject)
            .execute(&self.pool)
            .await
            .unwrap();
        let (token, _) = self.kits.sessions.issue(user_id, "test").await.unwrap();
        Person { user_id, subject, token }
    }

    pub async fn space(&self, members: &[(&Person, &str)]) -> String {
        let id = format!("space-{}", uuid::Uuid::new_v4());
        sqlx::query("INSERT INTO spaces (id, name) VALUES ($1, $1)").bind(&id).execute(&self.pool).await.unwrap();
        for (person, role) in members {
            sqlx::query("INSERT INTO space_members (space_id, subject, email, role) VALUES ($1, $2, $2, $3)")
                .bind(&id)
                .bind(&person.subject)
                .bind(role)
                .execute(&self.pool)
                .await
                .unwrap();
        }
        id
    }

    pub async fn call(
        &self,
        method: &str,
        path: &str,
        who: Option<&Person>,
        body: Option<serde_json::Value>,
    ) -> Answer {
        send(&self.router, method, path, who.map(|p| p.token.as_str()), body).await
    }
}
