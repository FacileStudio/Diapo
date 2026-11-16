//! The `{step=k}` reveal marker of markdown scenes.

use pulldown_cmark::{Event, Tag};

const OPEN: &str = "{step=";

/// Removes every marker from `text`. Answers the text left and, for each marker, its step,
/// or `None` when it is malformed (a malformed marker stays in the text).
pub(crate) fn strip(text: &str) -> (String, Vec<Option<u32>>) {
    let mut clean = String::new();
    let mut found = Vec::new();
    let mut rest = text;
    while let Some(at) = rest.find(OPEN) {
        let after = &rest[at + OPEN.len()..];
        let digits = after.bytes().take_while(u8::is_ascii_digit).count();
        let step = after[..digits].parse::<u32>().ok().filter(|step| (1..=999).contains(step));
        match step.filter(|_| after[digits..].starts_with('}')) {
            Some(step) => {
                clean.push_str(rest[..at].trim_end());
                rest = &after[digits + 1..];
                found.push(Some(step));
            }
            None => {
                clean.push_str(&rest[..at + OPEN.len()]);
                rest = after;
                found.push(None);
            }
        }
    }
    clean.push_str(rest);
    (clean, found)
}

/// The opening and closing HTML of the block `event` starts, revealed at `step`.
pub(crate) fn tags(event: &Event<'_>, step: u32) -> (String, String) {
    let name = match event {
        Event::Start(Tag::Heading { level, .. }) => level.to_string(),
        Event::Start(Tag::Item) => "li".to_owned(),
        Event::Start(Tag::BlockQuote(_)) => "blockquote".to_owned(),
        _ => "p".to_owned(),
    };
    (format!("<{name} data-step=\"{step}\">"), format!("</{name}>\n"))
}
