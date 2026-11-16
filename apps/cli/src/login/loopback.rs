//! The loopback sign-in: the browser signs in at the server, which redirects to a port on
//! this machine with a one-time code and the nonce this login sent.

use std::fs::File;
use std::io::{self, ErrorKind, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::thread;
use std::time::{Duration, Instant};

use crate::api::Client;
use crate::failure::{Failure, Outcome};
use crate::{browser, ui, url};

const WAIT: Duration = Duration::from_secs(180);

/// Sixteen random bytes in hex: the nonce that tells this login's callback from any other.
pub fn nonce() -> io::Result<String> {
    let mut bytes = [0u8; 16];
    File::open("/dev/urandom")?.read_exact(&mut bytes)?;
    Ok(bytes.iter().map(|byte| format!("{byte:02x}")).collect())
}

fn reply(stream: &mut TcpStream, status: &str, heading: &str, message: &str) {
    let body = format!(
        "<!doctype html><html lang=\"en\"><meta charset=\"utf-8\">\
         <meta name=\"viewport\" content=\"width=device-width,initial-scale=1\"><title>{heading} · Diapo</title>\
         <style>:root{{color-scheme:light dark}}body{{margin:0;min-height:100dvh;display:grid;place-items:center;\
         font:16px/1.6 ui-sans-serif,system-ui,sans-serif}}main{{text-align:center;padding:1.5rem}}\
         h1{{margin:0;font-size:1.25rem}}p{{margin:.5rem 0 0;opacity:.7}}</style>\
         <main><h1>{heading}</h1><p>{message}</p></main>"
    );
    let head = format!(
        "HTTP/1.1 {status}\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {}\r\n\
         Cache-Control: no-store\r\nConnection: close\r\n\r\n",
        body.len()
    );
    let _ = stream.write_all(head.as_bytes()).and_then(|()| stream.write_all(body.as_bytes()));
}

fn answer(mut stream: TcpStream, state: &str) -> Option<String> {
    stream.set_nonblocking(false).ok()?;
    stream.set_read_timeout(Some(Duration::from_secs(2))).ok()?;
    let mut buffer = [0u8; 4096];
    let read = stream.read(&mut buffer).ok()?;
    let request = String::from_utf8_lossy(&buffer[..read]);
    let target = request.split_whitespace().nth(1).unwrap_or("");
    let ours = url::query(target, "state").as_deref() == Some(state);
    match url::query(target, "code") {
        Some(code) if ours => {
            reply(&mut stream, "200 OK", "Signed in", "You can close this tab and go back to your terminal.");
            Some(code)
        }
        Some(_) => {
            reply(&mut stream, "400 Bad Request", "Not signed in", "This callback is not from the login in progress.");
            None
        }
        None => {
            reply(&mut stream, "404 Not Found", "Not found", "This is not the login redirect.");
            None
        }
    }
}

/// Serves the listener until the callback with a code and this login's `state` arrives. A
/// request without a code, or with another state, is refused and the wait goes on.
pub fn wait(listener: &TcpListener, state: &str, deadline: Instant) -> Outcome<String> {
    listener.set_nonblocking(true)?;
    while Instant::now() < deadline {
        match listener.accept() {
            Ok((stream, _)) => {
                if let Some(code) = answer(stream, state) {
                    return Ok(code);
                }
            }
            Err(error) if error.kind() == ErrorKind::WouldBlock => thread::sleep(Duration::from_millis(50)),
            Err(error) => return Err(error.into()),
        }
    }
    Err(Failure::new("timeout", "timed out waiting for the browser — run `diapo login` again"))
}

/// Signs in through the browser and answers the session token. Without `open`, or when no
/// browser opens, the link is printed for the person to follow.
pub fn sign_in(client: &Client, open: bool) -> Outcome<String> {
    let listener = TcpListener::bind("127.0.0.1:0")?;
    let state = nonce()?;
    let link = client.login_url(listener.local_addr()?.port(), &state);
    ui::step("Waiting for the sign-in in your browser");
    if !open || !browser::open(&link) {
        ui::hint(&format!("Open this URL in a browser on this machine: {link}"));
    }
    let code = wait(&listener, &state, Instant::now() + WAIT)?;
    Ok(client.exchange_code(&code)?)
}
