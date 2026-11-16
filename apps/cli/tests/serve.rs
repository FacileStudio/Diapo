//! `diapo serve`, through the binary.

#[path = "support/bin.rs"]
mod bin;

use std::fs;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpStream;
use std::process::{Child, Stdio};
use std::time::Duration;

/// A `diapo serve` child, killed when the test ends however it ends.
struct Served(Child);

impl Drop for Served {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

fn get(base: &str, path: &str) -> (u16, String) {
    let mut stream = TcpStream::connect(base).unwrap();
    stream
        .write_all(format!("GET {path} HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n").as_bytes())
        .unwrap();
    let mut answer = String::new();
    stream.read_to_string(&mut answer).unwrap();
    let status = answer.split_whitespace().nth(1).unwrap().parse().unwrap();
    (status, answer)
}

fn read_until(stream: &mut TcpStream, wanted: &str) {
    let (mut seen, mut buffer) = (String::new(), [0u8; 512]);
    while !seen.contains(wanted) {
        let read = stream.read(&mut buffer).unwrap();
        assert!(read > 0, "the stream closed before `{wanted}`: {seen}");
        seen.push_str(&String::from_utf8_lossy(&buffer[..read]));
    }
}

#[test]
fn serve_shows_the_deck_and_its_files_reloads_on_change_and_overlays_errors() {
    let work = tempfile::tempdir().unwrap();
    assert!(bin::command(work.path(), work.path(), &["new", "talk"]).status().unwrap().success());
    fs::write(work.path().join("talk/.diapo.json"), "{}").unwrap();
    let mut child = bin::command(work.path(), work.path(), &["serve", "talk", "-p", "0", "--json"]);
    let mut served = Served(child.stdout(Stdio::piped()).stderr(Stdio::null()).spawn().unwrap());
    let mut first = String::new();
    BufReader::new(served.0.stdout.take().unwrap()).read_line(&mut first).unwrap();
    let url = serde_json::from_str::<serde_json::Value>(&first).unwrap()["url"].as_str().unwrap().to_owned();
    let base = url.replace("http://localhost", "127.0.0.1");
    let (status, page) = get(&base, "/");
    assert!(status == 200 && page.contains("<title>talk</title>") && page.contains("new EventSource"), "{page}");
    assert!(get(&base, "/assets/chart.svg").1.contains("<svg"));
    assert_eq!((get(&base, "/.diapo.json").0, get(&base, "/../Cargo.toml").0), (404, 404));
    let mut events = TcpStream::connect(&base).unwrap();
    events.set_read_timeout(Some(Duration::from_secs(30))).unwrap();
    events.write_all(b"GET /__diapo/events HTTP/1.1\r\nHost: localhost\r\n\r\n").unwrap();
    read_until(&mut events, ": connected");
    fs::write(work.path().join("talk/diapo.md"), "---\nnope: 1\n---\n\n# Broken\n").unwrap();
    read_until(&mut events, "data: reload");
    let broken = get(&base, "/").1;
    assert!(
        broken.contains("id=\"diapo-problems\"") && broken.contains("talk/diapo.md:2: unknown key `nope`"),
        "{broken}"
    );
    assert!(broken.contains("Your turn"), "the last good deck stays under the overlay");
}
