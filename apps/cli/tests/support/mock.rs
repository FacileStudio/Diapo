//! A mock HTTP server in the test process.

use std::sync::{Arc, Mutex};
use std::thread;

use tiny_http::{Header, Response, Server};

/// What the mock saw: one `METHOD path [authorization] body` line per request.
pub type Seen = Arc<Mutex<Vec<String>>>;

/// Starts a server that answers each request with what `handler` gives for its
/// `METHOD path` and its body. Answers the base URL and the requests seen so far.
pub fn mock(handler: impl Fn(&str, &str) -> (u16, String) + Send + 'static) -> (String, Seen) {
    let server = Server::http("127.0.0.1:0").unwrap();
    let url = format!("http://{}", server.server_addr());
    let seen = Seen::default();
    let log = Arc::clone(&seen);
    thread::spawn(move || {
        for mut request in server.incoming_requests() {
            let mut body = Vec::new();
            request.as_reader().read_to_end(&mut body).unwrap();
            let body = String::from_utf8_lossy(&body).into_owned();
            let authorization = request.headers().iter().find(|header| header.field.equiv("Authorization"));
            let authorization = authorization.map(|header| header.value.to_string()).unwrap_or_default();
            let line = format!("{} {}", request.method(), request.url());
            log.lock().unwrap().push(format!("{line} [{authorization}] {body}"));
            let (status, answer) = handler(&line, &body);
            let json = Header::from_bytes("Content-Type", "application/json").unwrap();
            request.respond(Response::from_string(answer).with_status_code(status).with_header(json)).unwrap();
        }
    });
    (url, seen)
}
