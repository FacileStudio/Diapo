//! Chrome's `--remote-debugging-pipe`: JSON messages ended by a NUL byte, commands on the
//! child's descriptor 3, answers and events on 4. No port is opened, so only the process that
//! started Chrome can drive it.

use std::fs::File;
use std::io::{BufRead, BufReader, PipeReader, PipeWriter};
use std::path::Path;
use std::process::{Child, Command, Stdio};
use std::sync::mpsc::{self, Receiver, Sender};

use command_fds::{CommandFdExt, FdMapping};
use serde_json::Value;

use crate::Error;

/// A started Chrome: the process, where commands go, and the messages it sends back.
pub(crate) struct Piped {
    pub(crate) child: Child,
    pub(crate) commands: PipeWriter,
    pub(crate) inbox: Receiver<Value>,
}

fn forward(answers: PipeReader, outbox: Sender<Value>) {
    let mut reader = BufReader::new(answers);
    let mut message = Vec::new();
    while reader.read_until(0, &mut message).is_ok_and(|read| read > 0) {
        let body = message.strip_suffix(&[0]).unwrap_or(&message);
        let delivered = serde_json::from_slice(body).ok().map(|value| outbox.send(value).is_ok());
        if delivered == Some(false) {
            return;
        }
        message.clear();
    }
}

/// Starts `chrome` with `flags`, its profile and its stderr log under `profile`, and a thread
/// that reads its messages until the pipe closes.
pub(crate) fn spawn(chrome: &Path, flags: Vec<String>, profile: &Path) -> Result<Piped, Error> {
    let (to_chrome, commands) = std::io::pipe()?;
    let (answers, from_chrome) = std::io::pipe()?;
    let mut command = Command::new(chrome);
    command
        .args(flags)
        .arg(format!("--user-data-dir={}", profile.display()))
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(File::create(profile.join("stderr.log"))?);
    let mapped = vec![
        FdMapping { parent_fd: to_chrome.into(), child_fd: 3 },
        FdMapping { parent_fd: from_chrome.into(), child_fd: 4 },
    ];
    command.fd_mappings(mapped).map_err(|error| Error::Chrome(error.to_string()))?;
    let child = command.spawn()?;
    drop(command);
    let (outbox, inbox) = mpsc::channel();
    std::thread::spawn(move || forward(answers, outbox));
    Ok(Piped { child, commands, inbox })
}
