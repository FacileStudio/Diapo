//! Legacy scene sources converted to planche documents once: valid under the full profile,
//! with the same scenes, steps, durations, notes and, for HTML scenes, the same body.
#![cfg(feature = "planche")]

use std::path::{Path, PathBuf};

use planche::{Document, Profile, Value};
use scene::planche::{convert, from_project, is_planche, project, to_source};

fn fixtures() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("fixtures")
}

fn inner(html: &str) -> &str {
    let open = html.find("<section").unwrap();
    let start = open + html[open..].find(">\n").unwrap() + 2;
    let end = html.rfind("\n</section>").unwrap();
    html[start..end].trim_start_matches("<div class=\"content\">\n").trim_end_matches("</div>")
}

fn same_scenes(before: &scene::Project, document: &Document) {
    let after = scene::parse(&to_source(document)).unwrap();
    assert_eq!(after.scenes().len(), before.scenes().len());
    let options = scene::Options::new();
    for (index, (old, new)) in before.scenes().iter().zip(after.scenes()).enumerate() {
        assert_eq!((old.steps, old.duration, &old.note), (new.steps, new.duration, &new.note), "scene {index}");
        let id = format!("slide{}-1", index + 1);
        if document.shapes.get(&id).is_some_and(|shape| shape.kind == "html") {
            let was = scene::compile_scene(before, index, &options).unwrap();
            let now = scene::compile_scene(&after, index, &options).unwrap();
            assert!(now.contains(inner(&was)), "scene {index} lost its body:\n{}", inner(&was));
        }
    }
}

#[test]
fn legacy_decks_convert_to_valid_documents_that_keep_their_scenes() {
    for (folder, entry) in [("basic", "diapo.md"), ("steps", "diapo.md"), ("raw", "clap.md")] {
        let source = std::fs::read_to_string(fixtures().join(folder).join(entry)).unwrap();
        assert!(!is_planche(&source));
        let before = scene::parse(&source).unwrap();
        let document = from_project(&before);
        planche::yjs::validate(&document, Profile::Full).unwrap_or_else(|e| panic!("{folder}: {e}"));
        assert_eq!(convert(&source).unwrap(), document);
        assert_eq!(project(&source).unwrap(), before);
        same_scenes(&before, &document);
        let projection = planche::projection::write(&document);
        assert_eq!(convert(&projection).unwrap(), document, "{folder}");
    }
}

#[test]
fn editable_scenes_become_text_and_image_shapes() {
    let source = std::fs::read_to_string(fixtures().join("basic/diapo.md")).unwrap();
    let document = from_project(&scene::parse(&source).unwrap());
    let kinds = |frame: &str| -> Vec<&str> {
        let parent = Some(Value::Str(frame.to_owned()));
        document
            .shapes
            .values()
            .filter(|s| s.fields.get("parent") == parent.as_ref())
            .map(|s| s.kind.as_str())
            .collect()
    };
    assert_eq!(kinds("slide1"), ["text", "text"]);
    assert_eq!(kinds("slide3"), ["text", "text", "image"]);
    assert_eq!(kinds("slide4"), ["html"]);
    let Some(Value::Str(name)) = document.shapes["slide2"].fields.get("name") else { panic!() };
    assert_eq!(name, "Three things to remember");
    let projection = planche::projection::write(&document);
    assert!(projection.contains("- Revenue is up **12%**"), "{projection}");
    assert!(projection.contains("```notes\nPresenter note: thank the team first.\n```"), "{projection}");
}
