//! The shapes of a slide: text boxes, tables and pictures.

use super::geometry::{Frame, Stage};
use super::model::{Block, Run};
use super::text::{Style, paragraph};
use super::xml::escape;

/// A text box holding `blocks` in `frame`. `title` makes it the slide's title placeholder.
pub(crate) fn text_box(id: usize, frame: &Frame, blocks: &[&Block], style: &Style<'_>, title: bool) -> String {
    let body: String = blocks.iter().enumerate().map(|(index, block)| paragraph(block, style, index == 0)).collect();
    let (name, holder) = match title {
        true => ("Title", "<p:cNvSpPr><a:spLocks noGrp=\"1\"/></p:cNvSpPr><p:nvPr><p:ph type=\"title\"/></p:nvPr>"),
        false => ("Text", "<p:cNvSpPr txBox=\"1\"/><p:nvPr/>"),
    };
    let anchor = if style.cover { "b" } else { "t" };
    format!(
        "<p:sp><p:nvSpPr><p:cNvPr id=\"{id}\" name=\"{name} {id}\"/>{holder}</p:nvSpPr><p:spPr><a:xfrm>{}</a:xfrm>\
         <a:prstGeom prst=\"rect\"><a:avLst/></a:prstGeom><a:noFill/></p:spPr><p:txBody><a:bodyPr wrap=\"square\" \
         lIns=\"0\" tIns=\"0\" rIns=\"0\" bIns=\"0\" anchor=\"{anchor}\"><a:normAutofit/></a:bodyPr><a:lstStyle/>{}\
         </p:txBody></p:sp>",
        style.stage.xfrm(frame),
        if body.is_empty() { "<a:p/>".to_owned() } else { body }
    )
}

/// A picture in `frame`, showing the media part the slide relates as `relation`.
pub(crate) fn picture(id: usize, frame: &Frame, stage: &Stage, relation: &str, alt: &str) -> String {
    format!(
        "<p:pic><p:nvPicPr><p:cNvPr id=\"{id}\" name=\"Picture {id}\" descr=\"{}\"/><p:cNvPicPr><a:picLocks \
         noChangeAspect=\"1\"/></p:cNvPicPr><p:nvPr/></p:nvPicPr><p:blipFill><a:blip r:embed=\"{relation}\"/>\
         <a:stretch><a:fillRect/></a:stretch></p:blipFill><p:spPr><a:xfrm>{}</a:xfrm><a:prstGeom prst=\"rect\">\
         <a:avLst/></a:prstGeom></p:spPr></p:pic>",
        escape(alt),
        stage.xfrm(frame)
    )
}

fn cell(runs: &[Run], style: &Style<'_>, header: bool) -> String {
    let block = if header { Block::Heading(4, runs.to_vec()) } else { Block::Paragraph(runs.to_vec()) };
    let text = paragraph(&block, &Style { size: Some(34.0), centered: false, ..*style }, true);
    let fill = if header { style.palette.strong } else { style.palette.fill };
    let pad = style.stage.emu(24.0);
    let line = |side: &str| format!("<a:ln{side} w=\"0\"><a:noFill/></a:ln{side}>");
    format!(
        "<a:tc><a:txBody><a:bodyPr/><a:lstStyle/>{text}</a:txBody><a:tcPr marL=\"{pad}\" marR=\"{pad}\" \
         marT=\"{half}\" marB=\"{half}\" anchor=\"ctr\">{}{}{}{}<a:solidFill><a:srgbClr val=\"{fill}\"/></a:solidFill>\
         </a:tcPr></a:tc>",
        line("L"),
        line("R"),
        line("T"),
        line("B"),
        half = pad / 2
    )
}

/// A table in `frame`: equal columns, the first row as the header, fills instead of borders.
pub(crate) fn table(id: usize, frame: &Frame, rows: &[Vec<Vec<Run>>], style: &Style<'_>) -> String {
    let columns = rows.iter().map(Vec::len).max().unwrap_or(1).max(1);
    let width = style.stage.emu(frame.width) / columns as i64;
    let height = style.stage.emu(frame.height) / rows.len().max(1) as i64;
    let grid: String = (0..columns).map(|_| format!("<a:gridCol w=\"{width}\"/>")).collect();
    let row = |(index, row): (usize, &Vec<Vec<Run>>)| {
        let cells: String =
            (0..columns).map(|at| cell(row.get(at).map_or(&[], Vec::as_slice), style, index == 0)).collect();
        format!("<a:tr h=\"{height}\">{cells}</a:tr>")
    };
    let body: String = rows.iter().enumerate().map(row).collect();
    format!(
        "<p:graphicFrame><p:nvGraphicFramePr><p:cNvPr id=\"{id}\" name=\"Table {id}\"/><p:cNvGraphicFramePr>\
         <a:graphicFrameLocks noGrp=\"1\"/></p:cNvGraphicFramePr><p:nvPr/></p:nvGraphicFramePr><p:xfrm>{}</p:xfrm>\
         <a:graphic><a:graphicData uri=\"http://schemas.openxmlformats.org/drawingml/2006/table\"><a:tbl>\
         <a:tblPr firstRow=\"1\"/><a:tblGrid>{grid}</a:tblGrid>{body}</a:tbl></a:graphicData></a:graphic>\
         </p:graphicFrame>",
        style.stage.xfrm(frame)
    )
}
