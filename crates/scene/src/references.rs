//! Finds every place an HTML fragment names a file to load, and judges the URL written there.

use std::ops::Range;

use crate::error::ErrorKind;
use crate::html::{Attr, Tag, tags};

/// One URL an HTML fragment loads, with where it sits in the fragment.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Reference<'a> {
    pub url: &'a str,
    pub span: Range<usize>,
}

const LINKING: [&str; 4] = ["link", "image", "use", "script"];

fn loads(tag: &Tag<'_>, attr: &Attr<'_>) -> bool {
    let name = attr.name.to_ascii_lowercase();
    let linking = LINKING.iter().any(|known| tag.name.eq_ignore_ascii_case(known));
    match name.as_str() {
        "src" | "poster" | "srcset" => true,
        "href" | "xlink:href" => linking,
        "data" => tag.name.eq_ignore_ascii_case("object"),
        _ => false,
    }
}

fn trimmed(text: &str, span: Range<usize>) -> Reference<'_> {
    let quote = |c: char| c.is_whitespace() || c == '"' || c == '\'';
    let inner = text[span.clone()].trim_matches(quote);
    let start = span.start + text[span].find(inner).unwrap_or_default();
    Reference { url: inner, span: start..start + inner.len() }
}

fn candidates<'a>(text: &'a str, attr: &Attr<'a>, found: &mut Vec<Reference<'a>>) {
    if !attr.name.eq_ignore_ascii_case("srcset") {
        return found.push(Reference { url: attr.value, span: attr.span.clone() });
    }
    let mut start = attr.span.start;
    for part in attr.value.split(',') {
        let url = part.split_whitespace().next().unwrap_or_default();
        let at = start + part.find(url).unwrap_or_default();
        found.push(trimmed(text, at..at + url.len()));
        start += part.len() + 1;
    }
}

fn styles<'a>(text: &'a str, found: &mut Vec<Reference<'a>>) {
    for (at, _) in text.match_indices("url(") {
        let open = at + 4;
        let close = text[open..].find(')').map_or(text.len(), |end| open + end);
        found.push(trimmed(text, open..close));
    }
    for (at, _) in text.match_indices("@import") {
        let open = at + 7;
        let rest = text[open..].trim_start();
        let quoted = rest.starts_with(['"', '\'']).then(|| rest[1..].find(['"', '\''])).flatten();
        if let Some(end) = quoted {
            let start = text.len() - rest.len() + 1;
            found.push(trimmed(text, start..start + end));
        }
    }
}

/// Every URL `html` loads: resource attributes, CSS `url()` and `@import`, in source order.
pub(crate) fn references(html: &str) -> Vec<Reference<'_>> {
    let mut found = Vec::new();
    for tag in tags(html) {
        for attr in tag.attrs.iter().filter(|attr| loads(&tag, attr)) {
            candidates(html, attr, &mut found);
        }
    }
    styles(html, &mut found);
    found.sort_by_key(|reference| reference.span.start);
    found.dedup_by_key(|reference| reference.span.start);
    found
}

/// Whether `url` names a file of the project: not empty, not a fragment, not inline data.
pub(crate) fn local(url: &str) -> bool {
    let inline = url.get(..5).is_some_and(|head| head.eq_ignore_ascii_case("data:"));
    !url.is_empty() && !url.starts_with('#') && !inline
}

/// What is wrong with `url` as a reference to a project file, if anything.
pub(crate) fn judge(url: &str) -> Option<ErrorKind> {
    if !local(url) {
        return None;
    }
    let head = url.split(['/', '?', '#']).next().unwrap_or_default();
    if head.contains(':') || url.starts_with("//") {
        return Some(ErrorKind::RemoteUrl { url: url.to_owned() });
    }
    let leaves = url.starts_with(['/', '\\']) || url.split(['/', '\\']).any(|part| part == "..");
    leaves.then(|| ErrorKind::BadAssetPath { path: url.to_owned() })
}
