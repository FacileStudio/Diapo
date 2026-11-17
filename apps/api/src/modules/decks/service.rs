use tronc::{Error, Result};
use uuid::Uuid;

use super::access::{self, Access, member_spaces};
use super::error::DeckError;
use super::source;
use super::types::{Author, CreateRequest, Deck, DeckEntry, DeckRow, ListQuery, columns};
use super::write;
use crate::extract::db;
use crate::state::AppState;

/// The longest title, in characters.
pub const MAX_TITLE: usize = 200;

/// The title of a deck that names none.
pub const UNTITLED: &str = "Untitled";

/// Reads one row, whoever asks: callers check access first.
pub async fn fetch(state: &AppState, id: Uuid) -> Result<DeckRow> {
    sqlx::query_as::<_, DeckRow>(concat!("SELECT ", columns!(), " FROM decks WHERE id = $1"))
        .bind(id)
        .fetch_optional(&state.pool)
        .await
        .map_err(db)?
        .ok_or_else(|| Error::not_found("deck not found"))
}

/// The row as the API answers it for a caller with `access`.
pub fn answer(row: DeckRow, access: Access) -> Deck {
    Deck {
        id: row.id,
        space_id: row.space_id,
        title: row.title,
        source: row.source,
        rev: row.rev,
        created_at: row.created_at,
        updated_at: row.updated_at,
        trashed_at: row.trashed_at,
        access: access.name(),
    }
}

/// A trimmed title of at most [`MAX_TITLE`] characters.
pub fn title(raw: &str) -> Result<String> {
    let title = raw.trim();
    match title.chars().count() > MAX_TITLE {
        true => Err(Error::invalid(format!("title is longer than {MAX_TITLE} characters"))),
        false => Ok(title.to_owned()),
    }
}

/// `GET /decks/{id}`: the deck, for a caller who may at least read it.
pub async fn read(state: &AppState, user_id: i64, id: Uuid) -> Result<Deck> {
    let have = access::check(state, user_id, id, Access::Viewer).await?;
    Ok(answer(fetch(state, id).await?, have))
}

fn named(asked: &str, project: &scene::Project) -> Result<String> {
    let given = title(asked)?;
    let written = title(&project.config().title).unwrap_or_default();
    Ok([given, written].into_iter().find(|name| !name.is_empty()).unwrap_or_else(|| UNTITLED.to_owned()))
}

/// `POST /decks`: a new deck at `rev` 1, in a space the caller may edit or in their personal
/// decks. The source, in either format, is stored as its planche projection; the starter deck
/// when none is given. Its title is the one asked for, else the source's (a legacy `title`, a
/// projection's first frame name), else [`UNTITLED`].
pub async fn create(state: &AppState, author: &Author, request: CreateRequest) -> std::result::Result<Deck, DeckError> {
    let have = access::scope(state, author.user_id, request.space_id.as_deref()).await?;
    access::require(have, Access::Editor)?;
    let text = request.source.as_deref().unwrap_or(scene::planche::STARTER);
    let prepared = source::checked(text, state.limits.source)?;
    let legacy = (!scene::planche::is_planche(text)).then(|| scene::parse(text).ok()).flatten();
    let title = named(&request.title, legacy.as_ref().unwrap_or(&prepared.project))?;
    let mut tx = state.pool.begin().await.map_err(db)?;
    let row = sqlx::query_as::<_, DeckRow>(concat!(
        "INSERT INTO decks (space_id, owner_id, title, source, created_by, updated_by) \
         VALUES ($1, $2, $3, $4, $2, $2) RETURNING ",
        columns!()
    ))
    .bind(&request.space_id)
    .bind(author.user_id)
    .bind(title)
    .bind(&prepared.source)
    .fetch_one(&mut *tx)
    .await
    .map_err(db)?;
    write::revision(&mut tx, &row, author).await.map_err(db)?;
    tx.commit().await.map_err(db)?;
    Ok(answer(row, have))
}

/// `GET /decks`: every deck the caller reaches, newest first: their personal decks and those of
/// their spaces, or one space with `space_id` (`404` when the caller is not a member).
pub async fn list(state: &AppState, user_id: i64, query: &ListQuery) -> Result<Vec<DeckEntry>> {
    access::scope(state, user_id, query.space_id.as_deref()).await?;
    let rows = sqlx::query_as::<_, DeckEntry>(concat!(
        "SELECT d.id, d.space_id, d.title, d.rev, d.updated_at, d.trashed_at, s.role FROM decks d LEFT JOIN (",
        member_spaces!(),
        ") s ON s.space_id = d.space_id WHERE ((d.space_id IS NULL AND d.owner_id = $1) OR s.role IS NOT NULL) \
         AND ($3::text IS NULL OR d.space_id = $3) AND (d.trashed_at IS NOT NULL) = $4 \
         ORDER BY d.updated_at DESC, d.id"
    ))
    .bind(user_id)
    .bind(&*state.issuer)
    .bind(&query.space_id)
    .bind(query.trashed)
    .fetch_all(&state.pool)
    .await
    .map_err(db)?;
    let entry = |entry: DeckEntry| DeckEntry {
        access: entry.role.as_deref().map_or(Access::Manager, access::from_role).name(),
        ..entry
    };
    Ok(rows.into_iter().map(entry).collect())
}

#[cfg(test)]
mod tests {
    #[test]
    fn titles_are_trimmed_bounded_and_fall_back_to_the_source_then_untitled() {
        assert_eq!(super::title("  Plan  ").unwrap(), "Plan");
        assert!(super::title(&"x".repeat(201)).is_err());
        let titled = scene::parse("---\ntitle: Q3 review\n---\n\n# Hello\n").unwrap();
        let bare = scene::parse("# Hello\n").unwrap();
        assert_eq!(super::named(" Mine ", &titled).unwrap(), "Mine");
        assert_eq!(super::named("", &titled).unwrap(), "Q3 review");
        assert_eq!(super::named("", &bare).unwrap(), super::UNTITLED);
        let starter = scene::planche::project(scene::planche::STARTER).unwrap();
        assert_eq!(super::named("", &starter).unwrap(), "New deck");
    }
}
