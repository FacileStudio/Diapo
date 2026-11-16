//! Text blocks as DrawingML paragraphs.

use super::geometry::Stage;
use super::model::{Block, Run, plain};
use super::palette::Palette;
use super::xml::escape;

/// How the paragraphs of one text box are drawn.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Style<'a> {
    pub stage: &'a Stage,
    pub palette: &'a Palette,
    /// Centered, as the `center` layout sets its text.
    pub centered: bool,
    /// The cover layout: a larger title and muted paragraphs.
    pub cover: bool,
    /// A font size in stage pixels for every paragraph, instead of each block's own.
    pub size: Option<f64>,
}

/// The font size of `block` in stage pixels, as the theme's stylesheet sets it.
pub(crate) fn pixels(block: &Block, cover: bool) -> f64 {
    match block {
        Block::Heading(1, _) if cover => 136.0,
        Block::Heading(1, _) => 104.0,
        Block::Heading(2, _) => 72.0,
        Block::Heading(3, _) => 52.0,
        Block::Paragraph(_) if cover => 48.0,
        Block::Code(_) => 30.0,
        Block::Table(_) => 34.0,
        _ => 40.0,
    }
}

fn run(run: &Run, size: i64, bold: bool, color: &str) -> String {
    let weight = if bold || run.bold { " b=\"1\"" } else { "" };
    let slant = if run.italic { " i=\"1\"" } else { "" };
    let face = if run.code { "<a:latin typeface=\"Courier New\"/><a:cs typeface=\"Courier New\"/>" } else { "" };
    format!(
        "<a:r><a:rPr lang=\"en-US\" sz=\"{size}\"{weight}{slant} dirty=\"0\"><a:solidFill><a:srgbClr val=\"{color}\"/>\
         </a:solidFill>{face}</a:rPr><a:t>{}</a:t></a:r>",
        escape(&run.text)
    )
}

fn bullet(block: &Block, stage: &Stage) -> (String, &'static str) {
    let Block::Item { level, ordered, .. } = block else { return (String::new(), "<a:buNone/>") };
    let hang = stage.emu(44.0);
    let margin = hang * (i64::from(*level) + 1);
    let mark = match ordered {
        true => "<a:buFont typeface=\"+mj-lt\"/><a:buAutoNum type=\"arabicPeriod\"/>",
        false => "<a:buFont typeface=\"Arial\"/><a:buChar char=\"&#8226;\"/>",
    };
    (format!(" marL=\"{margin}\" indent=\"-{hang}\" lvl=\"{level}\""), mark)
}

fn properties(block: &Block, style: &Style<'_>, first: bool) -> String {
    let align = if style.centered { "ctr" } else { "l" };
    let before = if first { 0 } else { style.stage.font(pixels(block, style.cover)) / 2 };
    let (list, mark) = bullet(block, style.stage);
    format!("<a:pPr{list} algn=\"{align}\"><a:spcBef><a:spcPts val=\"{before}\"/></a:spcBef>{mark}</a:pPr>")
}

fn runs_of(block: &Block) -> Vec<Run> {
    match block {
        Block::Heading(_, runs) | Block::Paragraph(runs) | Block::Quote(runs) | Block::Item { runs, .. } => {
            runs.clone()
        }
        Block::Code(code) => vec![Run { text: code.trim_end().to_owned(), code: true, ..Run::default() }],
        Block::Table(_) | Block::Image { .. } => Vec::new(),
    }
}

/// `<a:p>` elements for one text block: one paragraph, or one per line of a code block.
pub(crate) fn paragraph(block: &Block, style: &Style<'_>, first: bool) -> String {
    let size = style.stage.font(style.size.unwrap_or_else(|| pixels(block, style.cover)));
    let muted = matches!(block, Block::Quote(_)) || (style.cover && matches!(block, Block::Paragraph(_)));
    let color = if muted { style.palette.muted } else { style.palette.fg };
    let bold = matches!(block, Block::Heading(..));
    let properties = properties(block, style, first);
    let Block::Code(code) = block else {
        let runs: String = runs_of(block).iter().map(|part| run(part, size, bold, color)).collect();
        return format!("<a:p>{properties}{runs}</a:p>");
    };
    let line = |text: &str| run(&Run { text: text.to_owned(), code: true, ..Run::default() }, size, false, color);
    code.trim_end().lines().map(|text| format!("<a:p>{properties}{}</a:p>", line(text))).collect()
}

/// How many lines `block` takes in a box `width` stage pixels wide: an estimate, half an em a
/// character.
pub(crate) fn lines(block: &Block, width: f64, cover: bool) -> f64 {
    let size = pixels(block, cover);
    let wrapped = |text: &str| (text.chars().count() as f64 * size * 0.5 / width.max(1.0)).ceil().max(1.0);
    match block {
        Block::Code(code) => code.trim_end().lines().count().max(1) as f64,
        other => wrapped(&plain(&runs_of(other))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::Size;

    #[test]
    fn items_carry_their_level_and_bullet_and_text_is_escaped() {
        let (stage, palette) =
            (Stage::of(Size { width: 1920, height: 1080 }), super::super::palette::of(crate::Scheme::Light));
        let style = Style { stage: &stage, palette: &palette, centered: false, cover: false, size: None };
        let runs = vec![Run { text: "Up <12%>".into(), bold: true, ..Run::default() }];
        let item = paragraph(&Block::Item { level: 1, ordered: false, runs: runs.clone() }, &style, true);
        assert!(item.contains("lvl=\"1\"") && item.contains("<a:buChar char=\"&#8226;\"/>"), "{item}");
        assert!(item.contains("<a:t>Up &lt;12%&gt;</a:t>") && item.contains(" b=\"1\""), "{item}");
        let numbered = paragraph(&Block::Item { level: 0, ordered: true, runs }, &style, false);
        assert!(numbered.contains("<a:buAutoNum type=\"arabicPeriod\"/>"), "{numbered}");
        let heading = paragraph(&Block::Heading(2, vec![Run { text: "T".into(), ..Run::default() }]), &style, true);
        assert!(heading.contains("sz=\"3600\"") && heading.contains("<a:buNone/>"), "{heading}");
        let code = paragraph(&Block::Code("a\nb\n".into()), &style, true);
        assert_eq!(code.matches("<a:p>").count(), 2);
    }
}
