//! `diapo guide`: the format reference, for people and for agents.

use serde_json::json;

use crate::commands::print_json;
use crate::failure::Outcome;
use crate::ui;

/// What the CLI adds to the planche guide: the older format still works here.
pub const LEGACY: &str = "\nA `diapo.md` in the older scene format is still accepted: `check`, `build` and `serve` \
read it as it is, and `push` sends it for the server to convert.\n";

/// Prints `scene::planche::GUIDE`, then [`LEGACY`].
pub fn run() -> Outcome {
    let guide = [scene::planche::GUIDE, LEGACY].concat();
    if ui::json() {
        print_json(&json!({ "guide": guide }));
    } else {
        print!("{guide}");
    }
    Ok(())
}
