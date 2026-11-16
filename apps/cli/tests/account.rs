//! Credentials and the account commands, through the binary.

#[path = "support/bin.rs"]
mod bin;
#[path = "support/mock.rs"]
mod mock;

use std::fs;
use std::os::unix::fs::PermissionsExt;

use mock::mock;
use tempfile::TempDir;

fn text(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).into_owned()
}

fn account(line: &str, _: &str) -> (u16, String) {
    match line {
        "GET /api/auth/me" => (200, r#"{"email":"ada@example.com","name":"Ada"}"#.to_owned()),
        "POST /api/auth/logout" => (200, r#"{"logged_out":true}"#.to_owned()),
        "GET /api/decks" => (200, r#"[{"id":"d1","title":"Q3"},{"id":"d22","title":"Launch"}]"#.to_owned()),
        _ => (404, r#"{"error":{"code":"not_found","message":"route not found"}}"#.to_owned()),
    }
}

fn logged_in(server: &str) -> TempDir {
    let home = tempfile::tempdir().unwrap();
    fs::create_dir_all(home.path().join("diapo")).unwrap();
    let file = home.path().join("diapo/config.yml");
    fs::write(&file, format!("# mine\nserver_url: {server}\ntoken: s3cret\n")).unwrap();
    fs::set_permissions(&file, fs::Permissions::from_mode(0o644)).unwrap();
    home
}

#[test]
fn the_stored_token_is_tightened_and_only_ever_sent_to_its_own_server() {
    let (url, seen) = mock(account);
    let (other, other_seen) = mock(account);
    let home = logged_in(&url);
    let me = bin::command(home.path(), home.path(), &["whoami"]).output().unwrap();
    assert_eq!(text(&me.stdout), format!("ada@example.com on {url}\n"));
    let mode = fs::metadata(home.path().join("diapo/config.yml")).unwrap().permissions().mode();
    assert_eq!(mode & 0o777, 0o600);
    let elsewhere = bin::command(home.path(), home.path(), &["whoami", "--server", &other]).output().unwrap();
    assert_eq!(elsewhere.status.code(), Some(1));
    assert!(text(&elsewhere.stderr).contains(&format!("not logged in to {other}")));
    assert!(other_seen.lock().unwrap().is_empty());
    let from_env = bin::command(home.path(), home.path(), &["whoami", "--json"])
        .env("DIAPO_SERVER_URL", format!("{other}/api"))
        .env("DIAPO_TOKEN", "from-env")
        .output()
        .unwrap();
    assert!(from_env.status.success() && !text(&from_env.stdout).contains("from-env"));
    assert_eq!(other_seen.lock().unwrap()[0], "GET /api/auth/me [Bearer from-env] ");
    assert_eq!(seen.lock().unwrap().len(), 1);
}

#[test]
fn list_open_and_logout_work_from_the_stored_login() {
    let (url, seen) = mock(account);
    let home = logged_in(&url);
    let run = |args: &[&str]| bin::command(home.path(), home.path(), args).output().unwrap();
    assert_eq!(text(&run(&["list"]).stdout), "d1   Q3\nd22  Launch\n");
    assert_eq!(text(&run(&["list", "-q"]).stdout), "d1\nd22\n");
    let listed: serde_json::Value = serde_json::from_slice(&run(&["list", "--json"]).stdout).unwrap();
    assert_eq!(listed[1]["title"], "Launch");
    fs::write(home.path().join(".diapo.json"), format!(r#"{{"deck":"d1","server":"{url}","rev":1}}"#)).unwrap();
    assert_eq!(text(&run(&["open", "--print"]).stdout), format!("{url}/decks/d1\n"));
    assert!(text(&run(&["logout"]).stdout).contains("Signed out"));
    assert!(seen.lock().unwrap().last().unwrap().starts_with("POST /api/auth/logout [Bearer s3cret]"));
    let kept = fs::read_to_string(home.path().join("diapo/config.yml")).unwrap();
    assert_eq!(kept, format!("# mine\nserver_url: {url}\n"));
    let again = run(&["logout"]);
    assert!(again.status.success() && text(&again.stdout).contains("Not logged in"));
    assert_eq!(run(&["list"]).status.code(), Some(1));
}
