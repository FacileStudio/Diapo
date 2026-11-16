//! The served page: the compiled deck plus the reload script, or the problems as an overlay.

use scene::Options;

use super::Site;
use crate::diagnose;
use crate::ui;

/// The path of the event stream.
pub const EVENTS: &str = "/__diapo/events";

const BLANK: &str = "<!doctype html>\n<html lang=\"en\">\n<head><meta charset=\"utf-8\"><title>diapo</title></head>\n\
                     <body>\n</body>\n</html>\n";

const RELOAD: &str = "<script>(function () {\n  var key = \"diapo:step\";\n  var kept = sessionStorage.getItem(key);\n  \
    window.addEventListener(\"load\", function () {\n    if (kept !== null && window.__scene) window.__scene.step(Number(kept));\n  });\n  \
    new EventSource(\"/__diapo/events\").onmessage = function () {\n    \
    if (window.__scene) sessionStorage.setItem(key, String(window.__scene.state().step));\n    location.reload();\n  };\n\
    })();</script>\n";

const OVERLAY_STYLE: &str = "position:fixed;inset:0;z-index:2147483647;overflow:auto;padding:48px;\
    background:rgba(24,24,27,.94);color:#fafafa;font:18px/1.6 ui-monospace,SFMono-Regular,Menlo,monospace";

fn escape(text: &str) -> String {
    text.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;")
}

fn inject(html: &str, addition: &str) -> String {
    match html.rfind("</body>") {
        Some(at) => [&html[..at], addition, &html[at..]].concat(),
        None => [html, addition].concat(),
    }
}

fn overlay(problems: &[String]) -> String {
    let title = format!("{} in the deck, fix the file and this page reloads", diagnose::count(problems.len()));
    let lines: Vec<String> = problems.iter().map(|problem| escape(problem)).collect();
    format!(
        "<div id=\"diapo-problems\" role=\"alert\" style=\"{OVERLAY_STYLE}\"><strong>{title}</strong>\
         <pre style=\"margin:24px 0 0;white-space:pre-wrap;font:inherit\">{}</pre></div>\n",
        lines.join("\n")
    )
}

fn compile(site: &Site) -> Result<String, Vec<String>> {
    let label = site.folder.label();
    let report = diagnose::inspect(&site.folder).map_err(|failure| vec![failure.message])?;
    let project = report.project.ok_or_else(|| diagnose::lines(&label, &report.problems))?;
    let html = scene::compile(&project, &Options::new());
    if let Ok(mut last_good) = site.last_good.lock() {
        *last_good = Some(html.clone());
    }
    Ok(html)
}

/// The page to serve now: the deck as it is on disk, or the last deck that compiled under an
/// overlay listing the problems.
pub fn render(site: &Site) -> String {
    let page = compile(site).unwrap_or_else(|problems| {
        let kept = site.last_good.lock().ok().and_then(|last_good| last_good.clone());
        inject(kept.as_deref().unwrap_or(BLANK), &overlay(&problems))
    });
    inject(&page, RELOAD)
}

/// Prints the state of the deck in the terminal: every problem as `file:line: message`.
pub fn report(site: &Site) {
    let label = site.folder.label();
    match diagnose::inspect(&site.folder) {
        Err(failure) => ui::error(&failure.message),
        Ok(report) => {
            let broken = report.project.is_none();
            for line in diagnose::lines(&label, &report.problems) {
                if broken { ui::error(&line) } else { ui::warn(&line) }
            }
        }
    }
}
