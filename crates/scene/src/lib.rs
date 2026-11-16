//! The scene format shared by Diapo and Clap.
//!
//! A project is one text file: a YAML headmatter, then scenes split by `---` lines. This
//! crate parses and validates that file and compiles it to one self-contained HTML document
//! whose `window.__scene` contract (`duration`, `steps`, `seek(t)`, `step(k)`) is the only
//! thing a preview, a presenter, a PDF capture or a video render drives. The full reference
//! is [`FORMAT_GUIDE`].
//!
//! ```
//! let project = scene::parse("# Hello\n\n- one {step=1}\n").unwrap();
//! assert_eq!(project.scenes()[0].steps, 1);
//! let html = scene::compile(&project, &scene::Options::new());
//! assert!(html.contains("data-step=\"1\""));
//! ```
#![deny(unused, dead_code, missing_docs)]

mod assets;
mod audit;
mod compile;
mod error;
mod frontmatter;
mod html;
mod markdown;
mod marker;
mod model;
mod note;
mod parse;
#[cfg(feature = "pptx")]
pub mod pptx;
mod references;
mod split;

pub use assets::Assets;
pub use compile::Options;
pub use error::{Error, ErrorKind, Errors};
pub use model::{Config, Layout, Project, Scene, Scheme, Size, Theme};

/// The format reference, for people and for agents: the text of `docs/format.md`.
pub const FORMAT_GUIDE: &str = include_str!("../../../docs/format.md");

/// Parses and validates a project source. A [`Project`] is valid by construction; on failure
/// the answer lists every problem, each with its line.
pub fn parse(source: &str) -> Result<Project, Errors> {
    parse::parse(source)
}

/// Compiles the whole project to one self-contained HTML document.
pub fn compile(project: &Project, options: &Options<'_>) -> String {
    compile::document(project, None, options)
}

/// Compiles the scene at `index` (from 0) to its own self-contained HTML document, or `None`
/// when the project has no such scene.
pub fn compile_scene(project: &Project, index: usize, options: &Options<'_>) -> Option<String> {
    project.scenes().get(index).map(|_| compile::document(project, Some(index), options))
}
