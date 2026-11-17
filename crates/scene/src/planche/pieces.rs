//! A markdown scene with nothing but text and pictures, as editable text and image shapes.

use ::planche::{Shape, Style, Value};
use pulldown_cmark::{Event, Options, Parser, Tag};

use super::legacy::{boxed, rich};
use crate::model::{Layout, Scene, Size};
use crate::pptx::geometry::{Frame, Stage};
use crate::pptx::layout::{Piece, place};
use crate::pptx::model::{Block, Run};
use crate::pptx::read::blocks;

/// Whether `scene` maps to editable shapes: a markdown scene with no step, clip, table, raw
/// HTML or code block.
pub(crate) fn editable(scene: &Scene) -> bool {
    let special = |event: &Event<'_>| {
        matches!(event, Event::Html(_) | Event::InlineHtml(_) | Event::Start(Tag::Table(_) | Tag::CodeBlock(_)))
    };
    scene.layout != Layout::Raw
        && scene.steps == 0
        && !Parser::new_ext(&scene.body, Options::ENABLE_TABLES).any(|e| special(&e))
}

fn escaped(text: &str) -> String {
    text.chars().fold(String::new(), |mut out, c| {
        if c.is_ascii_punctuation() {
            out.push('\\');
        }
        out.push(c);
        out
    })
}

fn inline(run: &Run) -> String {
    if run.code {
        let ticks = "`".repeat(run.text.split(|c| c != '`').map(str::len).max().unwrap_or(0) + 1);
        return format!("{ticks} {} {ticks}", run.text);
    }
    let text = run.text.trim();
    let mark = match (run.bold, run.italic) {
        (true, true) => "***",
        (true, false) => "**",
        (false, true) => "*",
        (false, false) => "",
    };
    let (lead, tail) =
        (&run.text[..run.text.len() - run.text.trim_start().len()], &run.text[run.text.trim_end().len()..]);
    match text.is_empty() {
        true => run.text.clone(),
        false => format!("{lead}{mark}{}{mark}{tail}", escaped(text)),
    }
}

fn line(block: &Block) -> String {
    let runs = |runs: &[Run]| runs.iter().map(inline).collect::<String>();
    match block {
        Block::Heading(level, text) => format!("{} {}", "#".repeat(usize::from(*level)), runs(text)),
        Block::Item { level, ordered, runs: text } => {
            format!("{}{} {}", "    ".repeat(usize::from(*level)), if *ordered { "1." } else { "-" }, runs(text))
        }
        Block::Quote(text) => format!("> {}", runs(text)),
        Block::Paragraph(text) => runs(text),
        Block::Code(text) => format!("```\n{text}```"),
        Block::Table(_) | Block::Image { .. } => String::new(),
    }
}

fn markdown(group: &[&Block]) -> String {
    let mut out = String::new();
    for (index, block) in group.iter().enumerate() {
        let listed =
            matches!(block, Block::Item { .. }) && matches!(group.get(index.wrapping_sub(1)), Some(Block::Item { .. }));
        out.push_str(match (index, listed) {
            (0, _) => "",
            (_, true) => "\n",
            _ => "\n\n",
        });
        out.push_str(&line(block));
    }
    out
}

/// The text and image shapes of an [`editable`] scene, placed as the scene lays out on `area`.
pub(crate) fn shapes(scene: &Scene, area: Frame) -> Vec<Shape> {
    let size = Size { width: area.width as u32, height: area.height as u32 };
    let stage = Stage::of(size);
    let read = blocks(&scene.body);
    let natural = |_: &str| Some((stage.width, stage.height));
    let align =
        (scene.layout == Layout::Center).then(|| Style { align: Some("center".to_owned()), ..Style::default() });
    let text = |group: &[&Block], frame| {
        let mut fields = vec![("text", rich(&markdown(group)))];
        fields.extend(align.clone().map(|style| ("style", Value::Style(style))));
        boxed("text", frame, fields)
    };
    place(&read, scene.layout, &stage, &natural)
        .into_iter()
        .filter_map(|placed| match placed.piece {
            Piece::Title(block) => Some(text(&[block], placed.frame)),
            Piece::Text(group) => Some(text(&group, placed.frame)),
            Piece::Image { path, .. } => Some(boxed(
                "image",
                placed.frame,
                vec![("asset", Value::Str(path.to_owned())), ("fit", Value::Str("contain".to_owned()))],
            )),
            Piece::Table(_) => None,
        })
        .collect()
}
