//! Where the blocks of one scene go on its slide.

use super::geometry::{Frame, Stage};
use super::model::{Block, Run};
use super::text::{lines, pixels};
use crate::model::Layout;

const GAP: f64 = 36.0;

/// One shape to draw.
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum Piece<'a> {
    /// The slide title: the scene's first heading.
    Title(&'a Block),
    /// Consecutive text blocks, in one text box.
    Text(Vec<&'a Block>),
    /// A table.
    Table(&'a [Vec<Vec<Run>>]),
    /// A picture, with its natural size in pixels.
    Image { path: &'a str, alt: &'a str, size: (f64, f64) },
}

/// A piece and its place.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Placed<'a> {
    pub frame: Frame,
    pub piece: Piece<'a>,
}

fn pieces<'a>(blocks: &'a [Block], natural: &dyn Fn(&str) -> Option<(f64, f64)>) -> Vec<Piece<'a>> {
    let mut found: Vec<Piece<'a>> = Vec::new();
    for (index, block) in blocks.iter().enumerate() {
        match (block, found.last_mut()) {
            (Block::Heading(..), _) if index == 0 => found.push(Piece::Title(block)),
            (Block::Table(rows), _) => found.push(Piece::Table(rows)),
            (Block::Image { path, alt }, _) => {
                found.extend(natural(path).map(|size| Piece::Image { path, alt, size }));
            }
            (_, Some(Piece::Text(group))) => group.push(block),
            _ => found.push(Piece::Text(vec![block])),
        }
    }
    found
}

fn text_height(blocks: &[&Block], width: f64, cover: bool) -> f64 {
    let height = |block: &&Block| {
        let leading = if matches!(block, Block::Heading(..)) { 1.15 } else { 1.45 };
        pixels(block, cover) * (lines(block, width, cover) * leading + 0.5)
    };
    blocks.iter().map(height).sum()
}

fn natural_size(piece: &Piece<'_>, area: &Frame, cover: bool) -> (f64, f64) {
    match piece {
        Piece::Title(block) => (area.width, text_height(&[block], area.width, cover)),
        Piece::Text(blocks) => (area.width, text_height(blocks, area.width, cover)),
        Piece::Table(rows) => (area.width, rows.len() as f64 * 76.0),
        Piece::Image { size: (width, height), .. } => {
            let scale = (area.width / width).min(area.height / height).min(1.0);
            (width * scale, height * scale)
        }
    }
}

fn stack<'a>(pieces: Vec<Piece<'a>>, area: Frame, layout: Layout, centered: bool) -> Vec<Placed<'a>> {
    let cover = layout == Layout::Cover;
    let sizes: Vec<(f64, f64)> = pieces.iter().map(|piece| natural_size(piece, &area, cover)).collect();
    let gaps = GAP * pieces.len().saturating_sub(1) as f64;
    let total: f64 = sizes.iter().map(|size| size.1).sum::<f64>() + gaps;
    let shrink = ((area.height - gaps) / (total - gaps).max(1.0)).clamp(0.1, 1.0);
    let used = (total - gaps) * shrink + gaps;
    let mut y = match layout {
        Layout::Center => area.y + (area.height - used) / 2.0,
        Layout::Cover => area.y + area.height - used,
        _ => area.y,
    };
    let mut placed = Vec::new();
    for (piece, (width, height)) in pieces.into_iter().zip(sizes) {
        let picture = matches!(piece, Piece::Image { .. });
        let width = if picture { width * shrink } else { width };
        let x = if picture && centered { area.x + (area.width - width) / 2.0 } else { area.x };
        placed.push(Placed { frame: Frame { x, y, width, height: height * shrink }, piece });
        y += height * shrink + GAP;
    }
    placed
}

/// Places `blocks` on a slide of `stage`, as `layout` arranges a scene: stacked from the top,
/// centered, set at the bottom for a cover, or text left and pictures and tables right for a
/// split. `natural` answers the pixel size of a picture the file can embed; others are skipped.
pub(crate) fn place<'a>(
    blocks: &'a [Block],
    layout: Layout,
    stage: &Stage,
    natural: &dyn Fn(&str) -> Option<(f64, f64)>,
) -> Vec<Placed<'a>> {
    let area = stage.content();
    let all = pieces(blocks, natural);
    let figures = all.iter().any(|piece| matches!(piece, Piece::Table(_) | Piece::Image { .. }));
    if layout != Layout::Split || !figures {
        return stack(all, area, layout, layout == Layout::Center);
    }
    let half = (area.width - 2.0 * GAP) / 2.0;
    let (text, shown): (Vec<_>, Vec<_>) =
        all.into_iter().partition(|piece| matches!(piece, Piece::Title(_) | Piece::Text(_)));
    let mut placed = stack(text, Frame { width: half, ..area }, Layout::Default, false);
    placed.extend(stack(shown, Frame { x: area.x + half + 2.0 * GAP, width: half, ..area }, Layout::Center, true));
    placed
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::Size;
    use crate::pptx::read::blocks;

    fn frames(body: &str, layout: Layout) -> Vec<(Frame, bool)> {
        let stage = Stage::of(Size { width: 1920, height: 1080 });
        let read = blocks(body);
        let natural = |_: &str| Some((800.0, 400.0));
        let placed = place(&read, layout, &stage, &natural);
        placed.into_iter().map(|placed| (placed.frame, matches!(placed.piece, Piece::Image { .. }))).collect()
    }

    #[test]
    fn pieces_stack_inside_the_stage_and_a_split_puts_pictures_right() {
        let body = "## Title\n\nText\n\n- a\n- b\n\n![](a.png)\n";
        let stacked = frames(body, Layout::Default);
        assert_eq!(stacked.len(), 3);
        assert!(stacked.windows(2).all(|pair| pair[1].0.y > pair[0].0.y));
        assert!(stacked.iter().all(|(frame, _)| frame.y + frame.height <= 1080.0 && frame.x >= 0.0));
        let split = frames(body, Layout::Split);
        let (picture, _) = split.iter().find(|(_, picture)| *picture).unwrap();
        assert!(picture.x > 960.0 && split[0].0.x < 200.0, "{split:?}");
        let tall = frames(&format!("# T\n\n{}", "- item\n".repeat(60)), Layout::Default);
        assert!(tall.iter().all(|(frame, _)| frame.y + frame.height <= 1080.0 + 1.0), "{tall:?}");
    }
}
