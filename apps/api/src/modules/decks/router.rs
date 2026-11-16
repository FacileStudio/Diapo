use axum::Router;
use axum::routing::{get, put};

use super::handlers::{create, list, read, remove, save, update};
use crate::state::AppState;

/// The deck routes, mounted under `/api` behind `require_auth`.
pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/decks", get(list).post(create))
        .route("/decks/{id}", get(read).patch(update).delete(remove))
        .route("/decks/{id}/source", put(save))
}
