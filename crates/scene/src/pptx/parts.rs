//! The package parts that hold the presentation together.

use super::geometry::{SLIDE_WIDTH, Stage};
use super::xml::{HEAD, NS, rels};

const PML: &str = "application/vnd.openxmlformats-officedocument.presentationml";

fn part(path: &str, kind: &str) -> String {
    format!("<Override PartName=\"{path}\" ContentType=\"{kind}\"/>")
}

/// `[Content_Types].xml` for `slides` slides, of which `noted` (numbers from 1) have a notes page.
pub(crate) fn content_types(slides: usize, noted: &[usize]) -> String {
    let mut out = format!(
        "{HEAD}<Types xmlns=\"http://schemas.openxmlformats.org/package/2006/content-types\"><Default Extension=\"rels\" \
         ContentType=\"application/vnd.openxmlformats-package.relationships+xml\"/><Default Extension=\"xml\" \
         ContentType=\"application/xml\"/><Default Extension=\"png\" ContentType=\"image/png\"/><Default \
         Extension=\"jpeg\" ContentType=\"image/jpeg\"/><Default Extension=\"gif\" ContentType=\"image/gif\"/>"
    );
    out.push_str(&part("/ppt/presentation.xml", &format!("{PML}.presentation.main+xml")));
    out.push_str(&part("/ppt/slideMasters/slideMaster1.xml", &format!("{PML}.slideMaster+xml")));
    out.push_str(&part("/ppt/slideLayouts/slideLayout1.xml", &format!("{PML}.slideLayout+xml")));
    out.push_str(&part("/ppt/notesMasters/notesMaster1.xml", &format!("{PML}.notesMaster+xml")));
    for theme in 1..=2 {
        let kind = "application/vnd.openxmlformats-officedocument.theme+xml";
        out.push_str(&part(&format!("/ppt/theme/theme{theme}.xml"), kind));
    }
    for slide in 1..=slides {
        out.push_str(&part(&format!("/ppt/slides/slide{slide}.xml"), &format!("{PML}.slide+xml")));
    }
    for slide in noted {
        out.push_str(&part(&format!("/ppt/notesSlides/notesSlide{slide}.xml"), &format!("{PML}.notesSlide+xml")));
    }
    out.push_str("</Types>");
    out
}

/// `_rels/.rels`: the package's one root, the presentation.
pub(crate) fn root_rels() -> String {
    rels(&[("rId1".to_owned(), "officeDocument", "ppt/presentation.xml".to_owned())])
}

/// `ppt/presentation.xml`: the masters, the slides in order and the slide size.
pub(crate) fn presentation(slides: usize, stage: &Stage) -> String {
    let ids: String =
        (0..slides).map(|index| format!("<p:sldId id=\"{}\" r:id=\"rId{}\"/>", 256 + index, 4 + index)).collect();
    format!(
        "{HEAD}<p:presentation {NS} saveSubsetFonts=\"1\"><p:sldMasterIdLst><p:sldMasterId id=\"2147483648\" \
         r:id=\"rId1\"/></p:sldMasterIdLst><p:notesMasterIdLst><p:notesMasterId r:id=\"rId2\"/></p:notesMasterIdLst>\
         <p:sldIdLst>{ids}</p:sldIdLst><p:sldSz cx=\"{SLIDE_WIDTH}\" cy=\"{}\"/><p:notesSz cx=\"6858000\" \
         cy=\"9144000\"/></p:presentation>",
        stage.slide_height()
    )
}

/// `ppt/_rels/presentation.xml.rels`.
pub(crate) fn presentation_rels(slides: usize) -> String {
    let mut entries = vec![
        ("rId1".to_owned(), "slideMaster", "slideMasters/slideMaster1.xml".to_owned()),
        ("rId2".to_owned(), "notesMaster", "notesMasters/notesMaster1.xml".to_owned()),
        ("rId3".to_owned(), "theme", "theme/theme1.xml".to_owned()),
    ];
    entries.extend(
        (0..slides).map(|index| (format!("rId{}", 4 + index), "slide", format!("slides/slide{}.xml", index + 1))),
    );
    rels(&entries)
}

/// The fixed relationship parts: master, layout and notes master, as `(path, content)`.
pub(crate) fn fixed_rels() -> [(&'static str, String); 3] {
    let master = [
        ("rId1".to_owned(), "slideLayout", "../slideLayouts/slideLayout1.xml".to_owned()),
        ("rId2".to_owned(), "theme", "../theme/theme1.xml".to_owned()),
    ];
    let layout = [("rId1".to_owned(), "slideMaster", "../slideMasters/slideMaster1.xml".to_owned())];
    let notes = [("rId1".to_owned(), "theme", "../theme/theme2.xml".to_owned())];
    [
        ("ppt/slideMasters/_rels/slideMaster1.xml.rels", rels(&master)),
        ("ppt/slideLayouts/_rels/slideLayout1.xml.rels", rels(&layout)),
        ("ppt/notesMasters/_rels/notesMaster1.xml.rels", rels(&notes)),
    ]
}
