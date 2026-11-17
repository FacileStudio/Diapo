use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use tronc::mcp::{Answer, Call, ToolError};

use super::{Context, user};
use crate::extract::id;
use crate::modules::decks::edit::{self, Replacement};
use crate::modules::decks::error::DeckError;
use crate::modules::decks::types::{Author, CreateRequest, Deck, SourceRequest};
use crate::modules::decks::{service, write as decks};

/// What a write answers.
#[derive(Serialize, JsonSchema)]
pub struct Written {
    pub id: String,
    pub title: String,
    /// The deck's new revision: name it in the next write.
    pub rev: i64,
    /// How many slides (scenes) the deck has now.
    pub scenes: usize,
    /// The page a person opens to see the deck.
    pub url: String,
}

fn written(ctx: &Context, deck: Deck) -> Answer<Written> {
    let scenes = scene::planche::project(&deck.source).map_or(0, |project| project.scenes().len());
    Written { id: deck.id.to_string(), title: deck.title, rev: deck.rev, scenes, url: ctx.deck_url(deck.id) }.into()
}

/// `diapo_create_deck`.
#[derive(Deserialize, JsonSchema)]
pub struct CreateIn {
    /// The deck title, at most 200 characters.
    pub title: String,
    /// The whole deck as a planche projection (scene markdown is converted); a one-slide starter
    /// when absent.
    pub source: Option<String>,
    /// The space; a personal deck when absent.
    pub space_id: Option<String>,
}

/// `diapo_create_deck`: a new deck at rev 1.
pub async fn create(ctx: Context, call: Call, input: CreateIn) -> Result<Answer<Written>, ToolError> {
    let request = CreateRequest { title: input.title, space_id: input.space_id, source: input.source };
    let deck = service::create(&ctx.state, &Author::agent(user(&call)?), request).await?;
    Ok(written(&ctx, deck))
}

/// `diapo_write_deck`.
#[derive(Deserialize, JsonSchema)]
pub struct WriteIn {
    /// The deck id.
    pub deck_id: String,
    /// The whole new source as a planche projection (scene markdown is converted).
    pub source: String,
    /// The rev diapo_get_deck answered.
    pub rev: i64,
}

/// `diapo_write_deck`: replaces the whole source if the deck is still at `rev`.
pub async fn write(ctx: Context, call: Call, input: WriteIn) -> Result<Answer<Written>, ToolError> {
    let deck_id = id(&input.deck_id).map_err(DeckError::Api)?;
    let request = SourceRequest { source: input.source, rev: input.rev };
    let deck = decks::save(&ctx.state, &Author::agent(user(&call)?), deck_id, request).await?;
    Ok(written(&ctx, deck))
}

/// `diapo_edit_deck`.
#[derive(Deserialize, JsonSchema)]
pub struct EditIn {
    /// The deck id.
    pub deck_id: String,
    /// The exact text to replace. It must be in the source exactly once.
    pub old_string: String,
    /// The text that takes its place; empty deletes `old_string`.
    pub new_string: String,
    /// The rev diapo_get_deck answered.
    pub rev: i64,
}

/// `diapo_edit_deck`: one exact string replacement, if the deck is still at `rev`.
pub async fn edit(ctx: Context, call: Call, input: EditIn) -> Result<Answer<Written>, ToolError> {
    let deck_id = id(&input.deck_id).map_err(DeckError::Api)?;
    let change = Replacement { old: input.old_string, new: input.new_string, rev: input.rev };
    let deck = edit::edit(&ctx.state, &Author::agent(user(&call)?), deck_id, change).await?;
    Ok(written(&ctx, deck))
}
