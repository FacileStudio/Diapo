//! Reading shape keys, and escaping what goes into the generated source.

use ::planche::{Shape, Style, Value};

/// The number `key` of `shape`, when it holds one.
pub(crate) fn number(shape: &Shape, key: &str) -> Option<f64> {
    match shape.fields.get(key) {
        Some(Value::Number(n)) => Some(*n),
        _ => None,
    }
}

/// The string `key` of `shape`, when it holds one.
pub(crate) fn string<'a>(shape: &'a Shape, key: &str) -> Option<&'a str> {
    match shape.fields.get(key) {
        Some(Value::Str(s)) => Some(s),
        _ => None,
    }
}

/// The `style` of `shape`, empty when unset.
pub(crate) fn style(shape: &Shape) -> Style {
    match shape.fields.get("style") {
        Some(Value::Style(style)) => style.clone(),
        _ => Style::default(),
    }
}

/// `text` escaped for HTML text and attribute values.
pub(crate) fn escape(text: &str) -> String {
    text.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;").replace('"', "&quot;")
}

/// A style value safe inside one CSS declaration.
pub(crate) fn css(value: &str) -> String {
    value.chars().filter(|c| !matches!(c, ';' | '{' | '}' | '\n' | '\r')).collect()
}

/// A body line the scene splitter would read as a separator or a code fence, indented so it
/// stays plain HTML.
pub(crate) fn guard(line: &str) -> String {
    let text = line.trim_start();
    let fence = text.starts_with("```") || text.starts_with("~~~");
    match fence || line.trim_end() == "---" {
        true => format!("    {line}"),
        false => line.to_owned(),
    }
}
