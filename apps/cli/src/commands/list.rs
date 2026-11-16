//! `diapo list`: the decks the account can open.

use serde_json::json;

use crate::cli::{Globals, ListArgs};
use crate::commands::{print_json, session};
use crate::failure::Outcome;
use crate::ui;

/// Prints one deck per line: its id, then its title.
pub fn run(args: &ListArgs, globals: &Globals) -> Outcome {
    let decks = session(globals.server.as_deref())?.decks()?;
    if ui::json() {
        print_json(&json!(decks));
    } else if decks.is_empty() && !args.quiet {
        println!("No decks");
    }
    let width = decks.iter().map(|deck| deck.id.len()).max().unwrap_or(0);
    for deck in decks.iter().filter(|_| !ui::json()) {
        match args.quiet {
            true => println!("{}", deck.id),
            false => println!("{:width$}  {}", deck.id, deck.title),
        }
    }
    Ok(())
}
