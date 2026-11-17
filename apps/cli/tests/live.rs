//! `pull` and `push` through the binary against the real API: the router of `apps/api` on a
//! local port, over the Postgres at `TEST_DATABASE_URL`. Skips with a banner without one.

#[path = "support/bin.rs"]
mod bin;

use std::fs;
use std::net::SocketAddr;
use std::path::Path;
use std::process::Output;

use api::auth::Kits;
use api::env::Config;
use sqlx::PgPool;
use tokio::runtime::Runtime;
use tronc::env::Vars;

const ISSUER: &str = "https://sso.test";

/// A running API and a token for one account on it.
struct Live {
    url: String,
    token: String,
    _runtime: Runtime,
}

async fn person(pool: &PgPool, kits: &Kits) -> String {
    let subject = format!("cli-{}", std::process::id());
    let user_id: i64 = sqlx::query_scalar("INSERT INTO porte_users (email, name) VALUES ($1, 'Ada') RETURNING id")
        .bind(format!("{subject}@example.test"))
        .fetch_one(pool)
        .await
        .unwrap();
    sqlx::query("INSERT INTO porte_identities (user_id, provider, subject) VALUES ($1, $2, $3)")
        .bind(user_id)
        .bind(ISSUER)
        .bind(&subject)
        .execute(pool)
        .await
        .unwrap();
    kits.sessions.issue(user_id, "test").await.unwrap().0
}

async fn boot(data_dir: &Path) -> Option<(String, String)> {
    let Some(pool) = tronc::testdb::pool("cli", &api::MIGRATOR).await else {
        eprintln!("{}", tronc::testdb::skip_reason("mise run db-up"));
        return None;
    };
    let pairs = vec![("DATABASE_URL", "postgres://unused/x"), ("DATA_DIR", data_dir.to_str().unwrap())];
    let config = Config::from_vars(&Vars::from_pairs(pairs)).unwrap();
    let hub = api::modules::rooms::Hub::default();
    let kits = Kits::new(&config, &pool, &hub).await.unwrap();
    let state = api::state::AppState { issuer: ISSUER.into(), ..api::server::state(&config, pool.clone(), &kits, hub) };
    let router = api::router::build_router(state, &kits, None).unwrap();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let url = format!("http://{}", listener.local_addr().unwrap());
    tokio::spawn(
        async move { axum::serve(listener, router.into_make_service_with_connect_info::<SocketAddr>()).await },
    );
    Some((url, person(&pool, &kits).await))
}

fn live(data_dir: &Path) -> Option<Live> {
    let runtime = Runtime::new().unwrap();
    let (url, token) = runtime.block_on(boot(data_dir))?;
    Some(Live { url, token, _runtime: runtime })
}

fn diapo(live: &Live, work: &Path, args: &[&str]) -> Output {
    let mut command = bin::command(work, work, args);
    command.env("DIAPO_SERVER_URL", &live.url).env("DIAPO_TOKEN", &live.token).output().unwrap()
}

fn projected(source: &str) -> String {
    api::modules::decks::source::checked(source, usize::MAX).map(|prepared| prepared.source).unwrap_or_default()
}

fn text(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).into_owned()
}

#[test]
fn a_deck_goes_up_comes_down_and_a_stale_push_is_refused_by_the_real_api() {
    let work = tempfile::tempdir().unwrap();
    let Some(live) = live(&work.path().join("data")) else { return };
    let run = |args: &[&str]| diapo(&live, work.path(), args);
    assert!(run(&["new", "talk"]).status.success());
    let created = run(&["push", "talk", "--create", "--json"]);
    assert!(created.status.success(), "{}", text(&created.stderr));
    let answer: serde_json::Value = serde_json::from_slice(&created.stdout).unwrap();
    let id = answer["deck"].as_str().unwrap().to_owned();
    assert_eq!((answer["rev"].as_i64(), answer["assets"]["uploaded"][0].as_str()), (Some(1), Some("assets/chart.svg")));
    assert!(
        text(&run(&["list", "-q"]).stdout).contains(&id) && text(&run(&["whoami"]).stdout).contains("@example.test")
    );
    let pulled = run(&["pull", &format!("{}/decks/{id}", live.url), "copy"]);
    assert!(pulled.status.success(), "{}", text(&pulled.stderr));
    let read = |path: &str| fs::read(work.path().join(path)).unwrap();
    assert_eq!(read("copy/diapo.md"), projected(&text(&read("talk/diapo.md"))).into_bytes());
    assert_eq!(read("copy/assets/chart.svg"), read("talk/assets/chart.svg"));
    fs::write(work.path().join("copy/diapo.md"), "# From the copy\n").unwrap();
    fs::remove_file(work.path().join("copy/assets/chart.svg")).unwrap();
    assert!(text(&run(&["push", "copy"]).stdout).contains("(revision 2)"));
    fs::write(work.path().join("talk/diapo.md"), "# From the first folder\n").unwrap();
    let stale = run(&["push", "talk"]);
    assert_eq!(stale.status.code(), Some(1));
    assert!(text(&stale.stderr).contains("since revision 1") && text(&stale.stderr).contains("`diapo pull`"));
    assert!(run(&["pull", "talk"]).status.code() == Some(1) && run(&["pull", "talk", "--force"]).status.success());
    assert_eq!(fs::read_to_string(work.path().join("talk/diapo.md")).unwrap(), projected("# From the copy\n"));
    assert!(!work.path().join("talk/assets/chart.svg").exists() && run(&["push", "talk", "--force"]).status.success());
}
