use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use time::format_description::well_known::Rfc3339;
use tronc::mcp::{Answer, Call, RESULT_BUDGET, ToolError, clip};

use super::page::{Listed, Window, window};
use super::{Context, user};
use crate::extract::id;
use crate::modules::assets::service as assets;
use crate::modules::decks::error::DeckError;
use crate::modules::decks::service;
use crate::modules::decks::types::ListQuery;

/// `diapo_list_decks`.
#[derive(Deserialize, JsonSchema)]
pub struct ListIn {
    /// One space; every deck you reach, personal ones included, when absent.
    pub space_id: Option<String>,
    /// True lists the trash instead of the live decks.
    pub trashed: Option<bool>,
    /// The `next_cursor` of the previous call; the first decks when absent.
    pub cursor: Option<String>,
    /// How many decks, 1 to 200; 50 when absent.
    pub limit: Option<usize>,
}

/// One deck of the list.
#[derive(Serialize, JsonSchema)]
pub struct Item {
    pub id: String,
    pub title: String,
    /// The space the deck is in; absent for a personal deck.
    pub space_id: Option<String>,
    pub rev: i64,
    /// RFC 3339.
    pub updated_at: String,
    /// The page a person opens to see the deck.
    pub url: String,
}

/// `diapo_list_decks`: the decks the caller reaches, newest first, a window at a time.
pub async fn list(ctx: Context, call: Call, input: ListIn) -> Result<Answer<Listed<Item>>, ToolError> {
    let query = ListQuery { space_id: input.space_id, trashed: input.trashed.unwrap_or(false) };
    let entries = service::list(&ctx.state, user(&call)?, &query).await.map_err(DeckError::Api)?;
    let decks = entries
        .into_iter()
        .map(|entry| Item {
            id: entry.id.to_string(),
            title: entry.title,
            space_id: entry.space_id,
            rev: entry.rev,
            updated_at: entry.updated_at.format(&Rfc3339).unwrap_or_default(),
            url: ctx.deck_url(entry.id),
        })
        .collect();
    Ok(window(decks, Window { cursor: input.cursor, limit: input.limit })?.into())
}

/// `diapo_get_deck`.
#[derive(Deserialize, JsonSchema)]
pub struct GetIn {
    /// The deck id.
    pub deck_id: String,
}

/// What `diapo_get_deck` answers.
#[derive(Serialize, JsonSchema)]
pub struct GetOut {
    pub id: String,
    pub title: String,
    /// The space the deck is in; absent for a personal deck.
    pub space_id: Option<String>,
    /// The revision to name in diapo_write_deck or diapo_edit_deck.
    pub rev: i64,
    /// The whole deck, as its planche projection.
    pub source: String,
    /// True when the source was cut to fit: never write it back whole, use diapo_edit_deck.
    pub truncated: bool,
    /// How many slides (scenes) the deck has.
    pub scenes: usize,
    /// The asset names the source may load, such as `assets/chart.png`.
    pub assets: Vec<String>,
    /// True when the deck is in the trash.
    pub trashed: bool,
    /// The page a person opens to see the deck.
    pub url: String,
}

/// `diapo_get_deck`: the source and the `rev` every write must name.
pub async fn get(ctx: Context, call: Call, input: GetIn) -> Result<Answer<GetOut>, ToolError> {
    let deck_id = id(&input.deck_id).map_err(DeckError::Api)?;
    let deck = service::read(&ctx.state, user(&call)?, deck_id).await.map_err(DeckError::Api)?;
    let mut names: Vec<String> = assets::names(&ctx.state, deck_id).await?.into_iter().collect();
    names.sort();
    let (source, truncated) = clip(&deck.source, RESULT_BUDGET);
    let out = GetOut {
        id: deck.id.to_string(),
        title: deck.title,
        space_id: deck.space_id,
        rev: deck.rev,
        source: source.to_owned(),
        truncated,
        scenes: scene::planche::project(&deck.source).map_or(0, |project| project.scenes().len()),
        assets: names,
        trashed: deck.trashed_at.is_some(),
        url: ctx.deck_url(deck.id),
    };
    Ok(out.into())
}

/// `diapo_format_guide`.
#[derive(Deserialize, JsonSchema)]
pub struct GuideIn {}

/// What `diapo_format_guide` answers.
#[derive(Serialize, JsonSchema)]
pub struct GuideOut {
    /// The format reference, in markdown.
    pub guide: String,
}

/// `diapo_format_guide`: the planche guide, `scene::planche::GUIDE`, then the scene format
/// reference, `scene::FORMAT_GUIDE`. Both, because a write still accepts scene markdown and a
/// locked `html` shape holds compiled scene HTML.
pub async fn guide(_call: Call, _input: GuideIn) -> Result<Answer<GuideOut>, ToolError> {
    let guide = format!("{}\n\n# Legacy scene markdown\n\n{}", scene::planche::GUIDE, scene::FORMAT_GUIDE);
    Ok(GuideOut { guide }.into())
}
