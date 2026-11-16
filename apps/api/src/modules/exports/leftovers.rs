//! The files that outlive their rows unless something removes them: export files under
//! `DATA_DIR/exports` and thumbnails under `DATA_DIR/thumbnails/<deck id>`. A deleted deck or a
//! deleted person leaves none behind.

use std::collections::HashSet;
use std::path::Path;

use sqlx::{PgConnection, PgPool};
use uuid::Uuid;

use super::path;
use crate::modules::public::thumbnail;

/// The exports of `decks`, and those `users` asked for, as they are before a delete cascades.
pub async fn exports_of(conn: &mut PgConnection, decks: &[Uuid], users: &[i64]) -> Result<Vec<Uuid>, sqlx::Error> {
    sqlx::query_scalar("SELECT id FROM exports WHERE deck_id = ANY($1) OR created_by = ANY($2)")
        .bind(decks)
        .bind(users)
        .fetch_all(conn)
        .await
}

/// Removes at once the thumbnails of `decks` and the files of `exports`.
pub async fn forget(data_dir: &Path, decks: &[Uuid], exports: &[Uuid]) {
    for deck in decks {
        tokio::fs::remove_dir_all(thumbnail::folder(data_dir, *deck)).await.ok();
    }
    for export in exports {
        tokio::fs::remove_file(path(data_dir, *export)).await.ok();
    }
}

async fn names(dir: &Path) -> Vec<(String, std::path::PathBuf)> {
    let mut found = Vec::new();
    let Ok(mut entries) = tokio::fs::read_dir(dir).await else { return found };
    while let Ok(Some(entry)) = entries.next_entry().await {
        found.push((entry.file_name().to_string_lossy().into_owned(), entry.path()));
    }
    found
}

async fn ids(pool: &PgPool, query: &'static str) -> Result<HashSet<String>, sqlx::Error> {
    let rows: Vec<String> = sqlx::query_scalar(query).fetch_all(pool).await?;
    Ok(rows.into_iter().collect())
}

fn kept(name: &str, known: &HashSet<String>) -> bool {
    let stem = name.split('.').next().unwrap_or(name);
    Uuid::parse_str(stem).is_ok_and(|id| known.contains(&id.to_string()))
}

/// Removes every export file and every thumbnail folder whose job or deck no row names any more.
/// Answers how many went.
pub async fn sweep(pool: &PgPool, data_dir: &Path) -> Result<usize, sqlx::Error> {
    let exports = ids(pool, "SELECT id::text FROM exports").await?;
    let decks = ids(pool, "SELECT id::text FROM decks").await?;
    let mut removed = 0;
    for (name, file) in names(&data_dir.join("exports")).await.into_iter().filter(|(name, _)| !kept(name, &exports)) {
        removed += usize::from(tokio::fs::remove_file(file).await.is_ok());
        tracing::info!(file = %name, "exports: removed a file no job names");
    }
    let folders = names(&data_dir.join("thumbnails")).await;
    for (name, folder) in folders.into_iter().filter(|(name, _)| !kept(name, &decks)) {
        removed += usize::from(tokio::fs::remove_dir_all(folder).await.is_ok());
        tracing::info!(deck = %name, "thumbnails: removed the thumbnails of a deck that is gone");
    }
    Ok(removed)
}

#[cfg(test)]
mod tests {
    #[test]
    fn a_file_is_kept_only_while_its_id_is_known() {
        let id = uuid::Uuid::new_v4();
        let known = std::collections::HashSet::from([id.to_string()]);
        assert!(super::kept(&id.to_string(), &known) && super::kept(&format!("{id}.part"), &known));
        assert!(!super::kept(&uuid::Uuid::new_v4().to_string(), &known) && !super::kept("junk", &known));
    }
}
