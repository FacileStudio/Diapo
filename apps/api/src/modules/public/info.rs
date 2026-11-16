use serde::Serialize;
use tronc::{Error, Result};
use uuid::Uuid;

use super::thumbnail;
use crate::modules::decks::service::fetch;
use crate::modules::share::service::resolve;
use crate::state::AppState;

/// A shared deck, read once for a public answer.
pub struct Shared {
    /// The deck id. It never leaves the server.
    pub id: Uuid,
    /// The deck title.
    pub title: String,
    /// The revision the deck is at.
    pub rev: i64,
    /// The parsed source.
    pub project: scene::Project,
}

/// What `GET /public/decks/{token}` answers: enough to drive the frame, and nothing an editor
/// wrote for themselves. No source, no presenter note, no deck id, no space.
#[derive(Debug, Serialize)]
pub struct PublicDeck {
    pub title: String,
    /// How many scenes the deck has.
    pub scenes: usize,
    /// The last global step of the whole deck, as `window.__scene.steps`.
    pub steps: u32,
    /// The stage size in CSS pixels.
    pub width: u32,
    pub height: u32,
    /// The revision; it changes when the frame should be reloaded.
    pub rev: i64,
    /// The route of the sandboxed frame.
    pub frame: String,
    /// The route of the first slide as a picture, when one exists.
    pub thumbnail: Option<String>,
}

/// The deck the live link `token` names, parsed.
pub async fn shared(state: &AppState, token: &str) -> Result<Shared> {
    let id = resolve(state, token).await?;
    let row = fetch(state, id).await?;
    let project = scene::parse(&row.source).map_err(|errors| {
        Error::internal("a stored deck no longer parses", std::io::Error::other(errors.to_string()))
    })?;
    Ok(Shared { id, title: row.title, rev: row.rev, project })
}

/// The last global step of a whole-deck document: each scene takes `steps + 1` of them.
pub fn steps(project: &scene::Project) -> u32 {
    let stops: u32 = project.scenes().iter().map(|scene| scene.steps + 1).sum();
    stops.saturating_sub(1)
}

/// `GET /public/decks/{token}`.
pub async fn deck(state: &AppState, token: &str) -> Result<PublicDeck> {
    let shared = shared(state, token).await?;
    let size = shared.project.config().size;
    let pictured = thumbnail::exists(state, shared.id, shared.rev).await;
    Ok(PublicDeck {
        title: shared.title,
        scenes: shared.project.scenes().len(),
        steps: steps(&shared.project),
        width: size.width,
        height: size.height,
        rev: shared.rev,
        frame: format!("/api/public/decks/{token}/frame"),
        thumbnail: pictured.then(|| thumbnail::route(token)),
    })
}

/// Whether `deck` has a live share link. A failed query answers no.
pub async fn is_shared(state: &AppState, deck: Uuid) -> bool {
    let live =
        sqlx::query_scalar("SELECT EXISTS (SELECT 1 FROM share_links WHERE deck_id = $1 AND revoked_at IS NULL)")
            .bind(deck)
            .fetch_one(&state.pool)
            .await;
    live.unwrap_or(false)
}

#[cfg(test)]
mod tests {
    #[test]
    fn global_steps_count_one_stop_per_scene_plus_its_reveals() {
        let project = scene::parse("# A\n\n---\n\n- one {step=1}\n- two {step=2}\n\n---\n\n# C\n").unwrap();
        assert_eq!(super::steps(&project), 4);
    }
}
