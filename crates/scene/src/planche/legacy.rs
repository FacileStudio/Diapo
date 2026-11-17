//! A legacy scene project to a planche document, once: one frame per scene, left to right.

use std::collections::BTreeMap;

use ::planche::ops::fractional::keys_between;
use ::planche::{Document, Meta, Rich, SCHEMA, Shape, Value};

use super::pieces;
use crate::audit::Audit;
use crate::model::{Layout, Scene, Size};
use crate::pptx::geometry::Frame;
use crate::pptx::model::{Block, plain};
use crate::pptx::read::blocks;

const GAP: f64 = 200.0;

/// A shape of `kind` at `frame`, holding `fields` too.
pub(crate) fn boxed(kind: &str, frame: Frame, fields: Vec<(&str, Value)>) -> Shape {
    let place = [("x", frame.x), ("y", frame.y), ("w", frame.width), ("h", frame.height)];
    let place = place.into_iter().map(|(key, n)| (key, Value::Number((n * 100.0).round() / 100.0)));
    let fields = place.chain(fields).map(|(key, value)| (key.to_owned(), value)).collect();
    Shape { kind: kind.to_owned(), fields }
}

/// An encre document of `markdown`, or one paragraph holding it verbatim when encre refuses it.
pub(crate) fn rich(markdown: &str) -> Value {
    let parsed = encre::markdown::parse(markdown).ok().filter(|doc| encre::validate(doc).is_ok());
    let mut doc = parsed.unwrap_or_else(|| {
        let paragraph = encre::Node::new("paragraph", Default::default(), vec![encre::Node::text(markdown, vec![])]);
        encre::Node::new("doc", Default::default(), vec![paragraph])
    });
    encre::assign_ids(&mut doc);
    Value::Rich(Rich(doc))
}

fn stepped(html: String, steps: u32) -> String {
    let mut audit = Audit::default();
    audit.html(&html, 0);
    match steps > audit.steps {
        true => format!("{html}\n<span data-step=\"{steps}\" hidden></span>"),
        false => html,
    }
}

fn wrapped(scene: &Scene) -> String {
    let html = match scene.layout {
        Layout::Raw => scene.html.clone(),
        layout => format!(
            "<div class=\"{}\" style=\"height:100%\"><div class=\"content\">\n{}</div></div>",
            format!("layout-{} {}", layout.name(), scene.class).trim_end(),
            scene.html
        ),
    };
    stepped(html, scene.steps)
}

fn children(scene: &Scene, area: Frame) -> Vec<Shape> {
    match pieces::editable(scene) {
        true => pieces::shapes(scene, area),
        false => vec![boxed("html", area, vec![("locked", Value::Bool(true)), ("html", Value::Str(wrapped(scene)))])],
    }
}

fn frame(scene: &Scene, index: usize, area: Frame, order: String) -> Shape {
    let read = blocks(&scene.body);
    let heading = read.iter().find_map(|block| match block {
        Block::Heading(_, runs) => Some(plain(runs)).filter(|text| !text.trim().is_empty()),
        _ => None,
    });
    let name = heading.unwrap_or_else(|| format!("Slide {}", index + 1));
    let at = Frame { x: index as f64 * (area.width + GAP), ..area };
    let mut fields = vec![("name", Value::Str(name)), ("order", Value::Str(order))];
    fields.extend(scene.note.as_deref().filter(|note| !note.is_empty()).map(|note| ("notes", rich(note))));
    if scene.duration != f64::from(scene.steps + 1) * 4.0 {
        fields.extend([("duration", Value::Number(scene.duration)), ("fixed_duration", Value::Bool(true))]);
    }
    boxed("frame", at, fields)
}

fn adopt(shapes: &mut BTreeMap<String, Shape>, frame: &str, inside: Vec<Shape>) {
    let keys = keys_between(None, None, inside.len());
    for (k, (mut shape, z)) in inside.into_iter().zip(keys).enumerate() {
        shape.fields.insert("parent".to_owned(), Value::Str(frame.to_owned()));
        shape.fields.insert("z".to_owned(), Value::Str(z));
        shapes.insert(format!("{frame}-{}", k + 1), shape);
    }
}

/// The planche document of `project`.
pub(crate) fn document(project: &crate::model::Project) -> Document {
    let Size { width, height } = project.config().size;
    let area = Frame { x: 0.0, y: 0.0, width: f64::from(width), height: f64::from(height) };
    let mut shapes = BTreeMap::new();
    let scenes = project.scenes();
    for (index, (scene, order)) in scenes.iter().zip(keys_between(None, None, scenes.len())).enumerate() {
        let id = format!("slide{}", index + 1);
        adopt(&mut shapes, &id, children(scene, area));
        shapes.insert(id, frame(scene, index, area, order));
    }
    Document { meta: Meta { schema: SCHEMA, size: Some((width, height)) }, shapes }
}
