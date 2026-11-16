//! A valid [`Project`] to one self-contained HTML document.

use std::fmt::Write;

use crate::assets::{self, Assets};
use crate::model::{Config, Layout, Project, Scene};

const THEME: &str = include_str!("theme/facile.css");

const RUNTIME: [&str; 5] = [
    include_str!("runtime/ease.js"),
    include_str!("runtime/values.js"),
    include_str!("runtime/timeline.js"),
    include_str!("runtime/stage.js"),
    include_str!("runtime/host.js"),
];

/// How to compile.
#[derive(Clone, Copy, Default)]
#[non_exhaustive]
pub struct Options<'a> {
    /// Where the document loads the project's files from. `None` keeps the paths as written.
    pub assets: Option<&'a dyn Assets>,
}

impl<'a> Options<'a> {
    /// The defaults: asset paths kept as written.
    pub fn new() -> Self {
        Options::default()
    }

    /// Loads the project's files through `assets`.
    pub fn assets(mut self, assets: &'a dyn Assets) -> Self {
        self.assets = Some(assets);
        self
    }
}

fn escape(text: &str) -> String {
    text.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;").replace('"', "&quot;")
}

fn head(config: &Config) -> String {
    let (width, height) = (config.size.width, config.size.height);
    let sized = format!(
        ":root{{--scene-width:{width}px;--scene-height:{height}px}}@media print{{@page{{size:{width}px {height}px;margin:0}}}}"
    );
    format!(
        "<meta charset=\"utf-8\">\n<meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">\n\
         <title>{}</title>\n<style>{sized}\n{THEME}</style>\n<script>(function () {{\n\"use strict\";\n{}}})();</script>\n",
        escape(&config.title),
        RUNTIME.concat()
    )
}

fn section(out: &mut String, scene: &Scene, index: usize, options: &Options<'_>) {
    let html = match options.assets {
        Some(assets) => assets::rewrite(&scene.html, assets),
        None => scene.html.clone(),
    };
    let class = [scene.layout.name(), " ", &scene.class].concat();
    let _ = writeln!(
        out,
        "<section class=\"scene layout-{}\" data-index=\"{index}\" data-steps=\"{}\" data-duration=\"{}\">",
        class.trim_end(),
        scene.steps,
        scene.duration
    );
    match scene.layout {
        Layout::Raw => out.push_str(&html),
        _ => {
            let _ = write!(out, "<div class=\"content\">\n{}</div>", html);
        }
    }
    out.push_str("\n</section>\n");
}

/// The document of `project`: every scene, or the one at `only`.
pub(crate) fn document(project: &Project, only: Option<usize>, options: &Options<'_>) -> String {
    let config = &project.config;
    let mut out = format!(
        "<!doctype html>\n<html lang=\"{}\" data-theme=\"facile\" data-scheme=\"{}\">\n<head>\n{}</head>\n<body>\n\
         <main class=\"stage\" data-width=\"{}\" data-height=\"{}\">\n",
        config.lang,
        config.scheme.name(),
        head(config),
        config.size.width,
        config.size.height
    );
    let shown = project.scenes.iter().enumerate().filter(|(index, _)| only.is_none_or(|one| one == *index));
    for (index, scene) in shown {
        section(&mut out, scene, index, options);
    }
    out.push_str("</main>\n</body>\n</html>\n");
    out
}
