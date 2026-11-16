//! One scene as one slide part.

use std::collections::HashMap;

use super::geometry::Stage;
use super::image::{Picture, picture};
use super::layout::{Piece, Placed, place};
use super::model::{Block, Run};
use super::palette::Palette;
use super::text::Style;
use super::xml::{HEAD, NS, TREE, rels};
use super::{Files, read, shapes};
use crate::model::{Layout, Scene};

/// What a `raw` scene shows until it can be rendered to a picture.
pub const RAW_TEXT: &str = "This slide is animated HTML. It is shown as an image once rendering is available.";

const RAW_TITLE: &str = "Animated slide";

/// The pictures a deck embeds, each stored once.
#[derive(Default)]
pub(crate) struct Media {
    /// The media parts, as `(file name, bytes)`.
    pub parts: Vec<(String, Vec<u8>)>,
    known: HashMap<String, Option<(usize, Picture)>>,
}

impl Media {
    /// The part index and facts of the picture at `path`, read through `files` the first time.
    /// `None` when the project has no such file or it is not a PNG, a JPEG or a GIF.
    fn find(&mut self, path: &str, files: &dyn Files) -> Option<(usize, Picture)> {
        if let Some(known) = self.known.get(path) {
            return known.clone();
        }
        let found = files.read(path).and_then(|bytes| Some((picture(&bytes)?, bytes))).map(|(facts, bytes)| {
            self.parts.push((format!("image{}.{}", self.parts.len() + 1, facts.extension), bytes));
            (self.parts.len() - 1, facts)
        });
        self.known.insert(path.to_owned(), found.clone());
        found
    }
}

/// What one slide part needs from its deck.
pub(crate) struct Deck<'a> {
    pub stage: Stage,
    pub palette: Palette,
    pub files: &'a dyn Files,
    pub media: Media,
}

/// One slide: its XML, its relationships and the text of its notes page.
pub(crate) struct Slide {
    pub xml: String,
    pub rels: String,
    pub note: Option<String>,
}

fn text(value: &str) -> Vec<Run> {
    vec![Run { text: value.to_owned(), ..Run::default() }]
}

fn blocks(scene: &Scene, deck: &mut Deck<'_>) -> Vec<Block> {
    if scene.layout == Layout::Raw {
        return vec![Block::Heading(2, text(RAW_TITLE)), Block::Paragraph(text(RAW_TEXT))];
    }
    let shown = |block: Block, deck: &mut Deck<'_>| match &block {
        Block::Image { path, alt } if deck.media.find(path, deck.files).is_none() => {
            let label = if alt.is_empty() { path } else { alt };
            Block::Quote(text(&format!("[picture: {label}]")))
        }
        _ => block,
    };
    read::blocks(&scene.body).into_iter().map(|block| shown(block, deck)).collect()
}

fn shape(
    placed: &Placed<'_>,
    id: usize,
    style: &Style<'_>,
    links: &mut Vec<(String, &'static str, String)>,
    deck: &Deck<'_>,
) -> String {
    match &placed.piece {
        Piece::Title(block) => shapes::text_box(id, &placed.frame, &[block], style, true),
        Piece::Text(group) => shapes::text_box(id, &placed.frame, group, style, false),
        Piece::Table(rows) => shapes::table(id, &placed.frame, rows, style),
        Piece::Image { path, alt, .. } => {
            let part = deck.media.known.get(*path).cloned().flatten().map_or(0, |(part, _)| part);
            let relation = format!("rId{}", links.len() + 1);
            links.push((relation.clone(), "image", format!("../media/{}", deck.media.parts[part].0)));
            shapes::picture(id, &placed.frame, style.stage, &relation, alt)
        }
    }
}

fn note(scene: &Scene) -> Option<String> {
    let written = scene.note.clone().filter(|note| !note.trim().is_empty());
    match (scene.layout == Layout::Raw, written) {
        (true, Some(note)) => Some(format!("{RAW_TEXT}\n{note}")),
        (true, None) => Some(RAW_TEXT.to_owned()),
        (false, written) => written,
    }
}

/// The slide of `scene`, numbered `number` from 1.
pub(crate) fn slide(scene: &Scene, number: usize, deck: &mut Deck<'_>) -> Slide {
    let read = blocks(scene, deck);
    let layout = if scene.layout == Layout::Raw { Layout::Center } else { scene.layout };
    let natural = |path: &str| {
        let (_, facts) = deck.media.known.get(path).cloned().flatten()?;
        Some((f64::from(facts.width), f64::from(facts.height)))
    };
    let placed = place(&read, layout, &deck.stage, &natural);
    let style = Style {
        stage: &deck.stage,
        palette: &deck.palette,
        centered: layout == Layout::Center,
        cover: layout == Layout::Cover,
        size: None,
    };
    let note = note(scene);
    let mut links = vec![("rId1".to_owned(), "slideLayout", "../slideLayouts/slideLayout1.xml".to_owned())];
    if note.is_some() {
        links.push(("rId2".to_owned(), "notesSlide", format!("../notesSlides/notesSlide{number}.xml")));
    }
    let drawn: String =
        placed.iter().enumerate().map(|(index, placed)| shape(placed, index + 2, &style, &mut links, deck)).collect();
    let xml = format!(
        "{HEAD}<p:sld {NS}><p:cSld><p:spTree>{TREE}{drawn}</p:spTree></p:cSld><p:clrMapOvr><a:masterClrMapping/>\
         </p:clrMapOvr></p:sld>"
    );
    Slide { xml, rels: rels(&links), note }
}
