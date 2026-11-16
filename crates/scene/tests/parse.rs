//! The parser: how a source splits into scenes and what each scene reads from it.

use scene::{Layout, Scheme, Size, parse};

#[test]
fn a_file_with_no_headmatter_is_scenes_with_defaults() {
    let project = parse("# One\n\n---\n\n# Two\n").unwrap();
    assert_eq!(project.scenes().len(), 2);
    assert!(parse("title: prose, since no `---` opens the file\n").unwrap().config().title.is_empty());
    let config = project.config();
    assert_eq!((config.title.as_str(), config.size), ("", Size { width: 1920, height: 1080 }));
    assert_eq!((config.scheme, config.step_duration, config.lang.as_str()), (Scheme::Light, 4.0, "en"));
    assert_eq!((project.scenes()[0].line, project.scenes()[1].line), (1, 4));
}

#[test]
fn the_headmatter_also_configures_the_first_scene() {
    let source = "---\ntitle: \"Deck: one\"\nsize: 1280x720 # hd\nscheme: dark\nlayout: cover\n---\n# Hi\n";
    let project = parse(source).unwrap();
    assert_eq!(project.config().title, "Deck: one");
    assert_eq!(project.config().size, Size { width: 1280, height: 720 });
    assert_eq!(project.config().scheme, Scheme::Dark);
    assert_eq!(project.scenes()[0].layout, Layout::Cover);
    assert_eq!(project.scenes()[0].line, 7);
}

#[test]
fn a_separator_inside_a_fence_does_not_split() {
    let project = parse("```md\n---\nlayout: raw\n---\n```\n\n~~~\n---\n~~~\n").unwrap();
    assert_eq!(project.scenes().len(), 1);
    assert_eq!(project.scenes()[0].layout, Layout::Default);
}

#[test]
fn a_blank_line_after_a_separator_keeps_key_like_text_in_the_body() {
    let project = parse("# One\n\n---\n\nnote: this is prose\n\n---\n---\n# Three\n\n---\n").unwrap();
    assert_eq!(project.scenes().len(), 3);
    assert!(project.scenes()[1].body.contains("note: this is prose"));
    assert_eq!(project.scenes()[2].body.trim(), "# Three");
}

#[test]
fn the_trailing_comment_is_the_note_and_leaves_the_body() {
    let project =
        parse("# One\n\n<!-- inline --> stays\n\n<!--\nSay hello.\nThen stop.\n-->\n\n---\n\n# Two\n").unwrap();
    let first = &project.scenes()[0];
    assert_eq!(first.note.as_deref(), Some("Say hello.\nThen stop."));
    assert!(first.body.contains("<!-- inline --> stays") && !first.body.contains("Say hello"));
    assert_eq!(project.scenes()[1].note, None);
    assert!(!scene::compile(&project, &scene::Options::new()).contains("Say hello"));
}

#[test]
fn steps_come_from_the_frontmatter_or_the_highest_marker() {
    let project = parse("- a {step=1}\n- b {step=3}\n\n---\nsteps: 5\n---\n- c {step=2}\n\n---\n\nplain\n").unwrap();
    let steps: Vec<u32> = project.scenes().iter().map(|scene| scene.steps).collect();
    assert_eq!(steps, [3, 5, 0]);
    assert_eq!(project.steps(), 4 + 6 + 1 - 1);
}

#[test]
fn duration_is_explicit_or_the_last_clip_or_the_steps() {
    let source = "---\nstep_duration: 2\nduration: 9\n---\n- a {step=1}\n\n---\nlayout: raw\n---\n\
                  <i data-start=\"1.5\" data-duration=\"2\"></i>\n\n---\n\n- b {step=2}\n";
    let project = parse(source).unwrap();
    let durations: Vec<f64> = project.scenes().iter().map(|scene| scene.duration).collect();
    assert_eq!(durations, [9.0, 3.5, 6.0]);
    assert_eq!(project.duration(), 18.5);
}

#[test]
fn windows_line_endings_and_a_byte_order_mark_parse_the_same() {
    let project = parse("\u{feff}---\r\ntitle: Win\r\n---\r\n# One\r\n\r\n---\r\n\r\n# Two\r\n").unwrap();
    assert_eq!((project.config().title.as_str(), project.scenes().len()), ("Win", 2));
}
