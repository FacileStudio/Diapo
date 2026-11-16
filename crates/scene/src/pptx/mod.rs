//! Writes a project as an editable PowerPoint file (`.pptx`, Office Open XML).
//!
//! Each scene is one slide of the deck's stage ratio. A markdown scene becomes real, editable
//! shapes: its first heading is the slide title, paragraphs, headings, quotes, code and list
//! items (with their nesting level, bulleted or numbered) flow in text boxes, a table is a
//! table, a PNG, JPEG or GIF picture is embedded. The presenter note goes to the notes page.
//! The `facile` theme gives the colours. A `raw` scene is an HTML composition: it becomes a
//! slide that shows the picture a renderer made of it ([`write_with`]), or says so in a text box
//! ([`RAW_TEXT`]) when there is none; its notes say which. Step reveals and timelines do not exist in a file: every step is shown.

mod geometry;
mod image;
mod layout;
mod masters;
mod model;
mod palette;
mod parts;
mod read;
mod shapes;
mod slide;
mod text;
mod theme;
mod xml;

use std::io::{Cursor, Write};

use zip::ZipWriter;
use zip::write::SimpleFileOptions;

use crate::model::Project;

pub use slide::{RAW_PICTURED, RAW_TEXT};

/// Reads the project's files for the writer: the web app answers from its store, the CLI from
/// the folder.
pub trait Files {
    /// The bytes of the file at `path`, a relative path as the source wrote it, or `None` when
    /// the project has no such file.
    fn read(&self, path: &str) -> Option<Vec<u8>>;
}

impl<F: Fn(&str) -> Option<Vec<u8>>> Files for F {
    fn read(&self, path: &str) -> Option<Vec<u8>> {
        self(path)
    }
}

/// Pictures of scenes, made by a renderer, for the scenes a file cannot hold as shapes.
pub trait Stills {
    /// A PNG, JPEG or GIF of the scene at `index` (from 0) at the stage ratio, or `None`.
    fn still(&self, index: usize) -> Option<Vec<u8>>;
}

impl<F: Fn(usize) -> Option<Vec<u8>>> Stills for F {
    fn still(&self, index: usize) -> Option<Vec<u8>> {
        self(index)
    }
}

/// The archive could not be written.
#[derive(Debug, thiserror::Error)]
#[error("failed to write the pptx archive: {0}")]
pub struct PptxError(String);

fn failed(error: impl std::fmt::Display) -> PptxError {
    PptxError(error.to_string())
}

fn parts(project: &Project, files: &dyn Files, stills: &dyn Stills) -> Vec<(String, Vec<u8>)> {
    let config = project.config();
    let stage = geometry::Stage::of(config.size);
    let palette = palette::of(config.scheme);
    let mut deck = slide::Deck { stage, palette, files, stills, media: Default::default() };
    let mut out: Vec<(String, Vec<u8>)> = Vec::new();
    let mut add = |path: String, text: String| out.push((path, text.into_bytes()));
    let mut noted = Vec::new();
    for (index, scene) in project.scenes().iter().enumerate() {
        let (number, made) = (index + 1, slide::slide(scene, index + 1, &mut deck));
        add(format!("ppt/slides/slide{number}.xml"), made.xml);
        add(format!("ppt/slides/_rels/slide{number}.xml.rels"), made.rels);
        if let Some(note) = made.note {
            noted.push(number);
            add(format!("ppt/notesSlides/notesSlide{number}.xml"), masters::notes(&note));
            add(format!("ppt/notesSlides/_rels/notesSlide{number}.xml.rels"), masters::notes_rels(number));
        }
    }
    let slides = project.scenes().len();
    add("[Content_Types].xml".into(), parts::content_types(slides, &noted));
    add("_rels/.rels".into(), parts::root_rels());
    add("ppt/presentation.xml".into(), parts::presentation(slides, &stage));
    add("ppt/_rels/presentation.xml.rels".into(), parts::presentation_rels(slides));
    add("ppt/slideMasters/slideMaster1.xml".into(), masters::master(&stage, &deck.palette));
    add("ppt/slideLayouts/slideLayout1.xml".into(), masters::layout(&stage));
    add("ppt/notesMasters/notesMaster1.xml".into(), masters::notes_master());
    add("ppt/theme/theme1.xml".into(), theme::part(&deck.palette));
    add("ppt/theme/theme2.xml".into(), theme::part(&palette::of(crate::model::Scheme::Light)));
    parts::fixed_rels().into_iter().for_each(|(path, text)| add(path.to_owned(), text));
    out.extend(deck.media.parts.into_iter().map(|(name, bytes)| (format!("ppt/media/{name}"), bytes)));
    out
}

/// Writes `project` as a `.pptx` file and answers its bytes. Pictures are read through `files`;
/// one the project lacks, or that is not a PNG, a JPEG or a GIF, leaves a line naming it.
pub fn write(project: &Project, files: &dyn Files) -> Result<Vec<u8>, PptxError> {
    write_with(project, files, &|_: usize| None)
}

/// [`write`], with a picture for each `raw` scene `stills` has one for: the slide then shows
/// the picture, full size, instead of the text that says it is animated HTML.
pub fn write_with(project: &Project, files: &dyn Files, stills: &dyn Stills) -> Result<Vec<u8>, PptxError> {
    let mut all = parts(project, files, stills);
    all.sort_by_key(|(path, _)| path != "[Content_Types].xml");
    let mut archive = ZipWriter::new(Cursor::new(Vec::new()));
    for (path, bytes) in all {
        archive.start_file(path, SimpleFileOptions::default()).map_err(failed)?;
        archive.write_all(&bytes).map_err(failed)?;
    }
    archive.finish().map(Cursor::into_inner).map_err(failed)
}
