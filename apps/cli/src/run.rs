//! From a parsed command line to an exit code.

use std::process::ExitCode;
use std::sync::atomic::Ordering;

use crate::cli::{Cli, Command, Globals, Local, Remote};
use crate::failure::Outcome;
use crate::{commands, serve, ui};

fn local(command: &Local) -> Outcome {
    match command {
        Local::New(args) => commands::new::run(args),
        Local::Serve(args) => serve::run(args),
        Local::Build(args) => commands::build::run(args),
        Local::Check(args) => commands::check::run(args),
        Local::Guide => commands::guide::run(),
        Local::Export(_) => commands::export::run(),
    }
}

fn remote(command: &Remote, globals: &Globals) -> Outcome {
    match command {
        Remote::Login(args) => commands::login::run(args, globals),
        Remote::Logout => commands::logout::run(),
        Remote::Whoami => commands::whoami::run(globals),
        Remote::List(args) => commands::list::run(args, globals),
        Remote::Pull(args) => commands::pull::run(args, globals),
        Remote::Push(args) => commands::push::run(args, globals),
        Remote::Open(args) => commands::open::run(args),
    }
}

/// Runs the command and answers the exit code: 0, or 1 after printing why.
pub fn run(cli: Cli) -> ExitCode {
    ui::JSON.store(cli.globals.json, Ordering::Relaxed);
    ui::PLAIN.store(cli.globals.no_color, Ordering::Relaxed);
    let outcome = match &cli.command {
        Command::Local(command) => local(command),
        Command::Remote(command) => remote(command, &cli.globals),
    };
    match outcome {
        Ok(()) => ExitCode::SUCCESS,
        Err(failure) => {
            failure.report();
            ExitCode::from(1)
        }
    }
}
