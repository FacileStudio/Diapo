//! A small HTML tag scanner: element start tags and their attributes, with byte offsets.

use std::ops::Range;

/// One attribute of a start tag. `span` is where the value sits in the scanned text.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Attr<'a> {
    pub name: &'a str,
    pub value: &'a str,
    pub span: Range<usize>,
}

/// One start tag. `offset` is where its `<` sits in the scanned text.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Tag<'a> {
    pub name: &'a str,
    pub offset: usize,
    pub attrs: Vec<Attr<'a>>,
}

struct Scanner<'a> {
    text: &'a str,
    at: usize,
}

impl<'a> Scanner<'a> {
    fn peek(&self) -> Option<u8> {
        self.text.as_bytes().get(self.at).copied()
    }

    fn take(&mut self, keep: impl Fn(u8) -> bool) -> &'a str {
        let start = self.at;
        while self.peek().is_some_and(&keep) {
            self.at += 1;
        }
        &self.text[start..self.at]
    }

    fn skip_past(&mut self, needle: &str) {
        let rest = self.text[self.at..].to_ascii_lowercase();
        self.at = rest.find(needle).map_or(self.text.len(), |found| self.at + found + needle.len());
    }

    fn value(&mut self) -> Range<usize> {
        match self.peek() {
            Some(quote @ (b'"' | b'\'')) => {
                self.at += 1;
                let start = self.at;
                self.take(|b| b != quote);
                let end = self.at;
                self.at = (end + 1).min(self.text.len());
                start..end
            }
            _ => {
                let start = self.at;
                self.take(|b| !b.is_ascii_whitespace() && b != b'>');
                start..self.at
            }
        }
    }

    fn attr(&mut self) -> Option<Attr<'a>> {
        self.take(|b| b.is_ascii_whitespace() || b == b'/');
        let name = self.take(|b| !b.is_ascii_whitespace() && !matches!(b, b'=' | b'>' | b'/'));
        if name.is_empty() {
            return None;
        }
        self.take(|b| b.is_ascii_whitespace());
        let mut span = self.at..self.at;
        if self.peek() == Some(b'=') {
            self.at += 1;
            self.take(|b| b.is_ascii_whitespace());
            span = self.value();
        }
        Some(Attr { name, value: &self.text[span.clone()], span })
    }

    fn tag(&mut self) -> Tag<'a> {
        let offset = self.at;
        self.at += 1;
        let name = self.take(|b| b.is_ascii_alphanumeric() || b == b'-');
        let mut attrs = Vec::new();
        while let Some(attr) = self.attr() {
            attrs.push(attr);
        }
        self.take(|b| b != b'>');
        self.at = (self.at + 1).min(self.text.len());
        if name.eq_ignore_ascii_case("script") || name.eq_ignore_ascii_case("style") {
            self.skip_past(&format!("</{}", name.to_ascii_lowercase()));
        }
        Tag { name, offset, attrs }
    }
}

/// Every start tag of `html`, in order. Comments and the text of `script` and `style` are
/// skipped.
pub(crate) fn tags(html: &str) -> Vec<Tag<'_>> {
    let mut scanner = Scanner { text: html, at: 0 };
    let mut found = Vec::new();
    while let Some(next) = html[scanner.at..].find('<') {
        scanner.at += next;
        let rest = &html[scanner.at..];
        if rest.starts_with("<!--") {
            scanner.skip_past("-->");
        } else if rest[1..].starts_with(|c: char| c.is_ascii_alphabetic()) {
            found.push(scanner.tag());
        } else {
            scanner.at += 1;
        }
    }
    found
}
