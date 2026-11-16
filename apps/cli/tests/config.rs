//! The config file editor, and what a person may type for a server or a deck.

use diapo_cli::config::{flat, site};
use diapo_cli::url::deck_reference;

#[test]
fn the_config_file_is_edited_line_by_line() {
    let text = "# my settings\nserver_url: \"https://diapo.example/api\"\ntheme: dark # kept\n";
    assert_eq!(flat::get(text, "server_url").as_deref(), Some("https://diapo.example/api"));
    assert_eq!(flat::get(text, "theme").as_deref(), Some("dark"));
    assert_eq!(flat::get(text, "token"), None);
    let logged_in = flat::set(&flat::set(text, "server_url", "https://diapo.example"), "token", "s3cret");
    assert_eq!(logged_in, "# my settings\nserver_url: https://diapo.example\ntheme: dark # kept\ntoken: s3cret\n");
    assert_eq!(
        flat::remove(&logged_in, "token"),
        "# my settings\nserver_url: https://diapo.example\ntheme: dark # kept\n"
    );
    assert_eq!(flat::remove("token: x\n", "token"), "");
}

#[test]
fn servers_and_deck_references_are_normalized() {
    for typed in
        ["https://diapo.example", "https://diapo.example/", "https://diapo.example/api", " https://diapo.example/api/ "]
    {
        assert_eq!(site(typed), "https://diapo.example", "{typed}");
    }
    assert_eq!(deck_reference("d1"), (None, "d1".to_owned()));
    let from_url = deck_reference("https://diapo.example/d/3f2a?present=1");
    assert_eq!(from_url, (Some("https://diapo.example".to_owned()), "3f2a".to_owned()));
}
