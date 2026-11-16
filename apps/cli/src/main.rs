//! The `diapo` binary: parses the command line and hands it to the library.

use std::process::ExitCode;

use clap::Parser;

fn main() -> ExitCode {
    diapo_cli::run(diapo_cli::cli::Cli::parse())
}
