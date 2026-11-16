//! The bundle of a deck: its compiled document and its assets, by the relative paths the source
//! writes. The worker's browser loads it from disk and reaches nothing else.

use std::path::Path;

use tronc::{Error, Result};
use uuid::Uuid;

use crate::extract::db;
use crate::modules::assets::files;
use crate::state::AppState;

fn failed(error: std::io::Error) -> Error {
    Error::internal("a capture bundle could not be written", error)
}

/// The assets of `deck`, as `(name, sha256)`.
pub async fn assets(state: &AppState, deck: Uuid) -> Result<Vec<(String, String)>> {
    sqlx::query_as("SELECT name, sha256 FROM assets WHERE deck_id = $1")
        .bind(deck)
        .fetch_all(&state.pool)
        .await
        .map_err(db)
}

/// Writes `index.html`, the whole deck compiled with its asset paths as written, and each asset
/// of `deck` at its name under `folder`. Asset names are validated relative paths, so every file
/// lands inside `folder`.
pub async fn write(state: &AppState, deck: Uuid, project: &scene::Project, folder: &Path) -> Result<()> {
    tokio::fs::create_dir_all(folder).await.map_err(failed)?;
    let html = scene::compile(project, &scene::Options::new());
    tokio::fs::write(folder.join("index.html"), html).await.map_err(failed)?;
    for (name, sha256) in assets(state, deck).await? {
        let target = folder.join(&name);
        if let Some(parent) = target.parent() {
            tokio::fs::create_dir_all(parent).await.map_err(failed)?;
        }
        tokio::fs::copy(files::path(&state.data_dir, &sha256), &target).await.map_err(failed)?;
    }
    Ok(())
}
