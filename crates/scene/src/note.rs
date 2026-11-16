//! The presenter note: the HTML comment a scene body ends with.

/// Splits `body` into the body proper and the text of its trailing comment, if it has one.
pub(crate) fn take(body: &str) -> (String, Option<String>) {
    let text = body.trim_end();
    let open =
        text.rfind("<!--").filter(|at| text.ends_with("-->") && text[..*at].rsplit('\n').next().is_none_or(blank));
    match open {
        Some(at) if !text[at + 4..text.len() - 3].contains("-->") => {
            let note = text[at + 4..text.len() - 3].trim().to_owned();
            (text[..at].to_owned(), Some(note))
        }
        _ => (body.to_owned(), None),
    }
}

fn blank(line: &str) -> bool {
    line.trim().is_empty()
}
