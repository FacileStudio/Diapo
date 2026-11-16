//! `diapo logout`: ends the stored session and removes its token.

use std::env;

use serde_json::json;

use crate::api::Client;
use crate::commands::print_json;
use crate::config::{self, ENV_TOKEN};
use crate::failure::Outcome;
use crate::ui;

/// Removes the stored token. Running it when not logged in is not an error.
pub fn run() -> Outcome {
    let stored = config::stored();
    if let Some(token) = &stored.token
        && Client::new(&stored.server, Some(token)).logout().is_err()
    {
        ui::warn("The server did not confirm the sign-out, the session ends when it expires");
    }
    let removed = config::clear_token()?;
    if env::var_os(ENV_TOKEN).is_some() {
        ui::warn(&format!("{ENV_TOKEN} is still set in this shell"));
    }
    if ui::json() {
        print_json(&json!({ "logged_out": removed }));
    } else if removed {
        ui::success(&format!("Signed out of {}", stored.server));
    } else {
        println!("Not logged in");
    }
    Ok(())
}
