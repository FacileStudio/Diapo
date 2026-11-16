//! Real captures in `chrome-headless-shell`: the thumbnail, the stills, both PDFs, and a page
//! that reaches nothing outside its bundle.
#![cfg(feature = "chrome")]

use std::io::Read;
use std::ops::ControlFlow;
use std::path::Path;

use capture::Tools;
use capture::shoot::{Deck, pdf, stills, thumbnail};

const SKIP: &str = "\n================ SKIPPED: the capture tests need chrome-headless-shell (DIAPO_CHROME, PATH or \
    Playwright's cache) ================\n";

const PNG: &[u8] =
    b"\x89PNG\r\n\x1a\n\0\0\0\rIHDR\0\0\0\x01\0\0\0\x01\x08\x06\0\0\0\x1f\x15\xc4\x89\0\0\0\rIDATx\x9cc\xf8\
\xcf\xc0\xf0\x1f\0\x05\0\x01\xff\x89\x99=\x1d\0\0\0\0IEND\xaeB`\x82";

fn source(port: u16) -> String {
    format!(
        "# Capture deck\n\nOpening words\n\n---\nsteps: 2\n---\n\n## Revenue\n\n- Up twelve {{step=1}}\n- Churn down \
         {{step=2}}\n\n![](assets/dot.png)\n\n---\nlayout: raw\nduration: 2\n---\n\n<div id=\"box\" style=\"position:absolute;\
         inset:0;background:#c00\">Animated</div>\n<script>new Image().src = 'http://127.0.0.1:{port}/leak';\
         fetch('http://127.0.0.1:{port}/fetch').catch(() => 0);</script>\n"
    )
}

fn bundle(dir: &Path, port: u16) {
    let project = scene::parse(&source(port)).unwrap();
    std::fs::create_dir_all(dir.join("assets")).unwrap();
    std::fs::write(dir.join("index.html"), scene::compile(&project, &scene::Options::new())).unwrap();
    std::fs::write(dir.join("assets/dot.png"), PNG).unwrap();
}

fn size(picture: &[u8]) -> (u32, u32) {
    let be = |at: usize| u32::from(u16::from_be_bytes([picture[at], picture[at + 1]]));
    if picture.starts_with(b"\x89PNG") {
        return (be(18), be(22));
    }
    let mut at = 2;
    while !(0xc0..=0xc3).contains(&picture[at + 1]) {
        at += 2 + be(at + 2) as usize;
    }
    (be(at + 7), be(at + 5))
}

fn pages(file: &Path) -> (usize, String, Vec<f32>) {
    let document = lopdf::Document::load(file).unwrap();
    let ids = document.get_pages();
    let numbers: Vec<u32> = ids.keys().copied().collect();
    let first = document.get_object(ids[&1]).unwrap().as_dict().unwrap();
    let media = first.get(b"MediaBox").unwrap().as_array().unwrap();
    let sides = media.iter().map(|side| side.as_float().unwrap()).collect();
    (ids.len(), document.extract_text(&numbers).unwrap_or_default(), sides)
}

#[test]
fn a_deck_gives_a_thumbnail_stills_and_a_pdf_and_reaches_nothing_outside_its_bundle() {
    let Ok(tools) = Tools::locate() else { return eprintln!("{SKIP}") };
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    listener.set_nonblocking(true).unwrap();
    let dir = tempfile::tempdir().unwrap();
    bundle(dir.path(), listener.local_addr().unwrap().port());
    let deck = Deck { bundle: dir.path(), width: 1920, height: 1080 };
    let out = tempfile::tempdir().unwrap();
    let thumb = out.path().join("thumbnail.jpg");
    assert_eq!(thumbnail(&deck, 1280, &thumb, &tools).unwrap().made, 1);
    assert_eq!(size(&std::fs::read(&thumb).unwrap()), (1280, 720));
    let mut seen = Vec::new();
    let mut progress = |done: u32, total: u32| {
        seen.push((done, total));
        ControlFlow::Continue(())
    };
    let shot = stills(&deck, Some(&[2]), &out.path().join("stills"), &tools, &mut progress).unwrap();
    assert!(shot.made == 1 && shot.refused >= 1, "{shot:?}");
    let still = std::fs::read(out.path().join("stills/2.png")).unwrap();
    assert_eq!((size(&still), seen.as_slice()), ((1920, 1080), &[(0, 1)][..]));
    assert!(
        stills(&deck, Some(&[9]), &out.path().join("none"), &tools, &mut |_, _| ControlFlow::Continue(())).is_err()
    );
    let mut buffer = [0u8; 1];
    let reached = listener.accept().is_ok_and(|(mut stream, _)| stream.read(&mut buffer).is_ok());
    assert!(!reached, "the page reached a local listener");
}

#[test]
fn the_pdf_has_a_page_per_scene_or_per_step_at_the_stage_size_with_text() {
    let Ok(tools) = Tools::locate() else { return eprintln!("{SKIP}") };
    let dir = tempfile::tempdir().unwrap();
    bundle(dir.path(), 9);
    let deck = Deck { bundle: dir.path(), width: 1920, height: 1080 };
    let out = tempfile::tempdir().unwrap();
    let mut quiet = |_: u32, _: u32| ControlFlow::Continue(());
    let by_scene = out.path().join("scenes.pdf");
    assert_eq!(pdf(&deck, false, &by_scene, &tools, &mut quiet).unwrap().made, 3);
    let (count, text, media) = pages(&by_scene);
    assert_eq!((count, media.as_slice()), (3, &[0.0, 0.0, 1440.0, 1080.0 * 0.75][..]), "{text}");
    assert!(text.contains("Capture deck") && text.contains("Churn down") && text.contains("Animated"), "{text}");
    let by_step = out.path().join("steps.pdf");
    assert_eq!(pdf(&deck, true, &by_step, &tools, &mut quiet).unwrap().made, 5);
    let (count, text, media) = pages(&by_step);
    assert_eq!((count, media.as_slice()), (5, &[0.0, 0.0, 1440.0, 810.0][..]), "{text}");
    assert!(text.contains("Revenue") && text.matches("Up twelve").count() == 2, "{text}");
    let mut stop = |_: u32, _: u32| ControlFlow::Break(());
    assert!(matches!(pdf(&deck, true, &out.path().join("no.pdf"), &tools, &mut stop), Err(capture::Error::Cancelled)));
}
