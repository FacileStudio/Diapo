//! The PowerPoint writer: the package holds together and each scene is real, editable shapes.
#![cfg(feature = "pptx")]

mod support;

use support::{DECK, PNG, files, text, unzip, well_formed};

fn package() -> std::collections::BTreeMap<String, Vec<u8>> {
    let project = scene::parse(DECK).unwrap();
    unzip(&scene::pptx::write(&project, &files).unwrap())
}

#[test]
fn every_part_is_well_formed_declared_and_reachable() {
    let parts = package();
    let types = text(&parts, "[Content_Types].xml");
    for (name, bytes) in parts.iter().filter(|(name, _)| name.ends_with(".xml") || name.ends_with(".rels")) {
        well_formed(name, std::str::from_utf8(bytes).unwrap());
        let declared =
            name.ends_with(".rels") || name.starts_with('[') || types.contains(&format!("PartName=\"/{name}\""));
        assert!(declared, "{name} has no content type");
    }
    for (name, bytes) in parts.iter().filter(|(name, _)| name.ends_with(".rels")) {
        let base = name.split("_rels/").next().unwrap();
        for target in std::str::from_utf8(bytes).unwrap().split("Target=\"").skip(1) {
            let target = target.split('"').next().unwrap();
            let folder = base.trim_end_matches('/').rsplit_once('/').map_or("", |(parent, _)| parent);
            let path = match target.strip_prefix("../") {
                Some(up) => format!("{folder}/{up}").trim_start_matches('/').to_owned(),
                None => format!("{base}{target}"),
            };
            assert!(parts.contains_key(&path), "{name} points at {target}, and {path} is not in the package");
        }
    }
    let presentation = text(&parts, "ppt/presentation.xml");
    assert_eq!(presentation.matches("<p:sldId ").count(), 4);
    assert!(presentation.contains("<p:sldSz cx=\"12192000\" cy=\"6858000\"/>"), "{presentation}");
}

#[test]
fn markdown_scenes_become_titles_leveled_lists_tables_and_pictures() {
    let parts = package();
    let cover = text(&parts, "ppt/slides/slide1.xml");
    assert!(cover.contains("<p:ph type=\"title\"/>") && cover.contains("<a:t>Q3 review</a:t>"), "{cover}");
    assert!(cover.contains("&amp;") && cover.contains("1 &lt; 2</a:t>") && cover.contains("sz=\"6800\""), "{cover}");
    let split = text(&parts, "ppt/slides/slide2.xml");
    assert!(split.contains("<a:t>Revenue</a:t>") && split.contains("<a:t>12%</a:t>"), "{split}");
    assert!(!split.contains("{step="), "{split}");
    assert_eq!(split.matches("<a:buChar ").count(), 3, "{split}");
    assert!(split.contains("lvl=\"1\"") && split.contains("<a:t>In France</a:t>"), "{split}");
    assert!(split.contains("<a:buAutoNum type=\"arabicPeriod\"/>") && split.contains("<a:t>First</a:t>"));
    assert!(split.contains("<a:blip r:embed=\"rId2\"/>") && split.contains("descr=\"Chart\""), "{split}");
    assert!(split.contains("<a:t>[picture: Logo]</a:t>"), "{split}");
    let rels = text(&parts, "ppt/slides/_rels/slide2.xml.rels");
    assert!(rels.contains("Target=\"../media/image1.png\""), "{rels}");
    assert_eq!(parts["ppt/media/image1.png"], PNG);
    let table = text(&parts, "ppt/slides/slide3.xml");
    assert_eq!((table.matches("<a:tr ").count(), table.matches("<a:tc>").count()), (3, 6), "{table}");
    assert!(table.contains("<a:t>Quarter</a:t>") && table.contains("<a:t>12</a:t>"));
    assert!(table.contains("<a:t>Quoted</a:t>") && table.contains("<a:t>let x = 1;</a:t>"), "{table}");
    assert!(table.contains("typeface=\"Courier New\""));
}

#[test]
fn notes_go_to_notes_pages_and_a_raw_scene_says_it_is_animated() {
    let parts = package();
    let cover = text(&parts, "ppt/notesSlides/notesSlide1.xml");
    assert!(cover.contains("<a:t>Thank the team first.</a:t>"), "{cover}");
    assert!(!text(&parts, "ppt/slides/slide1.xml").contains("Thank the team"));
    assert!(!parts.contains_key("ppt/notesSlides/notesSlide2.xml"));
    let raw = text(&parts, "ppt/slides/slide4.xml");
    assert!(raw.contains(&format!("<a:t>{}</a:t>", scene::pptx::RAW_TEXT)), "{raw}");
    assert!(!raw.contains("Launch") && !raw.contains("clip"), "{raw}");
    let note = text(&parts, "ppt/notesSlides/notesSlide4.xml");
    assert!(note.contains(scene::pptx::RAW_TEXT) && note.contains("<a:t>Pause here.</a:t>"), "{note}");
    assert!(text(&parts, "ppt/slides/_rels/slide4.xml.rels").contains("notesSlides/notesSlide4.xml"));
}

#[test]
fn the_slide_follows_the_deck_size_and_scheme() {
    let project = scene::parse("---\nsize: 1024x768\nscheme: dark\n---\n\n# Dark\n").unwrap();
    let parts = unzip(&scene::pptx::write(&project, &files).unwrap());
    assert!(text(&parts, "ppt/presentation.xml").contains("<p:sldSz cx=\"12192000\" cy=\"9144000\"/>"));
    assert!(text(&parts, "ppt/slideMasters/slideMaster1.xml").contains("<p:clrMap bg1=\"dk1\" tx1=\"lt1\""));
    assert!(text(&parts, "ppt/slides/slide1.xml").contains("<a:srgbClr val=\"FAFAFA\"/>"));
}

#[test]
fn libreoffice_opens_the_file_when_it_is_installed() {
    let found = ["soffice", "libreoffice"].into_iter().find(|name| {
        std::process::Command::new(name).arg("--version").output().is_ok_and(|output| output.status.success())
    });
    let Some(office) = found else {
        eprintln!("SKIPPED: no soffice on this machine, the pptx was not opened by LibreOffice");
        return;
    };
    let dir = std::env::temp_dir().join(format!("scene-pptx-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let project = scene::parse(DECK).unwrap();
    std::fs::write(dir.join("deck.pptx"), scene::pptx::write(&project, &files).unwrap()).unwrap();
    let home = format!("-env:UserInstallation=file://{}/profile", dir.display());
    let run = std::process::Command::new(office)
        .args([&home, "--headless", "--convert-to", "pdf", "--outdir"])
        .arg(&dir)
        .arg(dir.join("deck.pptx"))
        .output()
        .unwrap();
    let pdf = std::fs::read(dir.join("deck.pdf")).unwrap_or_default();
    std::fs::remove_dir_all(&dir).ok();
    assert!(run.status.success() && pdf.starts_with(b"%PDF"), "{}", String::from_utf8_lossy(&run.stderr));
}
