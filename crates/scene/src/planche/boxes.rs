//! The positioned box every drawn shape compiles to: its place, turn, opacity and timing.

use std::fmt::Write;

use ::planche::Shape;

use super::values::{escape, number};

fn place(shape: &Shape, size: Option<(f64, f64)>) -> String {
    let at = |key| number(shape, key).unwrap_or_default();
    let mut style = format!("left:{}px;top:{}px", at("x"), at("y"));
    let size = size.or_else(|| Some((number(shape, "w")?, number(shape, "h")?)));
    if let Some((w, h)) = size {
        let _ = write!(style, ";width:{w}px;height:{h}px");
    }
    if let Some(rot) = number(shape, "rot").filter(|rot| *rot != 0.0) {
        let _ = write!(style, ";transform:rotate({rot}rad)");
    }
    if let Some(opacity) = number(shape, "opacity") {
        let _ = write!(style, ";opacity:{opacity}");
    }
    style
}

fn timing(shape: &Shape) -> String {
    let mut out = String::new();
    if let Some(step) = number(shape, "step").filter(|step| *step > 0.0) {
        let _ = write!(out, " data-step=\"{step}\"");
    }
    for key in ["start", "duration"] {
        if let Some(seconds) = number(shape, key) {
            let _ = write!(out, " data-{key}=\"{seconds}\"");
        }
    }
    out
}

/// The opening tag of the box of `shape`: classes `pl pl-<type>`, its place in frame pixels
/// (`size` overrides `w` and `h`), then `extra` declarations, and its step and clip attributes.
pub(crate) fn open(shape: &Shape, size: Option<(f64, f64)>, extra: &str) -> String {
    let mut style = place(shape, size);
    if !extra.is_empty() {
        style = format!("{style};{extra}");
    }
    format!("<div class=\"pl pl-{}\" style=\"{}\"{}>", escape(&shape.kind), escape(&style), timing(shape))
}
