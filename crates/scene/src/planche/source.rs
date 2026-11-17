//! A planche document to scene source: one `layout: raw` scene per top-level frame.

use ::planche::{Document, Shape, Value};

use super::content::element;
use super::values::{guard, number, string};

const STYLE: &str = "<style>.pl{position:absolute;margin:0}\
:is(.pl-text,.pl-sticky,.pl-rect,.pl-ellipse,.pl-diamond){overflow-wrap:break-word}\
:is(.pl-text,.pl-sticky,.pl-rect,.pl-ellipse,.pl-diamond)>:last-child{margin-bottom:0}\
.pl-rect,.pl-ellipse,.pl-diamond{display:flex;flex-direction:column;justify-content:center;text-align:center;padding:24px}\
.pl-sticky{padding:32px;color:#1a1a1a;box-shadow:0 12px 32px rgba(0,0,0,.14)}\
.pl-image>img,.pl-video>video{display:block;width:100%;height:100%;max-width:none;max-height:none}\
.pl-card>a{display:flex;flex-direction:column;gap:12px;height:100%;padding:32px;border-radius:28px;\
background:var(--fill);text-decoration:none}\
.pl-card small{color:var(--muted)}</style>";

type Entry<'a> = (&'a String, &'a Shape);

fn frames(document: &Document) -> Vec<Entry<'_>> {
    let mut found: Vec<Entry<'_>> = document
        .shapes
        .iter()
        .filter(|(_, shape)| shape.kind == "frame" && !shape.fields.contains_key("parent"))
        .collect();
    found.sort_by_key(|(id, shape)| (string(shape, "order").is_none(), string(shape, "order"), *id));
    found
}

fn children<'a>(document: &'a Document, frame: &str) -> Vec<Entry<'a>> {
    let mut found: Vec<Entry<'_>> = document
        .shapes
        .iter()
        .filter(|(_, shape)| shape.kind != "frame" && string(shape, "parent") == Some(frame))
        .collect();
    found.sort_by_key(|(id, shape)| {
        let track = number(shape, "track").unwrap_or_default() as i64;
        (track, string(shape, "z").is_some(), string(shape, "z"), *id)
    });
    found
}

fn headmatter(document: &Document, first: Option<&Shape>) -> String {
    let (w, h) = document.meta.size.unwrap_or((1920, 1080));
    let side = |n: u32| n.clamp(16, 16_384);
    let name = first.and_then(|frame| string(frame, "name")).unwrap_or_default();
    let title: String = name.chars().map(|c| if c.is_control() { ' ' } else { c }).collect();
    let title = match title.trim() {
        "" => String::new(),
        text => format!("title: \"{text}\"\n"),
    };
    format!("{title}size: {}x{}\n", side(w), side(h))
}

fn front(frame: &Shape) -> String {
    let fixed = matches!(frame.fields.get("fixed_duration"), Some(Value::Bool(true)));
    match number(frame, "duration").filter(|seconds| fixed && *seconds > 0.0) {
        Some(seconds) => format!("layout: raw\nduration: {seconds}\n"),
        None => "layout: raw\n".to_owned(),
    }
}

fn note(frame: &Shape) -> Option<String> {
    let Some(Value::Rich(notes)) = frame.fields.get("notes") else { return None };
    let text = encre::markdown::serialize(&notes.0).replace("-->", "-- >").replace("<!--", "<! --");
    let text = text.trim();
    (!text.is_empty()).then(|| format!("\n\n<!-- {text} -->"))
}

fn body(document: &Document, id: &str, frame: &Shape) -> String {
    let shapes = children(document, id).into_iter().filter_map(|(_, shape)| element(shape));
    let html: Vec<String> = std::iter::once(STYLE.to_owned()).chain(shapes).collect();
    let lines: Vec<String> = html.join("\n").split('\n').map(guard).collect();
    format!("{}{}\n", lines.join("\n"), note(frame).unwrap_or_default())
}

/// The scene source of `document`.
pub(crate) fn write(document: &Document) -> String {
    let all = frames(document);
    let mut out = format!("---\n{}", headmatter(document, all.first().map(|(_, frame)| *frame)));
    match all.split_first() {
        None => out.push_str("layout: raw\n---\n"),
        Some(((id, frame), rest)) => {
            out.push_str(&format!("{}---\n\n{}", front(frame), body(document, id, frame)));
            for (id, frame) in rest {
                out.push_str(&format!("\n---\n{}---\n\n{}", front(frame), body(document, id, frame)));
            }
        }
    }
    out
}
