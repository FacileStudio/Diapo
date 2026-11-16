use std::collections::HashMap;

use tronc::{Error, Result};
use uuid::Uuid;

use super::Format;
use super::inline::{MAX_BYTES, html, too_large};
use crate::extract::db;
use crate::modules::assets::{files, name};
use crate::state::AppState;

/// The assets of one deck, read into memory: name to media type and bytes.
pub type Bundle = HashMap<String, (String, Vec<u8>)>;

/// Reads every asset of `deck` from the data volume.
pub async fn bundle(state: &AppState, deck: Uuid) -> Result<Bundle> {
    let rows: Vec<(String, String, String)> =
        sqlx::query_as("SELECT name, mime, sha256 FROM assets WHERE deck_id = $1")
            .bind(deck)
            .fetch_all(&state.pool)
            .await
            .map_err(db)?;
    let mut bundle = Bundle::new();
    for (name, mime, sha256) in rows {
        let bytes = tokio::fs::read(files::path(&state.data_dir, &sha256))
            .await
            .map_err(|error| Error::internal("an asset file is missing", error))?;
        bundle.insert(name, (mime, bytes));
    }
    Ok(bundle)
}

/// Pictures of scenes a file cannot hold as shapes, by scene index.
pub type Pictures = HashMap<usize, Vec<u8>>;

/// An editable PowerPoint file, with the deck's pictures embedded and a picture for each
/// animated scene `pictures` has one for. The writer reads each picture once per path it is
/// written as and embeds it once.
pub fn pptx(project: &scene::Project, bundle: &Bundle, pictures: &Pictures) -> Result<Vec<u8>> {
    let read = |path: &str| bundle.get(name::referenced(path).0).map(|(_, bytes)| bytes.clone());
    let still = |index: usize| pictures.get(&index).cloned();
    let file = scene::pptx::write_with(project, &read, &still)
        .map_err(|error| Error::internal("failed to write the pptx", error))?;
    match file.len() > MAX_BYTES {
        true => Err(too_large(MAX_BYTES)),
        false => Ok(file),
    }
}

/// The file of `format` for `source`, built off the async threads. Only HTML and PPTX are built
/// here: a PDF comes from the capture service and an MP4 from Clap.
pub async fn file(format: Format, source: String, bundle: Bundle, pictures: Pictures) -> Result<Vec<u8>> {
    let work = move || {
        let project = scene::parse(&source).map_err(|errors| Error::failed(errors.to_string()))?;
        match format {
            Format::Html => html(&source, &project, bundle, MAX_BYTES),
            Format::Pptx => pptx(&project, &bundle, &pictures),
            Format::Pdf | Format::Mp4 => Err(Error::unavailable("this export is not available yet")),
        }
    };
    tokio::task::spawn_blocking(work).await.map_err(|error| Error::internal("the export did not finish", error))?
}
