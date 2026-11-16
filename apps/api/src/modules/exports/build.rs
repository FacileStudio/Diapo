use std::collections::HashMap;

use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use tronc::{Error, Result};
use uuid::Uuid;

use super::Format;
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

/// One offline HTML file: the compiled deck with every asset it has inlined as a `data:` URI.
pub fn html(project: &scene::Project, bundle: &Bundle) -> Vec<u8> {
    let inline = |path: &str| {
        let (mime, bytes) = bundle.get(name::referenced(path).0)?;
        Some(format!("data:{mime};base64,{}", STANDARD.encode(bytes)))
    };
    scene::compile(project, &scene::Options::new().assets(&inline)).into_bytes()
}

/// Pictures of scenes a file cannot hold as shapes, by scene index.
pub type Pictures = HashMap<usize, Vec<u8>>;

/// An editable PowerPoint file, with the deck's pictures embedded and a picture for each
/// animated scene `pictures` has one for.
pub fn pptx(project: &scene::Project, bundle: &Bundle, pictures: &Pictures) -> Result<Vec<u8>> {
    let read = |path: &str| bundle.get(name::referenced(path).0).map(|(_, bytes)| bytes.clone());
    let still = |index: usize| pictures.get(&index).cloned();
    scene::pptx::write_with(project, &read, &still).map_err(|error| Error::internal("failed to write the pptx", error))
}

/// The file of `format` for `source`, built off the async threads. Only HTML and PPTX are built
/// here: a PDF comes from the capture service and an MP4 from Clap.
pub async fn file(format: Format, source: String, bundle: Bundle, pictures: Pictures) -> Result<Vec<u8>> {
    let work = move || {
        let project = scene::parse(&source).map_err(|errors| Error::failed(errors.to_string()))?;
        match format {
            Format::Html => Ok(html(&project, &bundle)),
            Format::Pptx => pptx(&project, &bundle, &pictures),
            Format::Pdf | Format::Mp4 => Err(Error::unavailable("this export is not available yet")),
        }
    };
    tokio::task::spawn_blocking(work).await.map_err(|error| Error::internal("the export did not finish", error))?
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_html_export_inlines_the_assets_it_has_and_loads_nothing() {
        let project = scene::parse("# One\n\n![](assets/dot.png)\n\n![](assets/missing.png)\n").unwrap();
        let bundle = Bundle::from([("assets/dot.png".to_owned(), ("image/png".to_owned(), b"png".to_vec()))]);
        let html = String::from_utf8(html(&project, &bundle)).unwrap();
        assert!(html.contains("src=\"data:image/png;base64,cG5n\""), "{html}");
        assert!(html.contains("src=\"assets/missing.png\""));
        assert!(!html.contains("/api/"));
    }
}
