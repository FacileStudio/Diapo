use axum::Router;
use axum::routing::get;
use tronc::health::{self, Check};

use crate::auth::{self, Kits};
use crate::state::AppState;

async fn api_not_found() -> tronc::Error {
    tronc::Error::not_found("route not found")
}

fn api(kits: &Kits) -> Router<AppState> {
    Router::new().merge(auth::routes(kits)).fallback(api_not_found)
}

/// The whole HTTP tree, before tronc's layers: probes, `/version`, `/docs`, `/api`, then the SPA.
pub fn build_router(state: AppState, kits: &Kits) -> Result<Router, String> {
    let pool = state.pool.clone();
    let root = health::routes(vec![Check::db(pool)])
        .route("/version", get(crate::version::handler))
        .nest("/api", api(kits))
        .with_state(state)
        .merge(tronc::apiref::routes(&crate::apiref::config(), crate::apiref::document())?);
    let dir = tronc::spa::dir_from_env();
    Ok(match tronc::spa::available(&dir) {
        true => root.fallback_service(tronc::spa::handler(tronc::spa::Config { dir, ..Default::default() })),
        false => root.fallback(tronc::http::not_found),
    })
}
