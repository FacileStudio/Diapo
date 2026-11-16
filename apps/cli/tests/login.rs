//! `diapo login` through the binary: both flows against a mock server, and what is stored.

#[path = "support/bin.rs"]
mod bin;
#[path = "support/mock.rs"]
mod mock;

use std::fs;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpStream;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::process::Stdio;
use std::sync::{Arc, OnceLock};

use mock::mock;

fn text(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).into_owned()
}

fn stored(home: &Path) -> (String, u32, u32) {
    let file = home.join("diapo/config.yml");
    let mode = |path: &Path| fs::metadata(path).unwrap().permissions().mode() & 0o777;
    (fs::read_to_string(&file).unwrap(), mode(&file), mode(&home.join("diapo")))
}

fn porte(line: &str, body: &str, device: bool) -> (u16, String) {
    match (line, body) {
        ("GET /api/auth/config", _) => (200, r#"{"sso_only":true,"oidc_enabled":true}"#.to_owned()),
        ("POST /api/auth/oidc/exchange", r#"{"code":"one-time"}"#) => {
            (200, r#"{"user_id":"1","token":"loop-session"}"#.to_owned())
        }
        ("POST /api/auth/oidc/device/exchange", "{}") if device => (400, "{}".to_owned()),
        ("POST /api/auth/oidc/device/exchange", r#"{"access_token":"registre-access"}"#) if device => {
            (200, r#"{"user_id":"1","token":"device-session"}"#.to_owned())
        }
        ("GET /api/auth/me", _) => (200, r#"{"email":"ada@example.com"}"#.to_owned()),
        _ => (404, r#"{"error":{"code":"not_found","message":"route not found"}}"#.to_owned()),
    }
}

#[test]
fn the_loopback_login_exchanges_the_code_and_stores_the_token_privately() {
    let (url, seen) = mock(|line, body| porte(line, body, false));
    let home = tempfile::tempdir().unwrap();
    let mut command = bin::command(home.path(), home.path(), &["login", &format!("{url}/api"), "--no-browser"]);
    let mut child = command.stdout(Stdio::piped()).spawn().unwrap();
    let mut lines = BufReader::new(child.stdout.take().unwrap()).lines().map(Result::unwrap);
    let link = lines.find(|line| line.contains("cli_state=")).unwrap();
    assert!(link.contains(&format!("{url}/api/auth/oidc?flow=cli&port=")), "{link}");
    let port = link.split("port=").nth(1).unwrap().split('&').next().unwrap().to_owned();
    let state = link.rsplit("cli_state=").next().unwrap().trim().to_owned();
    let mut browser = TcpStream::connect(format!("127.0.0.1:{port}")).unwrap();
    browser.write_all(format!("GET /?code=one-time&state={state} HTTP/1.1\r\n\r\n").as_bytes()).unwrap();
    let mut page = String::new();
    browser.read_to_string(&mut page).unwrap();
    assert!(page.contains("Signed in"), "{page}");
    assert!(lines.any(|line| line.contains(&format!("Signed in to {url} as ada@example.com"))));
    assert!(child.wait().unwrap().success());
    assert_eq!(stored(home.path()), (format!("server_url: {url}\ntoken: loop-session\n"), 0o600, 0o700));
    assert!(seen.lock().unwrap().iter().all(|line| !line.contains("loop-session") || line.contains("/auth/me")));
}

#[test]
fn the_device_login_polls_the_provider_then_trades_its_token() {
    let address = Arc::new(OnceLock::<String>::new());
    let known = Arc::clone(&address);
    let (url, _) = mock(move |line, body| {
        let base = known.get().unwrap();
        let grant = "urn:ietf:params:oauth:grant-type:device_code";
        let provider = format!(
            r#"{{"device_authorization_endpoint":"{base}/device","token_endpoint":"{base}/token","grant_types_supported":["{grant}"]}}"#
        );
        let started = format!(
            r#"{{"device_code":"dc","user_code":"ABCD-EFGH","verification_uri":"{base}/approve","interval":0}}"#
        );
        match line {
            "GET /.well-known/openid-configuration" => (200, provider),
            "POST /device" => (200, started),
            "POST /token" => (200, r#"{"access_token":"registre-access"}"#.to_owned()),
            _ => porte(line, body, true),
        }
    });
    address.set(url.clone()).unwrap();
    let home = tempfile::tempdir().unwrap();
    fs::create_dir_all(home.path().join("diapo")).unwrap();
    fs::write(home.path().join("diapo/config.yml"), "# keep me\ntoken: old\n").unwrap();
    let args = ["login", "--device", "--no-browser", "--server", &url, "--json"];
    let login = bin::command(home.path(), home.path(), &args).env("DIAPO_ISSUER", &url).output().unwrap();
    assert!(login.status.success(), "{}", text(&login.stderr));
    assert!(text(&login.stderr).contains(&format!("Approve this sign-in at {url}/approve (code ABCD-EFGH)")));
    let answer: serde_json::Value = serde_json::from_slice(&login.stdout).unwrap();
    assert_eq!((answer["server"].as_str(), answer["email"].as_str()), (Some(url.as_str()), Some("ada@example.com")));
    assert!(!text(&login.stdout).contains("device-session") && !text(&login.stderr).contains("device-session"));
    assert_eq!(stored(home.path()).0, format!("# keep me\ntoken: device-session\nserver_url: {url}\n"));
}

#[test]
fn login_explains_a_server_without_sso_or_without_device_sign_in() {
    let (plain, _) = mock(|_, _| (200, r#"{"sso_only":false,"oidc_enabled":false}"#.to_owned()));
    let home = tempfile::tempdir().unwrap();
    let no_sso = bin::command(home.path(), home.path(), &["login", &plain]).output().unwrap();
    assert_eq!(no_sso.status.code(), Some(1));
    assert!(text(&no_sso.stderr).contains("has no single sign-on"), "{}", text(&no_sso.stderr));
    let (url, _) = mock(|line, body| porte(line, body, false));
    let no_device = bin::command(home.path(), home.path(), &["login", &url, "--device"]).output().unwrap();
    assert_eq!(no_device.status.code(), Some(1));
    assert!(text(&no_device.stderr).contains("run `diapo login` without --device"), "{}", text(&no_device.stderr));
    assert!(!home.path().join("diapo/config.yml").exists());
}
