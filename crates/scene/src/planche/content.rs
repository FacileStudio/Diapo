//! What one shape draws inside its box, per type.

use ::planche::{Shape, Value};

use super::boxes::open;
use super::strokes;
use super::values::{css, escape, string, style};

const STICKY: &str = "#fde68a";

fn text(shape: &Shape) -> String {
    match shape.fields.get("text") {
        Some(Value::Rich(rich)) if rich.0.content.iter().all(|n| n.kind == "paragraph" && n.content.is_empty()) => {
            String::new()
        }
        Some(Value::Rich(rich)) => crate::markdown::plain(&encre::markdown::serialize_commonmark(&rich.0)),
        Some(Value::Plain(plain)) => escape(plain).replace('\n', "<br>"),
        _ => String::new(),
    }
}

fn look(shape: &Shape) -> String {
    let style = style(shape);
    let mut out = Vec::new();
    let fill = style.fill.as_deref().or((shape.kind == "sticky").then_some(STICKY));
    match (shape.kind.as_str(), fill) {
        ("text", Some(fill)) => out.push(format!("color:{}", css(fill))),
        (_, Some(fill)) => out.push(format!("background:{}", css(fill))),
        _ => {}
    }
    match shape.kind.as_str() {
        "ellipse" => out.push("border-radius:50%".to_owned()),
        "diamond" => out.push("clip-path:polygon(50% 0,100% 50%,50% 100%,0 50%)".to_owned()),
        _ => {}
    }
    if let Some(stroke) = style.stroke.as_deref().filter(|_| shape.kind != "text") {
        out.push(format!("outline:{}px solid {};outline-offset:0", style.width.unwrap_or(2.0), css(stroke)));
    }
    out.extend(style.font.map(|font| format!("font-family:{}", css(&font))));
    out.extend(style.size.map(|size| format!("font-size:{size}px")));
    out.extend(style.align.map(|align| format!("text-align:{}", css(&align))));
    out.extend(style.valign.as_deref().and_then(placement));
    out.join(";")
}

fn placement(valign: &str) -> Option<String> {
    let justify = match valign {
        "top" => "flex-start",
        "middle" => "center",
        "bottom" => "flex-end",
        _ => return None,
    };
    Some(format!("display:flex;flex-direction:column;justify-content:{justify}"))
}

fn media(shape: &Shape) -> String {
    let src = escape(string(shape, "asset").unwrap_or_default());
    let fit = escape(&css(string(shape, "fit").unwrap_or("contain")));
    match shape.kind.as_str() {
        "image" => format!("<img src=\"{src}\" alt=\"\" style=\"object-fit:{fit}\">"),
        "video" => format!("<video src=\"{src}\" muted playsinline style=\"object-fit:{fit}\"></video>"),
        _ => format!("<audio src=\"{src}\" hidden></audio>"),
    }
}

fn card(shape: &Shape) -> String {
    let field = |key| escape(string(shape, key).unwrap_or_default());
    let title = [field("title"), field("url")].into_iter().find(|text| !text.is_empty()).unwrap_or_default();
    format!(
        "<a href=\"{}\"><strong>{title}</strong><span>{}</span><small>{}</small></a>",
        field("url"),
        field("description"),
        field("site")
    )
}

/// The element of `shape`, box included, or `None` for a type that draws nothing in a frame.
pub(crate) fn element(shape: &Shape) -> Option<String> {
    let html = |key| string(shape, key).unwrap_or_default().to_owned();
    let (open, inner) = match shape.kind.as_str() {
        "sticky" | "text" | "rect" | "ellipse" | "diamond" => (open(shape, None, &look(shape)), text(shape)),
        "image" | "video" => (open(shape, None, ""), media(shape)),
        "audio" => (open(shape, None, "display:none"), media(shape)),
        "html" => (open(shape, None, ""), html("html")),
        "card" => (open(shape, None, ""), card(shape)),
        "line" | "arrow" | "ink" => {
            let (size, svg) = strokes::svg(shape);
            (open(shape, Some(size), ""), svg)
        }
        _ => return None,
    };
    Some(format!("{open}{inner}</div>"))
}
