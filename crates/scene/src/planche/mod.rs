//! The bridge between the scene format and planche documents (`docs/planche.md`).
//!
//! A deck is stored as a planche `full` document and written as its text projection. Every
//! top-level frame is one slide, compiled as one `layout: raw` scene whose shapes are
//! absolutely positioned elements. A legacy scene source is still read, and converted once.
//!
//! ```
//! let project = scene::planche::project(scene::planche::STARTER).unwrap();
//! assert_eq!(project.scenes().len(), 1);
//! ```

mod boxes;
mod content;
mod legacy;
mod pieces;
mod source;
mod strokes;
mod values;

use ::planche::{Document, Profile, projection};

use crate::error::{Error, ErrorKind, Errors};
use crate::model::Project;

/// The projection of a new deck: one slide holding a large title.
pub const STARTER: &str = "---\nplanche: 1\nsize: 1920x1080\n---\n\n\
````frame id=\"slide1\" x=\"0\" y=\"0\" w=\"1920\" h=\"1080\" name=\"New deck\" order=\"a0\"\n\
```text id=\"title1\" x=\"160\" y=\"420\" w=\"1600\" h=\"240\" z=\"a0\"\n\
# New deck\n\
```\n\
````\n";

/// The planche guide, for people and for agents: the text of `docs/planche.md`.
pub const GUIDE: &str = include_str!("../../../../docs/planche.md");

/// Whether `source` is a planche projection rather than a legacy scene source.
pub fn is_planche(source: &str) -> bool {
    source.trim_start_matches('\u{feff}').starts_with("---\nplanche:")
}

/// Either format to a scene [`Project`]: a projection is read, compiled to scene source and
/// parsed; a legacy source is parsed.
pub fn project(source: &str) -> Result<Project, Errors> {
    match is_planche(source) {
        true => crate::parse(&to_source(&read(source)?)),
        false => crate::parse(source),
    }
}

/// Either format to a planche document: a projection is read, a legacy source is parsed and
/// converted with [`from_project`].
pub fn convert(source: &str) -> Result<Document, Errors> {
    match is_planche(source) {
        true => read(source),
        false => crate::parse(source).map(|project| from_project(&project)),
    }
}

/// The scene source of `document`: one `layout: raw` scene per top-level frame, by `order`.
pub fn to_source(document: &Document) -> String {
    source::write(document)
}

/// The planche document of a legacy `project`, one frame per scene.
pub fn from_project(project: &Project) -> Document {
    legacy::document(project)
}

fn read(source: &str) -> Result<Document, Errors> {
    let text = source.trim_start_matches('\u{feff}');
    projection::read(text, Profile::Full).map_err(|error| {
        Errors(vec![Error { line: error.line, kind: ErrorKind::Projection { message: error.message } }])
    })
}
