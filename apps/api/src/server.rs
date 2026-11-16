//! Boot and the serve loop.

use std::net::SocketAddr;
use std::process::ExitCode;
use std::sync::Arc;
use std::time::Duration;

use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;
use tokio_util::sync::CancellationToken;
use tronc::mcp::{App, AppConfig, PostgresPending, keep_pruned};

use crate::auth::Kits;
use crate::env::Config;
use crate::modules::frame::FrameKey;
use crate::state::AppState;

type BootError = Box<dyn std::error::Error + Send + Sync>;

/// How long in-flight requests get after SIGINT or SIGTERM.
pub const DRAIN: Duration = Duration::from_secs(10);

fn fail(error: &dyn std::fmt::Display) -> ExitCode {
    tracing::error!(error = %error, "diapo: startup failed");
    eprintln!("api: {error}");
    ExitCode::FAILURE
}

/// Runs the binary: a `migrate` subcommand, or the server until a signal.
pub async fn run() -> ExitCode {
    let config = match Config::load() {
        Ok(config) => config,
        Err(error) => return fail(&error),
    };
    let _guard = tronc::logger::init(tronc::logger::Config::from_core(&config.core));
    let pool = match PgPoolOptions::new().max_connections(20).connect(&config.core.database_url).await {
        Ok(pool) => pool,
        Err(error) => return fail(&error),
    };
    let argv: Vec<String> = std::env::args().collect();
    if let Some(result) = tronc::migrate::command(&argv, &pool, &crate::MIGRATOR).await {
        return result.map_or_else(|error| fail(&error), |()| ExitCode::SUCCESS);
    }
    match serve(config, pool).await {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => fail(&error),
    }
}

async fn signal() {
    let mut term = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate()).ok();
    tokio::select! {
        _ = tokio::signal::ctrl_c() => {}
        _ = async { if let Some(term) = term.as_mut() { term.recv().await; } else { std::future::pending::<()>().await } } => {}
    }
}

/// The shared state for `pool` under `config`.
pub fn state(config: &Config, pool: PgPool, kits: &Kits) -> AppState {
    AppState {
        pool,
        issuer: Arc::from(config.porte.issuer.as_str()),
        data_dir: Arc::new(config.data_dir.clone()),
        events: Default::default(),
        frame_key: FrameKey::derive(&config.token_key),
        limits: config.limits,
        confirmations: None,
        exchanger: kits.exchanger.clone(),
        bulle: config.bulle.clone(),
    }
}

fn spawn_loops(kits: &Kits, state: &AppState, token: &CancellationToken) {
    tokio::spawn(crate::modules::assets::sweep::run(state.clone(), token.clone()));
    tokio::spawn(crate::modules::decks::purge::run(state.clone(), token.clone()));
    if let Some(syncer) = kits.syncer.clone() {
        let token = token.clone();
        tokio::spawn(async move { syncer.run(token).await });
    }
    if let Some(deletions) = kits.deletions.clone() {
        let token = token.clone();
        tokio::spawn(async move { deletions.run(token).await });
    }
    if state.confirmations.is_some() {
        let pruning = keep_pruned(PostgresPending::new(state.pool.clone()), Duration::ZERO, Duration::ZERO);
        let token = token.clone();
        tokio::spawn(async move {
            token.cancelled().await;
            pruning.abort()
        });
    }
}

fn mcp_app(config: &Config, kits: &Kits, state: &mut AppState) -> Result<Option<App>, BootError> {
    let Some(verifier) = kits.mcp.clone() else { return Ok(None) };
    let mut cfg = AppConfig::new(config.mcp_resource.clone(), config.porte.issuer.clone(), verifier);
    cfg.version = crate::version::VERSION.to_owned();
    cfg.state_key = config.mcp_state_key.as_bytes().to_vec();
    cfg.pending = Some(Arc::new(PostgresPending::new(state.pool.clone())));
    let app = crate::mcp::app(state, cfg)?;
    state.confirmations = app.confirmations.clone();
    Ok(Some(app))
}

async fn serve(config: Config, pool: PgPool) -> Result<(), BootError> {
    tronc::migrate::run(&pool, &crate::MIGRATOR).await?;
    let kits = Kits::new(&config, &pool).await?;
    let token = CancellationToken::new();
    let mut state = state(&config, pool, &kits);
    let mcp = mcp_app(&config, &kits, &mut state)?;
    spawn_loops(&kits, &state, &token);
    let mut http = tronc::http::Config::from_core(&config.core);
    http.security = crate::security::config(http.security, &tronc::spa::dir_from_env(), &config.bulle.url);
    let layers = tronc::http::layers(http)?;
    let app = crate::router::build_router(state, &kits, mcp)?.layer(layers);
    let listener = tokio::net::TcpListener::bind(("0.0.0.0", config.core.port)).await?;
    tracing::info!(port = config.core.port, "diapo: listening");
    let stop = token.clone();
    let server = axum::serve(listener, app.into_make_service_with_connect_info::<SocketAddr>()).with_graceful_shutdown(
        async move {
            signal().await;
            stop.cancel();
        },
    );
    let drained = async {
        token.cancelled().await;
        tokio::time::sleep(DRAIN).await;
    };
    tokio::select! {
        result = server => result?,
        () = drained => tracing::warn!("diapo: drain timed out"),
    }
    Ok(())
}
