use axum::Router;
use axum::routing::get;
use tronc::health::{self, Check};

use crate::auth::{self, Kits};
use crate::env::Config;
use crate::modules::{assets, assistant, decks, events, exports, frame, pending, public, rooms, share, spaces};
use crate::state::AppState;

async fn api_not_found() -> tronc::Error {
    tronc::Error::not_found("route not found")
}

fn protected(kits: &Kits, state: &AppState) -> Router<AppState> {
    Router::new()
        .merge(spaces::routes())
        .merge(decks::routes())
        .merge(events::routes())
        .merge(share::routes())
        .merge(rooms::routes())
        .merge(exports::routes())
        .merge(pending::routes(state.confirmations.clone()))
        .merge(assistant::routes())
        .route_layer(kits.sessions.require_auth())
}

fn api(kits: &Kits, state: &AppState) -> Result<Router<AppState>, String> {
    let mut api = Router::new()
        .merge(auth::routes(kits))
        .merge(protected(kits, state))
        .merge(assets::routes(&kits.sessions, state.limits.asset))
        .merge(frame::routes(&kits.sessions))
        .merge(exports::download_routes(&kits.sessions))
        .merge(public::api_routes(&state.embed_ancestors)?);
    if let Some(syncer) = &kits.syncer {
        api = api.route("/registre/sync", syncer.nudge_route());
    }
    if let Some(deletions) = &kits.deletions {
        api = api.route("/registre/deletions", deletions.route());
    }
    Ok(api.fallback(api_not_found))
}

/// The whole HTTP tree, before tronc's layers: probes, `/version`, `/docs`, `/mcp` when
/// `mcp` is given, `/api`, the public share pages, then the SPA.
pub fn build_router(state: AppState, kits: &Kits, mcp: Option<tronc::mcp::App>) -> Result<Router, String> {
    let pool = state.pool.clone();
    let mut root = health::routes(vec![Check::db(pool)])
        .route("/version", get(crate::version::handler))
        .nest("/api", api(kits, &state)?)
        .merge(public::root_routes(&state.embed_ancestors)?)
        .with_state(state.clone())
        .merge(tronc::apiref::routes(&crate::apiref::config(), crate::apiref::document())?);
    if let Some(app) = mcp {
        root = app.mount(root)?;
    }
    let dir = tronc::spa::dir_from_env();
    Ok(match tronc::spa::available(&dir) {
        true => root.fallback_service(tronc::spa::handler(tronc::spa::Config { dir, ..Default::default() })),
        false => root.fallback(tronc::http::not_found),
    })
}

/// [`build_router`] under tronc's HTTP layers, with the app's security policy.
pub fn layered(
    config: &Config,
    state: AppState,
    kits: &Kits,
    mcp: Option<tronc::mcp::App>,
) -> Result<Router, Box<dyn std::error::Error + Send + Sync>> {
    let mut http = tronc::http::Config::from_core(&config.core);
    http.security = crate::security::config(http.security, &tronc::spa::dir_from_env(), &config.bulle.url);
    let layers = tronc::http::layers(http)?;
    Ok(build_router(state, kits, mcp)?.layer(layers))
}
