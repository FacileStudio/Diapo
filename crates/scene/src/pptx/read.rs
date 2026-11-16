//! Reads a markdown scene body into [`Block`]s.

use pulldown_cmark::{Event, Options, Parser, Tag, TagEnd};

use super::model::{Block, Run};
use crate::marker;

#[derive(Default)]
struct Reader {
    blocks: Vec<Block>,
    runs: Vec<Run>,
    strong: u32,
    emphasis: u32,
    quotes: u32,
    lists: Vec<bool>,
    code: Option<String>,
    image: Option<(String, String)>,
    rows: Vec<Vec<Vec<Run>>>,
    row: Vec<Vec<Run>>,
}

impl Reader {
    fn item(&mut self) {
        let runs = std::mem::take(&mut self.runs);
        let Some(ordered) = self.lists.last().copied().filter(|_| !runs.is_empty()) else { return };
        self.blocks.push(Block::Item { level: (self.lists.len() - 1).min(8) as u8, ordered, runs });
    }

    fn paragraph(&mut self) {
        if !self.lists.is_empty() {
            return self.item();
        }
        let runs = std::mem::take(&mut self.runs);
        match (runs.is_empty(), self.quotes > 0) {
            (true, _) => {}
            (false, true) => self.blocks.push(Block::Quote(runs)),
            (false, false) => self.blocks.push(Block::Paragraph(runs)),
        }
    }

    fn start(&mut self, tag: Tag<'_>) {
        match tag {
            Tag::Strong => self.strong += 1,
            Tag::Emphasis => self.emphasis += 1,
            Tag::BlockQuote(_) => self.quotes += 1,
            Tag::CodeBlock(_) => self.code = Some(String::new()),
            Tag::Image { dest_url, .. } => self.image = Some((dest_url.to_string(), String::new())),
            Tag::List(first) => {
                self.item();
                self.lists.push(first.is_some());
            }
            Tag::TableHead | Tag::TableRow => self.row.clear(),
            Tag::Heading { .. } | Tag::Paragraph | Tag::Item | Tag::TableCell => self.runs.clear(),
            _ => {}
        }
    }

    fn close(&mut self, tag: TagEnd) {
        match tag {
            TagEnd::Strong => self.strong = self.strong.saturating_sub(1),
            TagEnd::Emphasis => self.emphasis = self.emphasis.saturating_sub(1),
            TagEnd::BlockQuote(_) => self.quotes = self.quotes.saturating_sub(1),
            TagEnd::List(_) => _ = self.lists.pop(),
            TagEnd::CodeBlock => self.blocks.extend(self.code.take().map(Block::Code)),
            TagEnd::Image => self.blocks.extend(self.image.take().map(|(path, alt)| Block::Image { path, alt })),
            other => self.end(other),
        }
    }

    fn end(&mut self, tag: TagEnd) {
        match tag {
            TagEnd::Heading(level) => self.blocks.push(Block::Heading(level as u8, std::mem::take(&mut self.runs))),
            TagEnd::Paragraph => self.paragraph(),
            TagEnd::Item => self.item(),
            TagEnd::TableCell => self.row.push(std::mem::take(&mut self.runs)),
            TagEnd::TableHead | TagEnd::TableRow => self.rows.push(std::mem::take(&mut self.row)),
            TagEnd::Table => self.blocks.push(Block::Table(std::mem::take(&mut self.rows))),
            _ => {}
        }
    }

    fn text(&mut self, text: &str, code: bool) {
        if let Some(block) = self.code.as_mut() {
            return block.push_str(text);
        }
        let clean = if code { text.to_owned() } else { marker::strip(text).0 };
        if let Some((_, alt)) = self.image.as_mut() {
            return alt.push_str(&clean);
        }
        let run = Run { text: clean, bold: self.strong > 0, italic: self.emphasis > 0, code };
        match self.runs.last_mut().filter(|last| (last.bold, last.italic, last.code) == (run.bold, run.italic, code)) {
            Some(last) => last.text.push_str(&run.text),
            None if run.text.is_empty() => {}
            None => self.runs.push(run),
        }
    }
}

/// The blocks of a markdown scene body. Step markers are dropped: a slide shows every step.
/// Raw HTML is skipped.
pub(crate) fn blocks(body: &str) -> Vec<Block> {
    let mut reader = Reader::default();
    for event in Parser::new_ext(body, Options::ENABLE_TABLES) {
        match event {
            Event::Start(tag) => reader.start(tag),
            Event::End(tag) => reader.close(tag),
            Event::Text(text) => reader.text(&text, false),
            Event::Code(text) => reader.text(&text, true),
            Event::SoftBreak | Event::HardBreak => reader.text(" ", false),
            _ => {}
        }
    }
    reader.blocks
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pptx::model::plain;

    #[test]
    fn a_body_becomes_headings_paragraphs_and_leveled_items_without_markers() {
        let body = "## Revenue\n\nUp **12%** this `q3`.\n\n- One {step=1}\n  - Nested\n- Two\n\n1. First\n";
        let read = blocks(body);
        assert_eq!(read[0], Block::Heading(2, vec![Run { text: "Revenue".into(), ..Run::default() }]));
        let Block::Paragraph(runs) = &read[1] else { panic!("{read:?}") };
        assert_eq!((runs[1].text.as_str(), runs[1].bold, runs[3].code), ("12%", true, true));
        let items: Vec<(u8, bool, String)> = read[2..]
            .iter()
            .map(|block| match block {
                Block::Item { level, ordered, runs } => (*level, *ordered, plain(runs)),
                other => panic!("{other:?}"),
            })
            .collect();
        let expected = [(0, false, "One"), (1, false, "Nested"), (0, false, "Two"), (0, true, "First")];
        assert_eq!(items.len(), expected.len());
        for (item, want) in items.iter().zip(expected) {
            assert_eq!((item.0, item.1, item.2.as_str()), want);
        }
    }
}
