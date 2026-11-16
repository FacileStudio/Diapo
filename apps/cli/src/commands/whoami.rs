//! `diapo whoami`: the account behind the token.

use serde_json::json;

use crate::cli::Globals;
use crate::commands::{print_json, session};
use crate::failure::Outcome;
use crate::ui;

/// Prints the account's email and the server it is on.
pub fn run(globals: &Globals) -> Outcome {
    let client = session(globals.server.as_deref())?;
    let me = client.me()?;
    if ui::json() {
        print_json(&json!({ "server": client.server(), "email": me.email, "name": me.name }));
    } else {
        println!("{} on {}", me.email, client.server());
    }
    Ok(())
}
