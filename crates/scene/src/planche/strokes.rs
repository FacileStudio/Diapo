//! Lines, arrows and ink strokes as inline SVG drawn from their points.

use ::planche::{Shape, Value};

use super::values::{css, escape, number, string, style};

fn round(value: f64) -> f64 {
    (value * 100.0).round() / 100.0
}

fn points(shape: &Shape) -> Vec<(f64, f64)> {
    let stride = if shape.kind == "ink" { 3 } else { 2 };
    match shape.fields.get("points") {
        Some(Value::Points(flat)) => flat.chunks_exact(stride).map(|p| (round(p[0]), round(p[1]))).collect(),
        _ => Vec::new(),
    }
}

fn head(at: &[(f64, f64)], width: f64) -> Option<String> {
    let [.., (x1, y1), (x2, y2)] = at else { return None };
    let (angle, length) = ((y2 - y1).atan2(x2 - x1), 4.0 * width + 8.0);
    let corner = |turn: f64| {
        format!("{},{}", round(x2 - length * (angle + turn).cos()), round(y2 - length * (angle + turn).sin()))
    };
    Some(format!("<polygon points=\"{x2},{y2} {} {}\" stroke=\"none\"/>", corner(-0.5), corner(0.5)))
}

/// The box size and the SVG of a `line`, `arrow` or `ink` shape. Without `w` and `h` the box
/// reaches the farthest point.
pub(crate) fn svg(shape: &Shape) -> ((f64, f64), String) {
    let at = points(shape);
    let far = |pick: fn(&(f64, f64)) -> f64| at.iter().map(pick).fold(1.0, f64::max);
    let size = (number(shape, "w").unwrap_or_else(|| far(|p| p.0)), number(shape, "h").unwrap_or_else(|| far(|p| p.1)));
    let style = style(shape);
    let color = escape(&css(style.stroke.as_deref().unwrap_or("currentColor")));
    let width = style.width.unwrap_or(4.0);
    let path: Vec<String> = at.iter().map(|(x, y)| format!("{x} {y}")).collect();
    let pointed = string(shape, "head").unwrap_or(if shape.kind == "arrow" { "arrow" } else { "none" }) != "none";
    let tip = if pointed && shape.kind != "ink" { head(&at, width).unwrap_or_default() } else { String::new() };
    let svg = format!(
        "<svg width=\"100%\" height=\"100%\" fill=\"{color}\" stroke=\"{color}\" stroke-width=\"{width}\" \
         stroke-linecap=\"round\" stroke-linejoin=\"round\" style=\"overflow:visible\"><path d=\"M{}\" \
         fill=\"none\"/>{tip}</svg>",
        path.join(" L")
    );
    (size, svg)
}
