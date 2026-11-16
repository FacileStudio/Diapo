//! Cuts a source file into blocks: one per scene, each with its frontmatter lines and its body.

use crate::error::{Error, ErrorKind};

/// One scene before it is read: frontmatter lines with their line numbers, then the body.
#[derive(Debug, Default)]
pub(crate) struct Block<'a> {
    pub front: Vec<(usize, &'a str)>,
    pub body: Vec<&'a str>,
    pub line: usize,
}

fn separator(line: &str) -> bool {
    line.trim_end() == "---"
}

fn key_line(line: &str) -> bool {
    let name = line.split(':').next().unwrap_or_default();
    let named = !name.is_empty() && name.bytes().all(|b| b.is_ascii_lowercase() || b == b'_' || b.is_ascii_digit());
    named && line[name.len()..].starts_with(':') && matches!(line.as_bytes().get(name.len() + 1), None | Some(b' '))
}

fn fence(line: &str) -> Option<char> {
    let text = line.trim_start();
    let mark = text.chars().next().filter(|c| *c == '`' || *c == '~')?;
    (line.len() - text.len() < 4 && text.chars().take_while(|c| *c == mark).count() >= 3).then_some(mark)
}

fn front<'a>(lines: &[&'a str], at: &mut usize, errors: &mut Vec<Error>) -> Vec<(usize, &'a str)> {
    let opens = lines.get(*at).is_some_and(|line| key_line(line) || separator(line));
    if !opens {
        return Vec::new();
    }
    let opened = *at;
    let mut found = Vec::new();
    while let Some(line) = lines.get(*at) {
        *at += 1;
        if separator(line) {
            return found;
        }
        match line.trim() {
            text if text.is_empty() || text.starts_with('#') => {}
            _ if key_line(line) => found.push((*at, *line)),
            _ => errors.push(Error { line: *at, kind: ErrorKind::BadFrontmatterLine }),
        }
    }
    errors.push(Error { line: opened, kind: ErrorKind::UnclosedFrontmatter });
    found
}

fn body<'a>(lines: &[&'a str], at: &mut usize) -> Vec<&'a str> {
    let mut found = Vec::new();
    let mut fenced: Option<char> = None;
    while let Some(line) = lines.get(*at) {
        if fenced.is_none() && separator(line) {
            break;
        }
        match (fenced, fence(line)) {
            (None, mark) => fenced = mark,
            (Some(open), Some(mark)) if open == mark && line.trim().chars().all(|c| c == mark) => fenced = None,
            _ => {}
        }
        found.push(*line);
        *at += 1;
    }
    found
}

fn blank(block: &Block<'_>) -> bool {
    block.front.is_empty() && block.body.iter().all(|line| line.trim().is_empty())
}

/// Splits `source` on `---` lines. The first block carries the headmatter as its frontmatter.
pub(crate) fn split<'a>(source: &'a str, errors: &mut Vec<Error>) -> Vec<Block<'a>> {
    let lines: Vec<&str> = source.trim_start_matches('\u{feff}').lines().collect();
    let mut opened = lines.first().is_some_and(|line| separator(line));
    let mut at = usize::from(opened);
    let mut blocks = Vec::new();
    loop {
        let front = if opened { front(&lines, &mut at, errors) } else { Vec::new() };
        opened = true;
        let line = at + 1;
        blocks.push(Block { front, body: body(&lines, &mut at), line });
        if at >= lines.len() {
            break;
        }
        at += 1;
    }
    if blocks.last().is_some_and(blank) {
        blocks.pop();
    }
    blocks
}
