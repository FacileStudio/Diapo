//! `/mcp`: eight curated tools over the decks, `diapo_delete_deck` behind a confirmation.

mod delete;
mod export;
mod page;
mod read;
mod tools;
mod write;

use std::collections::HashMap;
use std::sync::Arc;

use tronc::mcp::{App, AppConfig, AppSpec, Call, ToolError};
use uuid::Uuid;

use crate::state::AppState;

pub use delete::KIND;

const INSTRUCTIONS: &str = "Diapo holds slide decks. A deck is one text file: markdown scenes split by `---` \
lines, with YAML frontmatter. Call diapo_format_guide before you write your first deck. Every write names the \
rev it read: read the deck with diapo_get_deck, then write with that rev; a stale rev is refused and nothing \
changes. Prefer diapo_edit_deck for a small change and diapo_write_deck to rewrite the whole deck.";

/// What every tool shares: the app state and the origin deck links start with.
#[derive(Clone)]
pub struct Context {
    /// The app state.
    pub state: AppState,
    /// The origin of the MCP resource, such as `https://diapo.facile.studio`.
    pub origin: Arc<str>,
}

impl Context {
    /// The page a person opens to see the deck: `https://<host>/decks/<id>`.
    pub fn deck_url(&self, id: Uuid) -> String {
        format!("{}/decks/{id}", self.origin)
    }
}

/// The porte user id of the caller.
pub fn user(call: &Call) -> Result<i64, ToolError> {
    call.caller.user_id.parse().map_err(|_| ToolError("the token names no local account".to_owned()))
}

/// Diapo's MCP server from `cfg`: the eight tools, and `diapo_delete_deck` confirmed on the
/// client's `/pending` page. `cfg.pending` must be set.
pub fn app(state: &AppState, cfg: AppConfig) -> Result<App, String> {
    let spec = AppSpec {
        name: "diapo".to_owned(),
        title: "Diapo".to_owned(),
        instructions: INSTRUCTIONS.to_owned(),
        kinds: HashMap::from([(KIND.to_owned(), delete::kind(state))]),
        ..AppSpec::default()
    };
    let mut app = App::new(cfg, spec)?;
    let confirmations = app.confirmations.clone().ok_or("diapo: diapo_delete_deck needs confirmations")?;
    let context = Context { state: state.clone(), origin: Arc::from(app.url.as_str()) };
    app.options.tools = tools::all(&context, &confirmations);
    Ok(app)
}
