//! `diapo export`: not built yet, and says so.

use crate::failure::{Failure, Outcome};

/// Fails with where each export comes from.
pub fn run() -> Outcome {
    Err(Failure::new(
        "not_available",
        "PDF, PPTX and MP4 exports are not available yet, they arrive with the server's export feature \
         — run `diapo build` for a standalone HTML file",
    ))
}
