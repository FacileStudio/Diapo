//! Checks an HTML fragment: its timing attributes and the files it loads.

use crate::error::ErrorKind;
use crate::html::{Tag, tags};
use crate::references::{judge, references};

/// What the checks of one scene found.
#[derive(Debug, Default)]
pub(crate) struct Audit {
    /// The highest step a marker or a `data-step` names.
    pub steps: u32,
    /// The latest end of a clip, in seconds.
    pub clip_end: f64,
    /// Problems, each at a byte offset of the scene body.
    pub problems: Vec<(usize, ErrorKind)>,
}

fn seconds(tag: &Tag<'_>, name: &str) -> Option<Result<f64, ErrorKind>> {
    let attr = tag.attrs.iter().find(|attr| attr.name.eq_ignore_ascii_case(name))?;
    let parsed = attr.value.trim().parse::<f64>().ok().filter(|n| n.is_finite() && *n >= 0.0);
    Some(parsed.ok_or(ErrorKind::BadValue { key: name.to_owned(), expected: "a number of seconds, zero or more" }))
}

impl Audit {
    /// Records the step `raw` names, or a problem at `offset` when it is not a step.
    pub(crate) fn step(&mut self, step: Option<u32>, offset: usize) {
        match step {
            Some(step) => self.steps = self.steps.max(step),
            None => self.problems.push((offset, ErrorKind::BadStepMarker)),
        }
    }

    /// Checks one URL written at `offset`.
    pub(crate) fn url(&mut self, url: &str, offset: usize) {
        if let Some(kind) = judge(url) {
            self.problems.push((offset, kind));
        }
    }

    fn timing(&mut self, tag: &Tag<'_>, offset: usize) {
        if let Some(attr) = tag.attrs.iter().find(|attr| attr.name.eq_ignore_ascii_case("data-step")) {
            self.step(attr.value.trim().parse().ok().filter(|step| (1..=999).contains(step)), offset);
        }
        let mut read = |name| match seconds(tag, name) {
            Some(Err(kind)) => {
                self.problems.push((offset, kind));
                None
            }
            Some(Ok(value)) => Some(value),
            None => None,
        };
        let start = read("data-start").unwrap_or_default();
        if let Some(duration) = read("data-duration") {
            self.clip_end = self.clip_end.max(start + duration);
        }
    }

    /// Checks `html`, a fragment that starts at byte `base` of the scene body.
    pub(crate) fn html(&mut self, html: &str, base: usize) {
        for tag in tags(html) {
            self.timing(&tag, base + tag.offset);
        }
        for reference in references(html) {
            self.url(reference.url, base + reference.span.start);
        }
    }
}
