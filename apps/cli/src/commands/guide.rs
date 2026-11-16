//! `diapo guide`: the format reference, for people and for agents.

use serde_json::json;

use crate::commands::print_json;
use crate::failure::Outcome;
use crate::ui;

/// Prints `scene::FORMAT_GUIDE`.
pub fn run() -> Outcome {
    if ui::json() {
        print_json(&json!({ "guide": scene::FORMAT_GUIDE }));
    } else {
        print!("{}", scene::FORMAT_GUIDE);
    }
    Ok(())
}
