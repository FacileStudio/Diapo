//! Diapo's capture worker.
//!
//! It polls one directory for jobs, captures one at a time in `chrome-headless-shell`, and
//! answers through files in the job's own directory. It opens no socket and reads no database:
//! the service runs with no network, and the directory is all it shares with the API. See
//! `docs/capture.md`.
#![deny(unused, dead_code, unsafe_code)]

mod config;
mod files;
mod queue;
mod run;
mod shared;
mod spec;

use std::process::ExitCode;
use std::time::Duration;

use capture::Tools;
use capture::job::{HEARTBEAT, fresh, now};
use serde_json::{Value, json};

use config::Config;

fn log(level: &str, message: &str, detail: Value) {
    eprintln!("{}", json!({"time": now(), "level": level, "msg": message, "detail": detail}));
}

/// Writes the heartbeat into the captures directory.
fn beat(config: &Config) {
    let _ = files::write(&config.captures.join(HEARTBEAT), now().as_bytes());
}

fn alive(config: &Config) -> bool {
    files::read(&config.captures.join(HEARTBEAT)).is_ok_and(|written| fresh(&written))
}

/// Waits until the captures directory exists. On a first deploy the service can start before
/// the one-shot that creates it, so a missing directory is waited for, not fatal.
fn wait_for_directory(config: &Config) {
    let mut waited = 0u64;
    while !files::is_dir(&config.captures) {
        if waited.is_multiple_of(30) {
            log("warn", "waiting for the captures directory", json!({"captures": config.captures}));
        }
        std::thread::sleep(Duration::from_secs(1));
        waited += 1;
    }
}

fn serve(config: &Config, tools: &Tools) -> ! {
    wait_for_directory(config);
    let recovered = queue::recover(config);
    let started = json!({"captures": config.captures, "recovered": recovered, "sandbox": tools.sandbox});
    log("info", "capture worker started", started);
    loop {
        beat(config);
        let Some(dir) = queue::next(config) else {
            std::thread::sleep(config.poll);
            continue;
        };
        let job = dir.file_name().unwrap_or_default().to_string_lossy().into_owned();
        log("info", "capture started", json!({"job": job}));
        let status = run::run(&dir, tools, config);
        let finished = json!({"job": job, "state": status.state, "done": status.done, "error": status.error});
        log("info", "capture finished", finished);
    }
}

fn main() -> ExitCode {
    let config = Config::from_env();
    if std::env::args().nth(1).as_deref() == Some("healthcheck") {
        return if alive(&config) { ExitCode::SUCCESS } else { ExitCode::FAILURE };
    }
    match Tools::locate() {
        Ok(tools) => serve(&config, &tools),
        Err(error) => {
            log("error", "the browser is missing", json!({"error": error.to_string()}));
            ExitCode::FAILURE
        }
    }
}
