#![allow(dead_code)]

use std::collections::BTreeMap;
use std::io::{Cursor, Read};
use std::path::Path;

pub const PNG: &[u8] = b"\x89PNG\r\n\x1a\n\0\0\0\rIHDR\0\0\x03\x20\0\0\x01\x90\x08\x06\0\0\0\x1f\x15\xc4\x89";

pub const DECK: &str = "---\ntitle: Q3 review\nlayout: cover\n---\n\n# Q3 review\n\nBoard & team, 1 < 2\n\n\
<!-- Thank the team first. -->\n\n---\nlayout: split\nsteps: 2\n---\n\n## Revenue\n\n- Up **12%** {step=1}\n  - In France\n\
- Churn down {step=2}\n\n1. First\n\n![Chart](assets/chart.png)\n\n![Logo](assets/logo.svg)\n\n---\n\n## Numbers\n\n\
| Quarter | Revenue |\n|---|---|\n| Q2 | 10 |\n| Q3 | 12 |\n\n> Quoted\n\n```\nlet x = 1;\n```\n\n---\nlayout: raw\n\
duration: 2\n---\n\n<div class=\"clip\" data-start=\"0\" data-duration=\"2\"><h1 id=\"t\">Launch</h1></div>\n\n\
<!-- Pause here. -->\n";

pub fn files(path: &str) -> Option<Vec<u8>> {
    match path {
        "assets/chart.png" => Some(PNG.to_vec()),
        "assets/logo.svg" => Some(b"<svg xmlns=\"http://www.w3.org/2000/svg\"/>".to_vec()),
        _ => None,
    }
}

pub fn unzip(bytes: &[u8]) -> BTreeMap<String, Vec<u8>> {
    let mut archive = zip::ZipArchive::new(Cursor::new(bytes)).unwrap();
    let mut parts = BTreeMap::new();
    for index in 0..archive.len() {
        let mut file = archive.by_index(index).unwrap();
        let mut content = Vec::new();
        file.read_to_end(&mut content).unwrap();
        let name = file.name().unwrap().into_owned();
        parts.insert(name, content);
    }
    parts
}

pub fn text(parts: &BTreeMap<String, Vec<u8>>, name: &str) -> String {
    String::from_utf8(parts.get(name).unwrap_or_else(|| panic!("no part {name}")).clone()).unwrap()
}

pub fn well_formed(name: &str, xml: &str) {
    let mut open: Vec<&str> = Vec::new();
    let mut rest = xml;
    while let Some(start) = rest.find('<') {
        let end = start + rest[start..].find('>').unwrap_or_else(|| panic!("{name}: unclosed tag"));
        let tag = &rest[start + 1..end];
        rest = &rest[end + 1..];
        let label = tag.trim_start_matches('/').split([' ', '/']).next().unwrap();
        if tag.starts_with('?') || tag.ends_with('/') {
            continue;
        }
        match tag.starts_with('/') {
            true => assert_eq!(open.pop(), Some(label), "{name}: </{label}> closes the wrong element"),
            false => open.push(label),
        }
    }
    assert!(open.is_empty(), "{name}: left open {open:?}");
    assert_eq!(xml.matches('"').count() % 2, 0, "{name}: an odd number of quotes");
}

fn cut(text: &str, open: &str, close: &str, label: &str) -> String {
    let start = text.find(open).unwrap() + open.len();
    let end = start + text[start..].find(close).unwrap();
    format!("{}{label}{}", &text[..start], &text[end..])
}

pub fn body_of(html: &str) -> String {
    let styled = cut(html, "</title>\n<style>", "</style>", "/* theme */");
    cut(&styled, "<script>", "</script>\n</head>", "/* runtime */")
}

pub fn golden(path: &Path, actual: &str) {
    if std::env::var_os("UPDATE_GOLDEN").is_some() {
        std::fs::write(path, actual).unwrap();
    }
    let expected = std::fs::read_to_string(path).unwrap_or_default();
    assert!(expected == actual, "{} differs from the compiled output:\n{actual}", path.display());
}
