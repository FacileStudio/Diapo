use schemars::JsonSchema;
use serde::Deserialize;
use tronc::mcp::{Action, Answer, Call, Confirmation, Confirmations, Kind, ToolError};
use uuid::Uuid;

use super::{Context, user};
use crate::extract::id;
use crate::modules::decks::error::DeckError;
use crate::modules::decks::trash;
use crate::state::AppState;

/// The confirmation kind of `diapo_delete_deck`.
pub const KIND: &str = "diapo_delete_deck";

const DONE: &str = "The user confirmed and the deck is in the trash, where it stays 30 days.";

/// `diapo_delete_deck`.
#[derive(Deserialize, JsonSchema)]
pub struct DeleteIn {
    /// The deck id.
    pub deck_id: String,
}

fn target(action: &Action) -> tronc::Result<(i64, Uuid)> {
    let user = action.user_id.parse().map_err(|e| tronc::Error::internal("bad action user", e))?;
    Ok((user, id(&action.target)?))
}

async fn fingerprint(state: AppState, action: Action) -> tronc::Result<String> {
    let (user, deck) = target(&action)?;
    let deck = trash::managed(&state, user, deck).await?;
    Ok(format!("{}:{}", deck.rev, deck.trashed_at.is_some()))
}

async fn run(state: AppState, action: Action) -> tronc::Result<()> {
    let (user, deck) = target(&action)?;
    trash::trash(&state, user, deck).await.map(|_| ())
}

/// The confirmation kind that trashes a deck once the person agrees. An edit made in between
/// closes the action.
pub fn kind(state: &AppState) -> Kind {
    let (a, b) = (state.clone(), state.clone());
    Kind::new(KIND, move |action| fingerprint(a.clone(), action), move |action| run(b.clone(), action))
}

/// `diapo_delete_deck`: asks the person, then moves the deck to the trash if they agree.
pub async fn delete(
    ctx: Context,
    confirmations: Confirmations,
    call: Call,
    input: DeleteIn,
) -> Result<Answer<Confirmation>, ToolError> {
    if call.retrying() {
        return Ok(confirmations.answered(&call, KIND, DONE).await?.into());
    }
    let deck_id = id(&input.deck_id).map_err(DeckError::Api)?;
    let deck = trash::managed(&ctx.state, user(&call)?, deck_id).await.map_err(DeckError::Api)?;
    if deck.trashed_at.is_some() {
        return Err(ToolError("the deck is already in the trash: nothing to do".to_owned()));
    }
    let question = format!("Move the deck \"{}\" to the trash?", deck.title);
    confirmations.ask(&call, KIND, &deck_id.to_string(), &question).await
}
