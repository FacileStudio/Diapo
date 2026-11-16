//! The PowerPoint file read back by other programs: python-pptx, and LibreOffice when it is here.
#![cfg(feature = "pptx")]

mod support;

use support::{DECK, PNG, files};

fn banner(text: &str) {
    use std::io::Write;
    let line = format!("\n================ SKIPPED: {text} ================\n");
    std::io::stderr().lock().write_all(line.as_bytes()).ok();
}

fn ran(program: &str, args: &[&std::ffi::OsStr]) -> Option<String> {
    let output = std::process::Command::new(program).args(args).output().ok()?;
    if !output.status.success() {
        use std::io::Write;
        std::io::stderr().lock().write_all(&output.stderr).ok();
    }
    output.status.success().then(|| String::from_utf8_lossy(&output.stdout).into_owned())
}

fn read_back(file: &std::path::Path, scratch: &std::path::Path) -> Option<String> {
    let script = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/support/read_pptx.py");
    let uv = ["run", "--no-project", "--quiet", "--with", "python-pptx", "python"].map(std::ffi::OsStr::new);
    let with_uv = [&uv[..], &[script.as_os_str(), file.as_os_str()]].concat();
    if let Some(answer) = ran("uv", &with_uv) {
        return Some(answer);
    }
    let venv = scratch.join("venv");
    ran("python3", &["-m".as_ref(), "venv".as_ref(), venv.as_os_str()])?;
    let quiet = ["install", "--quiet", "--disable-pip-version-check", "python-pptx"].map(std::ffi::OsStr::new);
    ran(venv.join("bin/pip").to_str()?, &quiet)?;
    ran(venv.join("bin/python").to_str()?, &[script.as_os_str(), file.as_os_str()])
}

#[test]
fn python_pptx_reads_the_file_back() {
    let dir = std::env::temp_dir().join(format!("scene-pptx-read-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let project = scene::parse(DECK).unwrap();
    let stills = |index: usize| (index == 3).then(|| PNG.to_vec());
    std::fs::write(dir.join("deck.pptx"), scene::pptx::write_with(&project, &files, &stills).unwrap()).unwrap();
    let answer = read_back(&dir.join("deck.pptx"), &dir);
    std::fs::remove_dir_all(&dir).ok();
    let Some(answer) = answer else {
        return banner(
            "python-pptx could not be run (no uv, no python3 venv or no network): the pptx was not read back",
        );
    };
    let slides: Vec<&str> = answer.split("{\"title\"").skip(1).collect();
    assert!(answer.contains("\"width\": 12192000, \"height\": 6858000") && slides.len() == 4, "{answer}");
    assert!(
        slides[0].starts_with(": \"Q3 review\"") && slides[0].contains("\"notes\": \"Thank the team first.\""),
        "{answer}"
    );
    assert!(slides[1].starts_with(": \"Revenue\"") && slides[1].contains("\"pictures\": [true]"), "{answer}");
    assert!(slides[1].contains("\"levels\": [0, 0, 1, 0, 0, 0]") && slides[1].contains("In France"), "{answer}");
    assert!(slides[2].contains("[[[\"Quarter\", \"Revenue\"], [\"Q2\", \"10\"], [\"Q3\", \"12\"]]]"), "{answer}");
    assert!(slides[3].contains("\"pictures\": [true]") && slides[3].contains("Pause here."), "{answer}");
}

#[test]
fn libreoffice_opens_the_file_when_it_is_installed() {
    let found = ["soffice", "libreoffice"].into_iter().find(|name| {
        std::process::Command::new(name).arg("--version").output().is_ok_and(|output| output.status.success())
    });
    let Some(office) = found else {
        return banner("no soffice on this machine: the pptx was not opened by LibreOffice");
    };
    let dir = std::env::temp_dir().join(format!("scene-pptx-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let project = scene::parse(DECK).unwrap();
    std::fs::write(dir.join("deck.pptx"), scene::pptx::write(&project, &files).unwrap()).unwrap();
    let home = format!("-env:UserInstallation=file://{}/profile", dir.display());
    let run = std::process::Command::new(office)
        .args([&home, "--headless", "--convert-to", "pdf", "--outdir"])
        .arg(&dir)
        .arg(dir.join("deck.pptx"))
        .output()
        .unwrap();
    let pdf = std::fs::read(dir.join("deck.pdf")).unwrap_or_default();
    std::fs::remove_dir_all(&dir).ok();
    assert!(run.status.success() && pdf.starts_with(b"%PDF"), "{}", String::from_utf8_lossy(&run.stderr));
}
