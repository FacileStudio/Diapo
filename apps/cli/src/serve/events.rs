//! Live reload over server-sent events: every open page holds one stream, and a change to
//! the folder writes one event to each.

use std::io::Write;
use std::sync::Mutex;

use tiny_http::Request;

const HEAD: &str = "HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\nCache-Control: no-store\r\n\
                    Connection: keep-alive\r\n\r\n: connected\n\n";

/// The open event streams.
#[derive(Default)]
pub struct Hub {
    streams: Mutex<Vec<Box<dyn Write + Send>>>,
}

fn send(stream: &mut Box<dyn Write + Send>, text: &str) -> bool {
    stream.write_all(text.as_bytes()).and_then(|()| stream.flush()).is_ok()
}

impl Hub {
    /// Turns `request` into an event stream and keeps it.
    pub fn subscribe(&self, request: Request) {
        let mut stream = request.into_writer();
        if let (true, Ok(mut streams)) = (send(&mut stream, HEAD), self.streams.lock()) {
            streams.push(stream);
        }
    }

    /// Tells every page to reload, and forgets the ones that went away.
    pub fn reload(&self) {
        if let Ok(mut streams) = self.streams.lock() {
            streams.retain_mut(|stream| send(stream, "data: reload\n\n"));
        }
    }
}
