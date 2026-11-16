//! The `diapo` binary, isolated from the caller's own login.

use std::path::Path;
use std::process::Command;

/// `diapo` with `args`, run in `cwd`, with a config home of its own and no credential from
/// the caller's environment.
pub fn command(cwd: &Path, config_home: &Path, args: &[&str]) -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_diapo"));
    command
        .args(args)
        .current_dir(cwd)
        .env("XDG_CONFIG_HOME", config_home)
        .env("NO_COLOR", "1")
        .env_remove("DIAPO_TOKEN")
        .env_remove("DIAPO_SERVER_URL")
        .env_remove("DIAPO_ISSUER");
    command
}
