//! The compiler: what the document holds and how assets are loaded.

use scene::{Options, compile, compile_scene, parse};

#[test]
fn the_document_is_self_contained() {
    let html = compile(&parse("---\ntitle: A <b> & \"c\"\n---\n# Hi\n").unwrap(), &Options::new());
    assert!(html.starts_with("<!doctype html>\n<html lang=\"en\" data-theme=\"facile\" data-scheme=\"light\">"));
    assert!(html.contains("<title>A &lt;b&gt; &amp; &quot;c&quot;</title>"));
    assert!(html.contains("window.__scene = api") && html.contains("--scene-width:1920px"));
    assert!(!html.contains("http://") && !html.contains("https://") && !html.contains("@import"));
    assert_eq!(html.matches("</script>").count(), 1);
    assert_eq!(html.matches("<script").count(), 1);
}

#[test]
fn the_runtime_reads_no_clock_no_randomness_and_no_network() {
    let html = compile(&parse("# Hi\n").unwrap(), &Options::new());
    for banned in ["Date.", "Math.random", "performance.", "fetch(", "XMLHttpRequest", "setTimeout", "setInterval"] {
        assert!(!html.contains(banned), "the runtime uses {banned}");
    }
    assert!(!html.contains("requestAnimationFrame"));
}

#[test]
fn one_scene_compiles_alone_with_its_own_index() {
    let project = parse("# One\n\n---\n\n# Two {step=1}\n").unwrap();
    let html = compile_scene(&project, 1, &Options::new()).unwrap();
    assert!(html.contains("data-index=\"1\" data-steps=\"1\"") && !html.contains("One"));
    assert!(compile_scene(&project, 2, &Options::new()).is_none());
}

#[test]
fn assets_are_rewritten_wherever_a_file_is_loaded() {
    let source = "![c](assets/c.png) [doc](notes.pdf)\n\n<video src='v.mp4' poster=\"p.jpg\"></video>\n\n\
                  <div style=\"background:url(bg.png)\" data-x=\"assets/c.png\"></div>\n\n\
                  <img src=\"data:image/gif;base64,AA\"> <img srcset=\"a.png 1x, b.png 2x\"> <a href=\"#top\">top</a>\n";
    let project = parse(source).unwrap();
    let assets = |path: &str| (path != "p.jpg").then(|| format!("/api/decks/7/assets/{path}?v=1&x=\"y\""));
    let html = compile(&project, &Options::new().assets(&assets));
    for path in ["assets/c.png", "v.mp4", "bg.png", "a.png", "b.png"] {
        assert!(html.contains(&format!("/api/decks/7/assets/{path}?v=1&amp;x=&quot;y&quot;")), "{path} in {html}");
    }
    assert!(html.contains("poster=\"p.jpg\"") && html.contains("href=\"notes.pdf\""));
    assert!(html.contains("data-x=\"assets/c.png\"") && html.contains("data:image/gif;base64,AA"));
    assert!(html.contains("href=\"#top\""));
}

#[test]
fn layouts_and_classes_land_on_the_scene_element() {
    let project =
        parse("---\nlayout: split\nclass: wide dark-1\n---\n# A\n\n---\nlayout: raw\n---\n<b>x</b>\n").unwrap();
    let html = compile(&project, &Options::new());
    assert!(html.contains("<section class=\"scene layout-split wide dark-1\" data-index=\"0\""));
    assert!(html.contains(
        "<section class=\"scene layout-raw\" data-index=\"1\" data-steps=\"0\" data-duration=\"4\">\n<b>x</b>"
    ));
}

#[test]
fn the_format_guide_ships_with_the_crate() {
    assert!(scene::FORMAT_GUIDE.starts_with("# The scene format"));
    assert!(scene::FORMAT_GUIDE.contains("window.__scene"));
    let example = scene::FORMAT_GUIDE.split("````md\n").nth(1).unwrap().split("````").next().unwrap();
    let project = parse(example).unwrap();
    assert_eq!((project.scenes().len(), project.steps(), project.duration()), (3, 4, 22.0));
}
