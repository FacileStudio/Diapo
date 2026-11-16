//! Renders a markdown scene body to HTML, turning step markers into `data-step` attributes.

use std::ops::Range;

use pulldown_cmark::{CowStr, Event, Options, Parser, Tag, html};

use crate::audit::Audit;
use crate::error::ErrorKind;
use crate::marker;

enum Slot {
    Block(usize),
    Literal,
    Other,
}

struct Walker<'a> {
    body: &'a str,
    events: Vec<Event<'a>>,
    open: Vec<Slot>,
    marked: Vec<(usize, usize, u32)>,
    chunk: Range<usize>,
    audit: Audit,
}

impl<'a> Walker<'a> {
    fn start(&mut self, tag: &Tag<'a>, offset: usize) {
        let at = self.events.len();
        self.open.push(match tag {
            Tag::Paragraph | Tag::Heading { .. } | Tag::Item | Tag::BlockQuote(_) => Slot::Block(at),
            Tag::CodeBlock(_) => Slot::Literal,
            _ => Slot::Other,
        });
        if let Tag::Image { dest_url, .. } = tag {
            self.audit.url(dest_url, offset);
        }
    }

    fn end(&mut self) {
        let Some(Slot::Block(opened)) = self.open.pop() else { return };
        let at = self.events.len();
        self.marked.iter_mut().filter(|mark| mark.0 == opened).for_each(|mark| mark.1 = at);
    }

    fn target(&self) -> Option<usize> {
        let mut blocks = self.open.iter().rev().filter_map(|slot| match slot {
            Slot::Block(at) => Some(*at),
            _ => None,
        });
        let found = blocks.next()?;
        let holder = |at: &usize| matches!(self.events[*at], Event::Start(Tag::Item | Tag::BlockQuote(_)));
        match (&self.events[found], blocks.next().filter(holder)) {
            (Event::Start(Tag::Paragraph), Some(holder)) => Some(holder),
            _ => Some(found),
        }
    }

    fn text(&mut self, text: CowStr<'a>, offset: usize) {
        if matches!(self.open.last(), Some(Slot::Literal)) || !text.contains("{step=") {
            return self.events.push(Event::Text(text));
        }
        let (clean, found) = marker::strip(&text);
        for step in found {
            self.audit.step(step, offset);
            match (step, self.target()) {
                (Some(step), Some(block)) => self.marked.push((block, usize::MAX, step)),
                (Some(_), None) => self.audit.problems.push((offset, ErrorKind::StepMarkerOutsideBlock)),
                (None, _) => {}
            }
        }
        if !clean.is_empty() {
            self.events.push(Event::Text(clean.into()));
        }
    }

    fn markup(&mut self, range: Range<usize>) {
        let joined = !self.chunk.is_empty() && self.body[self.chunk.end..range.start].trim().is_empty();
        if joined {
            self.chunk.end = range.end;
            return;
        }
        self.audit.html(&self.body[self.chunk.clone()], self.chunk.start);
        self.chunk = range;
    }

    fn finish(mut self) -> (String, Audit) {
        self.audit.html(&self.body[self.chunk.clone()], self.chunk.start);
        self.marked.sort_unstable();
        self.marked.dedup_by_key(|mark| mark.0);
        for (opened, closed, step) in self.marked {
            let (open, close) = marker::tags(&self.events[opened], step);
            self.events[opened] = Event::Html(open.into());
            self.events[closed] = Event::Html(close.into());
        }
        let mut out = String::new();
        html::push_html(&mut out, self.events.into_iter());
        (out, self.audit)
    }
}

/// Renders `body` (CommonMark with tables, raw HTML kept) and checks it on the way.
pub(crate) fn render(body: &str) -> (String, Audit) {
    let mut walker =
        Walker { body, events: Vec::new(), open: Vec::new(), marked: Vec::new(), chunk: 0..0, audit: Audit::default() };
    for (event, range) in Parser::new_ext(body, Options::ENABLE_TABLES).into_offset_iter() {
        match event {
            Event::Start(ref tag) => walker.start(tag, range.start),
            Event::End(_) => walker.end(),
            Event::Html(_) | Event::InlineHtml(_) => walker.markup(range.clone()),
            _ => {}
        }
        match event {
            Event::Text(text) => walker.text(text, range.start),
            other => walker.events.push(other),
        }
    }
    walker.finish()
}
