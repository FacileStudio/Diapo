use axum::body::Body;
use http::{Request, header};

use super::http::{Answer, raw};
use super::{App, Person};

pub const PNG: &[u8] = b"\x89PNG\r\n\x1a\n\0\0\0\rIHDR\0\0\0\x01\0\0\0\x01\x08\x06\0\0\0\x1f\x15\xc4\x89";

pub const BOUNDARY: &str = "diapo-test-boundary";

pub struct File<'a> {
    pub name: Option<&'a str>,
    pub file_name: &'a str,
    pub bytes: &'a [u8],
}

pub fn body(file: &File<'_>) -> Vec<u8> {
    let mut body = Vec::new();
    if let Some(name) = file.name {
        let part = format!("--{BOUNDARY}\r\nContent-Disposition: form-data; name=\"name\"\r\n\r\n{name}\r\n");
        body.extend_from_slice(part.as_bytes());
    }
    let head = format!(
        "--{BOUNDARY}\r\nContent-Disposition: form-data; name=\"file\"; filename=\"{}\"\r\n\
         Content-Type: text/html\r\n\r\n",
        file.file_name
    );
    body.extend_from_slice(head.as_bytes());
    body.extend_from_slice(file.bytes);
    body.extend_from_slice(format!("\r\n--{BOUNDARY}--\r\n").as_bytes());
    body
}

pub async fn upload(app: &App, who: &Person, deck: &str, file: File<'_>) -> Answer {
    let request = Request::post(format!("/api/decks/{deck}/assets"))
        .header(header::HOST, "diapo.test")
        .header(header::AUTHORIZATION, format!("Bearer {}", who.token))
        .header(header::CONTENT_TYPE, format!("multipart/form-data; boundary={BOUNDARY}"))
        .body(Body::from(body(&file)))
        .unwrap();
    raw(&app.router, request).await
}

pub async fn png(app: &App, who: &Person, deck: &str, name: &str) -> Answer {
    upload(app, who, deck, File { name: Some(name), file_name: "upload.png", bytes: PNG }).await
}
