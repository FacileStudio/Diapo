//! The three things a capture job makes from a bundle: a thumbnail, stills, a PDF.

use std::ops::ControlFlow;
use std::path::Path;

use crate::Error;
use crate::browser::Browser;
use crate::guard::Guard;
use crate::page::{Page, Stop};
use crate::tools::Tools;
use crate::{flags, merge};

/// What a caller answers to each `(done, total)`: `Break` cancels the capture.
pub type OnProgress<'a> = &'a mut dyn FnMut(u32, u32) -> ControlFlow<()>;

/// A compiled deck to capture: a directory holding `index.html` and every file it loads.
#[derive(Debug, Clone, Copy)]
pub struct Deck<'a> {
    /// The bundle directory. The page may load files under it and nothing else.
    pub bundle: &'a Path,
    /// The stage width in pixels.
    pub width: u32,
    /// The stage height in pixels.
    pub height: u32,
}

/// What a capture answers besides its files.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Report {
    /// Pictures or pages made.
    pub made: u32,
    /// Requests the page made and was refused.
    pub refused: u32,
}

fn told(progress: OnProgress, done: u32, total: u32) -> Result<(), Error> {
    match progress(done, total) {
        ControlFlow::Continue(()) => Ok(()),
        ControlFlow::Break(()) => Err(Error::Cancelled),
    }
}

fn started(deck: &Deck<'_>, tools: &Tools) -> Result<Browser, Error> {
    let sized = (16..=16384).contains(&deck.width) && (16..=16384).contains(&deck.height);
    if !sized || !deck.bundle.join("index.html").is_file() {
        return Err(Error::Job(format!("{}x{} with an index.html is needed", deck.width, deck.height)));
    }
    let size = (deck.width, deck.height);
    Browser::launch(tools, flags::flags(tools, size), Guard::new(deck.bundle)?)
}

/// Writes scene 0 at its last step to `output` as a JPEG `wide` pixels wide.
pub fn thumbnail(deck: &Deck<'_>, wide: u32, output: &Path, tools: &Tools) -> Result<Report, Error> {
    let mut browser = started(deck, tools)?;
    let mut page = Page::open(&mut browser, (deck.width, deck.height))?;
    let first = page.load()?[0];
    page.step(first.first + first.steps)?;
    std::fs::write(output, page.shot(Some(wide))?)?;
    Ok(Report { made: 1, refused: browser.guard.refused })
}

/// Writes each scene of `scenes` (all of them for `None`) at its last step to
/// `<folder>/<scene>.png`, at the stage size.
pub fn stills(
    deck: &Deck<'_>,
    scenes: Option<&[usize]>,
    folder: &Path,
    tools: &Tools,
    progress: OnProgress,
) -> Result<Report, Error> {
    let mut browser = started(deck, tools)?;
    let mut page = Page::open(&mut browser, (deck.width, deck.height))?;
    let stops = page.load()?;
    let wanted: Vec<usize> = scenes.map_or_else(|| (0..stops.len()).collect(), <[usize]>::to_vec);
    if let Some(missing) = wanted.iter().find(|scene| **scene >= stops.len()) {
        return Err(Error::Job(format!("the deck has no scene {missing}")));
    }
    std::fs::create_dir_all(folder)?;
    for (made, scene) in wanted.iter().enumerate() {
        told(progress, made as u32, wanted.len() as u32)?;
        page.step(stops[*scene].first + stops[*scene].steps)?;
        std::fs::write(folder.join(format!("{scene}.png")), page.shot(None)?)?;
    }
    Ok(Report { made: wanted.len() as u32, refused: browser.guard.refused })
}

fn every_step(page: &mut Page<'_>, stops: &[Stop], progress: OnProgress) -> Result<(Vec<u8>, u32), Error> {
    let last = stops.last().map_or(0, |stop| stop.first + stop.steps);
    let mut sheets = Vec::new();
    for step in 0..=last {
        told(progress, step, last + 1)?;
        page.step(step)?;
        sheets.push(page.pdf(false)?);
    }
    Ok((merge::join(&sheets)?, last + 1))
}

/// Writes the deck to `output` as a PDF whose pages are the stage size and whose text can be
/// selected. One page per scene with every step revealed; with `steps`, one page per step.
pub fn pdf(deck: &Deck<'_>, steps: bool, output: &Path, tools: &Tools, progress: OnProgress) -> Result<Report, Error> {
    let mut browser = started(deck, tools)?;
    let mut page = Page::open(&mut browser, (deck.width, deck.height))?;
    let stops = page.load()?;
    told(progress, 0, stops.len() as u32)?;
    let (bytes, made) = match steps {
        true => every_step(&mut page, &stops, progress)?,
        false => (page.pdf(true)?, stops.len() as u32),
    };
    std::fs::write(output, bytes)?;
    Ok(Report { made, refused: browser.guard.refused })
}
