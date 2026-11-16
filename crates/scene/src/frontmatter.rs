//! Reads frontmatter lines into the typed configuration of a project and of a scene.

use crate::error::{Error, ErrorKind};
use crate::model::{Config, Layout, Scheme, Size};

/// What a scene's frontmatter says, before defaults.
#[derive(Debug, Default)]
pub(crate) struct Front {
    pub layout: Option<Layout>,
    pub steps: Option<u32>,
    pub duration: Option<f64>,
    pub class: String,
}

const PROJECT_KEYS: [&str; 6] = ["title", "theme", "scheme", "size", "step_duration", "lang"];

fn value(raw: &str) -> &str {
    let text = raw.trim();
    let quoted = |q: char| text.len() >= 2 && text.starts_with(q) && text.ends_with(q);
    if quoted('"') || quoted('\'') {
        return &text[1..text.len() - 1];
    }
    text.split(" #").next().unwrap_or_default().trim_end()
}

fn seconds(text: &str) -> Option<f64> {
    text.parse::<f64>().ok().filter(|n| n.is_finite() && *n > 0.0 && *n <= 86_400.0)
}

fn size(text: &str) -> Option<Size> {
    let (width, height) = text.split_once('x')?;
    let side = |raw: &str| raw.parse::<u32>().ok().filter(|n| (16..=16_384).contains(n));
    Some(Size { width: side(width)?, height: side(height)? })
}

fn scene_key(front: &mut Front, key: &str, text: &str) -> Result<(), &'static str> {
    match key {
        "layout" => {
            let known = [Layout::Default, Layout::Cover, Layout::Center, Layout::Split, Layout::Raw];
            let layout = known.into_iter().find(|layout| layout.name() == text);
            front.layout = Some(layout.ok_or("one of default, cover, center, split, raw")?);
        }
        "steps" => {
            let steps = text.parse::<u32>().ok().filter(|n| *n <= 999);
            front.steps = Some(steps.ok_or("a whole number from 0 to 999")?);
        }
        "duration" => front.duration = Some(seconds(text).ok_or("a number of seconds above 0")?),
        _ => {
            let plain = text.bytes().all(|b| b.is_ascii_alphanumeric() || matches!(b, b' ' | b'-' | b'_'));
            front.class = plain.then(|| text.to_owned()).ok_or("class names made of letters, digits, - and _")?;
        }
    }
    Ok(())
}

fn project_key(config: &mut Config, key: &str, text: &str) -> Result<(), &'static str> {
    match key {
        "title" => config.title = text.to_owned(),
        "theme" => (text == "facile").then_some(()).ok_or("facile")?,
        "scheme" => {
            let scheme = [Scheme::Light, Scheme::Dark, Scheme::Auto].into_iter().find(|s| s.name() == text);
            config.scheme = scheme.ok_or("one of light, dark, auto")?;
        }
        "size" => config.size = size(text).ok_or("WIDTHxHEIGHT in pixels, each from 16 to 16384")?,
        "step_duration" => config.step_duration = seconds(text).ok_or("a number of seconds above 0")?,
        _ => {
            let tag =
                !text.is_empty() && text.len() <= 35 && text.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-');
            config.lang = tag.then(|| text.to_owned()).ok_or("a language tag such as en or fr-FR")?;
        }
    }
    Ok(())
}

fn assign(front: &mut Front, config: Option<&mut Config>, key: &str, text: &str) -> Result<(), ErrorKind> {
    let bad = |expected| ErrorKind::BadValue { key: key.to_owned(), expected };
    let project = PROJECT_KEYS.contains(&key);
    match config {
        _ if ["layout", "steps", "duration", "class"].contains(&key) => scene_key(front, key, text).map_err(bad),
        Some(config) if project => project_key(config, key, text).map_err(bad),
        None if project => Err(ErrorKind::ProjectKeyInScene { key: key.to_owned() }),
        _ => Err(ErrorKind::UnknownKey { key: key.to_owned() }),
    }
}

/// Reads one frontmatter block. With `config`, the block is the headmatter and may set
/// project keys too.
pub(crate) fn read(lines: &[(usize, &str)], mut config: Option<&mut Config>, errors: &mut Vec<Error>) -> Front {
    let mut front = Front::default();
    let mut seen: Vec<&str> = Vec::new();
    for (line, text) in lines {
        let (key, raw) = text.split_once(':').unwrap_or((text, ""));
        let result = match seen.contains(&key) {
            true => Err(ErrorKind::DuplicateKey { key: key.to_owned() }),
            false => assign(&mut front, config.as_deref_mut(), key, value(raw)),
        };
        seen.push(key);
        if let Err(kind) = result {
            errors.push(Error { line: *line, kind });
        }
    }
    front
}
