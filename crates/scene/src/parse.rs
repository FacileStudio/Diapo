//! Source text to a valid [`Project`], or every problem found.

use crate::audit::Audit;
use crate::error::{Error, ErrorKind, Errors};
use crate::frontmatter::{self, Front};
use crate::model::{Config, Layout, Project, Scene};
use crate::split::{Block, split};
use crate::{markdown, note};

fn line_of(body: &str, offset: usize, first: usize) -> usize {
    first + body[..offset.min(body.len())].matches('\n').count()
}

fn scene(block: &Block<'_>, front: Front, config: &Config, errors: &mut Vec<Error>) -> Scene {
    let (body, note) = note::take(&block.body.join("\n"));
    let layout = front.layout.unwrap_or(Layout::Default);
    let (html, audit) = match layout {
        Layout::Raw => {
            let mut audit = Audit::default();
            audit.html(&body, 0);
            (body.trim().to_owned(), audit)
        }
        _ => markdown::render(&body),
    };
    errors.extend(audit.problems.into_iter().map(|(at, kind)| Error { line: line_of(&body, at, block.line), kind }));
    let steps = front.steps.unwrap_or(audit.steps);
    if audit.steps > steps {
        errors.push(Error { line: block.line, kind: ErrorKind::StepOutOfRange { step: audit.steps, steps } });
    }
    let stops = f64::from(steps + 1) * config.step_duration;
    let timed = Some(audit.clip_end).filter(|end| *end > 0.0);
    let duration = front.duration.or(timed).unwrap_or(stops);
    Scene { layout, steps, duration, class: front.class, body, note, line: block.line, html }
}

/// Parses and validates `source`.
pub(crate) fn parse(source: &str) -> Result<Project, Errors> {
    let mut errors = Vec::new();
    let blocks = split(source, &mut errors);
    let mut config = Config::default();
    let mut scenes = Vec::new();
    for (index, block) in blocks.iter().enumerate() {
        let head = (index == 0).then_some(&mut config);
        let front = frontmatter::read(&block.front, head, &mut errors);
        scenes.push(scene(block, front, &config, &mut errors));
    }
    if scenes.is_empty() {
        errors.push(Error { line: 1, kind: ErrorKind::Empty });
    }
    errors.sort_by_key(|error| error.line);
    match errors.is_empty() {
        true => Ok(Project { config, scenes }),
        false => Err(Errors(errors)),
    }
}
