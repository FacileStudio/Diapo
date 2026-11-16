//! Compiles a project file: `compile <source.md> <out.html> [scene-index]`.

use std::process::ExitCode;

fn run(args: &[String]) -> Result<(), String> {
    let [source, out, rest @ ..] = args else { return Err("usage: compile <source.md> <out.html> [scene]".to_owned()) };
    let text = std::fs::read_to_string(source).map_err(|error| format!("{source}: {error}"))?;
    let project = scene::parse(&text).map_err(|errors| errors.to_string())?;
    let options = scene::Options::new();
    let html = match rest.first() {
        Some(index) => {
            let index: usize = index.parse().map_err(|_| "the scene index is a number from 0".to_owned())?;
            scene::compile_scene(&project, index, &options).ok_or("no such scene")?
        }
        None => scene::compile(&project, &options),
    };
    std::fs::write(out, html).map_err(|error| format!("{out}: {error}"))
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match run(&args) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}
