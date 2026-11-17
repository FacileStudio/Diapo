//! The planche bridge: the starter deck, compiling every shape type and projection errors. `UPDATE_GOLDEN=1 cargo test -p facile-scene --features planche` rewrites
//! the golden document.
#![cfg(feature = "planche")]

mod support;

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use planche::{Document, Meta, Profile, Rich, Shape, Value};
use scene::planche::{STARTER, convert, is_planche, project};
use support::{body_of, golden};

fn fixtures() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("fixtures")
}

fn shape(kind: &str, fields: &[(&str, Value)]) -> Shape {
    let fields = fields.iter().map(|(key, value)| ((*key).to_owned(), value.clone())).collect();
    Shape { kind: kind.to_owned(), fields }
}

fn starter() -> Document {
    let n = Value::Number;
    let s = |text: &str| Value::Str(text.to_owned());
    let frame = shape(
        "frame",
        &[
            ("x", n(0.0)),
            ("y", n(0.0)),
            ("w", n(1920.0)),
            ("h", n(1080.0)),
            ("name", s("New deck")),
            ("order", s("a0")),
        ],
    );
    let mut title_text = encre::markdown::parse("# New deck").unwrap();
    encre::assign_ids(&mut title_text);
    let title = shape(
        "text",
        &[
            ("x", n(160.0)),
            ("y", n(420.0)),
            ("w", n(1600.0)),
            ("h", n(240.0)),
            ("z", s("a0")),
            ("parent", s("slide1")),
            ("text", Value::Rich(Rich(title_text))),
        ],
    );
    let shapes = BTreeMap::from([("slide1".to_owned(), frame), ("title1".to_owned(), title)]);
    Document { meta: Meta { schema: 1, size: Some((1920, 1080)) }, shapes }
}

#[test]
fn the_starter_is_the_projection_of_one_title_slide_and_compiles() {
    let document = starter();
    planche::yjs::validate(&document, Profile::Full).unwrap();
    assert_eq!(planche::projection::write(&document), STARTER);
    assert!(is_planche(STARTER));
    let compiled = project(STARTER).unwrap();
    assert_eq!(compiled.scenes().len(), 1);
    assert_eq!(compiled.config().title, "New deck");
    let html = scene::compile(&compiled, &scene::Options::new());
    assert!(html.contains("<h1>New deck</h1>"), "{html}");
}

#[test]
fn every_shape_type_compiles_to_its_golden_document() {
    let dir = fixtures().join("planche");
    let source = std::fs::read_to_string(dir.join("deck.md")).unwrap();
    let document = convert(&source).unwrap();
    assert_eq!(planche::projection::write(&document), source, "the fixture is a canonical projection");
    let compiled = project(&source).unwrap();
    let durations: Vec<f64> = compiled.scenes().iter().map(|scene| scene.duration).collect();
    let steps: Vec<u32> = compiled.scenes().iter().map(|scene| scene.steps).collect();
    assert_eq!((durations, steps), (vec![12.0, 4.0], vec![2, 0]));
    let note = compiled.scenes()[0].note.as_deref();
    assert_eq!(note, Some("Start with the **title**, then reveal the rest."));
    let html = body_of(&scene::compile(&compiled, &scene::Options::new()));
    for expected in [
        "class=\"pl pl-sticky\" style=\"left:80px;top:240px;width:240px;height:200px;transform:rotate(0.1rad)",
        "data-step=\"2\"",
        "data-start=\"2\" data-duration=\"4\" data-track-index=\"1\"><video src=\"assets/clip.mp4\" data-in=\"1.5\" data-volume=\"0.8\" muted playsinline",
        "<audio src=\"assets/music.mp3\" data-volume=\"0.4\" hidden>",
        "Every <em>shape</em> {step=9}",
        "<div class=\"badge\">Live <em>now</em></div>\n    ---\n<p>A rule above</p></div>",
        "<a href=\"https://example.com\">",
    ] {
        assert!(html.contains(expected), "missing {expected} in\n{html}");
    }
    assert!(!html.contains("not on a slide"));
    assert!(html.find("pl pl-card").unwrap() < html.find("pl pl-video").unwrap(), "track 1 stacks above track 0");
    golden(&dir.join("expected.html"), &html);
}

#[test]
fn the_guide_example_compiles() {
    let open = scene::planche::GUIDE.find("`````text\n").unwrap() + 10;
    let close = open + scene::planche::GUIDE[open..].find("`````").unwrap();
    let example = &scene::planche::GUIDE[open..close];
    planche::yjs::validate(&convert(example).unwrap(), Profile::Full).unwrap();
    let compiled = project(example).unwrap();
    let steps: Vec<u32> = compiled.scenes().iter().map(|scene| scene.steps).collect();
    assert_eq!((steps, compiled.scenes()[0].note.as_deref()), (vec![2, 0], Some("Thank the team first.")));
}

#[test]
fn a_projection_error_reports_its_line() {
    let source = "---\nplanche: 1\nsize: 1920x1080\n---\n\n```rect id=\"a\" x=\"0\" y=\"0\" colour=\"red\"\n```\n";
    let errors = convert(source).unwrap_err();
    assert_eq!(errors.0.len(), 1);
    assert_eq!(errors.0[0].line, 6, "{errors}");
    assert_eq!(project(source).unwrap_err(), errors);
}

#[test]
fn text_alignment_compiles_to_text_align_and_a_flex_column() {
    let source = "---\nplanche: 1\nsize: 1920x1080\n---\n\n```frame id=\"s\" x=\"0\" y=\"0\" w=\"1920\" h=\"1080\" \
                  order=\"a0\"\n```\n\n```text id=\"a\" x=\"0\" y=\"0\" w=\"800\" h=\"400\" z=\"a0\" parent=\"s\" \
                  style.align=\"justify\" style.valign=\"bottom\"\nHello\n```\n\n```rect id=\"b\" x=\"0\" y=\"500\" \
                  w=\"800\" h=\"400\" z=\"a1\" parent=\"s\" style.valign=\"top\"\nTop\n```\n";
    let compiled = project(source).unwrap();
    let html = body_of(&scene::compile(&compiled, &scene::Options::new()));
    let column = "display:flex;flex-direction:column;justify-content:";
    assert!(html.contains(&format!("text-align:justify;{column}flex-end")), "{html}");
    assert!(html.contains(&format!("{column}flex-start")), "{html}");
}
