//! The sign-in calls against a mock server, and the loopback listener.

#[path = "support/mock.rs"]
mod mock;

use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::thread;
use std::time::{Duration, Instant};

use diapo_cli::api::device::{self, Answer};
use diapo_cli::api::{ApiError, Client};
use diapo_cli::login::device::{Poll, read};
use diapo_cli::login::loopback;
use mock::mock;

fn porte(line: &str, body: &str) -> (u16, String) {
    match (line, body) {
        ("GET /api/auth/config", _) => (200, r#"{"sso_only":true,"oidc_enabled":true}"#.to_owned()),
        ("POST /api/auth/oidc/exchange", r#"{"code":"one-time"}"#) => {
            (200, r#"{"user_id":"1","token":"session"}"#.to_owned())
        }
        ("POST /api/auth/oidc/exchange", _) => {
            (401, r#"{"error":{"code":"unauthorized","message":"invalid or expired code"}}"#.to_owned())
        }
        ("POST /api/auth/oidc/device/exchange", "{}") => {
            (400, r#"{"error":{"code":"bad_request","message":"empty"}}"#.to_owned())
        }
        ("POST /api/auth/oidc/device/exchange", _) => (200, r#"{"user_id":"1","token":"device-session"}"#.to_owned()),
        ("GET /api/auth/me", _) => (200, r#"{"email":"ada@example.com","name":"Ada"}"#.to_owned()),
        ("POST /api/auth/logout", _) => (200, r#"{"logged_out":true}"#.to_owned()),
        _ => (404, r#"{"error":{"code":"not_found","message":"route not found"}}"#.to_owned()),
    }
}

#[test]
fn the_porte_routes_answer_discovery_exchange_identity_and_logout() {
    let (url, seen) = mock(porte);
    let client = Client::new(&url, Some("session"));
    assert!(client.auth_config().unwrap().oidc_enabled);
    assert_eq!(client.exchange_code("one-time").unwrap(), "session");
    assert!(matches!(client.exchange_code("replayed"), Err(ApiError::Refused { status: 401, .. })));
    assert_eq!(client.me().unwrap().email, "ada@example.com");
    client.logout().unwrap();
    assert!(seen.lock().unwrap()[4].starts_with("POST /api/auth/logout [Bearer session]"));
    let link = client.login_url(4242, "n0nce");
    assert_eq!(link, format!("{url}/api/auth/oidc?flow=cli&port=4242&cli_state=n0nce"));
}

#[test]
fn the_device_exchange_is_probed_then_used() {
    let (url, _) = mock(porte);
    let client = Client::new(&url, None);
    assert!(client.device_mounted().unwrap());
    assert_eq!(client.device_exchange("registre-access").unwrap(), "device-session");
    let (bare, _) = mock(|_, _| (404, r#"{"error":{"code":"not_found","message":"route not found"}}"#.to_owned()));
    assert!(!Client::new(&bare, None).device_mounted().unwrap());
}

#[test]
fn the_device_grant_starts_and_polls_at_the_provider() {
    let (issuer, seen) = mock(|line, body| match line {
        "GET /.well-known/openid-configuration" => {
            (200, r#"{"token_endpoint":"T","grant_types_supported":[]}"#.to_owned())
        }
        "POST /device" => {
            (200, r#"{"device_code":"dc","user_code":"ABCD","verification_uri":"https://sso/device"}"#.to_owned())
        }
        "POST /token" if body.contains("device_code=dc") => (400, r#"{"error":"authorization_pending"}"#.to_owned()),
        _ => (500, String::new()),
    });
    assert_eq!(device::provider(&issuer).unwrap().token_endpoint, "T");
    assert_eq!(device::authorize(&format!("{issuer}/device")).unwrap().user_code, "ABCD");
    let answer = device::poll(&format!("{issuer}/token"), "dc").unwrap();
    assert_eq!(read(answer).unwrap(), Poll::Pending);
    let seen = seen.lock().unwrap();
    assert!(seen[1].ends_with("client_id=facile-cli&scope=openid%20email%20profile"), "{seen:?}");
    assert!(seen[2].contains("grant_type=urn%3Aietf%3Aparams%3Aoauth%3Agrant-type%3Adevice_code"), "{seen:?}");
}

#[test]
fn a_token_endpoint_answer_means_approved_wait_or_stop() {
    let failing = |error: &str| read(Answer { error: Some(error.to_owned()), ..Answer::default() });
    let approved = Answer { access_token: Some("registre".to_owned()), ..Answer::default() };
    assert_eq!(read(approved).unwrap(), Poll::Approved("registre".to_owned()));
    assert_eq!(failing("slow_down").unwrap(), Poll::SlowDown);
    assert!(failing("expired_token").unwrap_err().message.contains("expired"));
    assert!(failing("access_denied").unwrap_err().message.contains("refused"));
    assert!(failing("invalid_grant").is_err());
    assert!(read(Answer::default()).is_err());
}

fn request(port: u16, target: &str) -> String {
    let mut stream = TcpStream::connect(("127.0.0.1", port)).unwrap();
    stream.write_all(format!("GET {target} HTTP/1.1\r\nHost: localhost\r\n\r\n").as_bytes()).unwrap();
    let mut answer = String::new();
    stream.read_to_string(&mut answer).unwrap();
    answer
}

#[test]
fn the_loopback_listener_takes_only_the_callback_with_its_own_state() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    let browser = thread::spawn(move || {
        let favicon = request(port, "/favicon.ico");
        let forged = request(port, "/?code=stolen&state=not-ours");
        let ours = request(port, "/?code=one%2Dtime&state=ours");
        (favicon, forged, ours)
    });
    let code = loopback::wait(&listener, "ours", Instant::now() + Duration::from_secs(20)).unwrap();
    let (favicon, forged, ours) = browser.join().unwrap();
    assert_eq!(code, "one-time");
    assert!(favicon.starts_with("HTTP/1.1 404"), "{favicon}");
    assert!(forged.starts_with("HTTP/1.1 400"), "{forged}");
    assert!(ours.starts_with("HTTP/1.1 200") && ours.contains("Signed in"), "{ours}");
}

#[test]
fn the_loopback_listener_gives_up_at_its_deadline_and_nonces_differ() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let late = loopback::wait(&listener, "ours", Instant::now() + Duration::from_millis(120)).unwrap_err();
    assert_eq!(late.code, "timeout");
    let (first, second) = (loopback::nonce().unwrap(), loopback::nonce().unwrap());
    assert_eq!(first.len(), 32);
    assert_ne!(first, second);
}
