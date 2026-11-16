//! The slide master, its one layout and the notes master.

use super::geometry::{Frame, Stage};
use super::palette::Palette;
use super::theme::dark;
use super::xml::{HEAD, NS, TREE};

const TITLE: &str = "<p:nvSpPr><p:cNvPr id=\"2\" name=\"Title 1\"/><p:cNvSpPr><a:spLocks noGrp=\"1\"/></p:cNvSpPr>\
<p:nvPr><p:ph type=\"title\"/></p:nvPr></p:nvSpPr>";

const BODY: &str = "<p:txBody><a:bodyPr/><a:lstStyle/><a:p><a:endParaRPr lang=\"en-US\"/></a:p></p:txBody>";

fn map(palette: &Palette) -> &'static str {
    match dark(palette) {
        true => {
            "<p:clrMap bg1=\"dk1\" tx1=\"lt1\" bg2=\"dk2\" tx2=\"lt2\" accent1=\"accent1\" accent2=\"accent2\" \
                 accent3=\"accent3\" accent4=\"accent4\" accent5=\"accent5\" accent6=\"accent6\" hlink=\"hlink\" \
                 folHlink=\"folHlink\"/>"
        }
        false => {
            "<p:clrMap bg1=\"lt1\" tx1=\"dk1\" bg2=\"lt2\" tx2=\"dk2\" accent1=\"accent1\" accent2=\"accent2\" \
                  accent3=\"accent3\" accent4=\"accent4\" accent5=\"accent5\" accent6=\"accent6\" hlink=\"hlink\" \
                  folHlink=\"folHlink\"/>"
        }
    }
}

fn title(stage: &Stage) -> String {
    let area = stage.content();
    let frame = Frame { height: area.height * 0.2, ..area };
    format!("<p:sp>{TITLE}<p:spPr><a:xfrm>{}</a:xfrm></p:spPr>{BODY}</p:sp>", stage.xfrm(&frame))
}

fn level(name: &str, size: i64) -> String {
    format!(
        "<p:{name}><a:lvl1pPr algn=\"l\"><a:buNone/><a:defRPr sz=\"{size}\"><a:solidFill><a:schemeClr val=\"tx1\"/>\
         </a:solidFill><a:latin typeface=\"+mn-lt\"/></a:defRPr></a:lvl1pPr></p:{name}>"
    )
}

/// The slide master: the background of the scheme, a title placeholder and the text defaults.
pub(crate) fn master(stage: &Stage, palette: &Palette) -> String {
    format!(
        "{HEAD}<p:sldMaster {NS}><p:cSld><p:bg><p:bgRef idx=\"1001\"><a:schemeClr val=\"bg1\"/></p:bgRef></p:bg>\
         <p:spTree>{TREE}{}</p:spTree></p:cSld>{}<p:sldLayoutIdLst><p:sldLayoutId id=\"2147483649\" r:id=\"rId1\"/>\
         </p:sldLayoutIdLst><p:txStyles>{}{}{}</p:txStyles></p:sldMaster>",
        title(stage),
        map(palette),
        level("titleStyle", stage.font(72.0)),
        level("bodyStyle", stage.font(40.0)),
        level("otherStyle", stage.font(40.0))
    )
}

/// The one layout every slide uses: a title, and room for anything.
pub(crate) fn layout(stage: &Stage) -> String {
    format!(
        "{HEAD}<p:sldLayout {NS} type=\"titleOnly\" preserve=\"1\"><p:cSld name=\"Title Only\"><p:spTree>{TREE}{}\
         </p:spTree></p:cSld><p:clrMapOvr><a:masterClrMapping/></p:clrMapOvr></p:sldLayout>",
        title(stage)
    )
}

const NOTES_IMAGE: &str = "<p:sp><p:nvSpPr><p:cNvPr id=\"2\" name=\"Slide Image 1\"/><p:cNvSpPr><a:spLocks noGrp=\"1\" \
noRot=\"1\" noChangeAspect=\"1\"/></p:cNvSpPr><p:nvPr><p:ph type=\"sldImg\" idx=\"2\"/></p:nvPr></p:nvSpPr><p:spPr>\
<a:xfrm><a:off x=\"685800\" y=\"1143000\"/><a:ext cx=\"5486400\" cy=\"3086100\"/></a:xfrm><a:prstGeom prst=\"rect\">\
<a:avLst/></a:prstGeom></p:spPr></p:sp>";

const NOTES_BODY: &str = "<p:sp><p:nvSpPr><p:cNvPr id=\"3\" name=\"Notes 2\"/><p:cNvSpPr><a:spLocks noGrp=\"1\"/>\
</p:cNvSpPr><p:nvPr><p:ph type=\"body\" sz=\"quarter\" idx=\"3\"/></p:nvPr></p:nvSpPr><p:spPr><a:xfrm>\
<a:off x=\"685800\" y=\"4400550\"/><a:ext cx=\"5486400\" cy=\"3600450\"/></a:xfrm><a:prstGeom prst=\"rect\"><a:avLst/>\
</a:prstGeom></p:spPr><p:txBody><a:bodyPr/><a:lstStyle/><a:p><a:endParaRPr lang=\"en-US\"/></a:p></p:txBody></p:sp>";

const NOTES_MAP: &str = "<p:clrMap bg1=\"lt1\" tx1=\"dk1\" bg2=\"lt2\" tx2=\"dk2\" accent1=\"accent1\" accent2=\"accent2\" \
accent3=\"accent3\" accent4=\"accent4\" accent5=\"accent5\" accent6=\"accent6\" hlink=\"hlink\" folHlink=\"folHlink\"/>";

/// The notes master: the slide picture above the note.
pub(crate) fn notes_master() -> String {
    format!(
        "{HEAD}<p:notesMaster {NS}><p:cSld><p:bg><p:bgRef idx=\"1001\"><a:schemeClr val=\"bg1\"/></p:bgRef></p:bg>\
         <p:spTree>{TREE}{NOTES_IMAGE}{NOTES_BODY}</p:spTree></p:cSld>{NOTES_MAP}<p:notesStyle><a:lvl1pPr>\
         <a:defRPr sz=\"1200\"/></a:lvl1pPr></p:notesStyle></p:notesMaster>"
    )
}

/// The notes page of one slide, holding `note`, one paragraph per line.
pub(crate) fn notes(note: &str) -> String {
    let line = |text: &str| {
        format!("<a:p><a:r><a:rPr lang=\"en-US\" dirty=\"0\"/><a:t>{}</a:t></a:r></a:p>", super::xml::escape(text))
    };
    let body: String = note.lines().map(line).collect();
    format!(
        "{HEAD}<p:notes {NS}><p:cSld><p:spTree>{TREE}<p:sp><p:nvSpPr><p:cNvPr id=\"2\" name=\"Slide Image 1\"/>\
         <p:cNvSpPr><a:spLocks noGrp=\"1\" noRot=\"1\" noChangeAspect=\"1\"/></p:cNvSpPr><p:nvPr><p:ph type=\"sldImg\"/>\
         </p:nvPr></p:nvSpPr><p:spPr/></p:sp><p:sp><p:nvSpPr><p:cNvPr id=\"3\" name=\"Notes 2\"/><p:cNvSpPr>\
         <a:spLocks noGrp=\"1\"/></p:cNvSpPr><p:nvPr><p:ph type=\"body\" idx=\"1\"/></p:nvPr></p:nvSpPr><p:spPr/>\
         <p:txBody><a:bodyPr/><a:lstStyle/>{}</p:txBody></p:sp></p:spTree></p:cSld><p:clrMapOvr>\
         <a:masterClrMapping/></p:clrMapOvr></p:notes>",
        if body.is_empty() { "<a:p/>".to_owned() } else { body }
    )
}
