//! The tools themselves: the same service functions as REST, the same `rev` rule, the same event.

mod common;

use common::fixture::SOURCE;
use common::mcp::{Served, args, refused, served};
use serde_json::{Value, json};
use tronc::mcptest::{self, Client, ClientOptions, call, connect, decode};

async fn client(served: &Served, scopes: &str) -> Client {
    connect(&served.url, &format!("{}|{scopes}", served.person.user_id), ClientOptions::default()).await
}

async fn created(served: &Served, client: &Client) -> String {
    let input = json!({"title": "Plan", "source": SOURCE});
    let deck: Value = decode(&call(client, "diapo_create_deck", args(input)).await);
    let id = deck["id"].as_str().unwrap().to_owned();
    assert_eq!(
        (deck["rev"].clone(), deck["scenes"].clone(), deck["title"].clone()),
        (json!(1), json!(2), json!("Plan"))
    );
    assert_eq!(deck["url"], json!(format!("{}/decks/{id}", served.url.trim_end_matches("/mcp"))));
    id
}

#[tokio::test]
async fn an_agent_creates_reads_edits_and_rewrites_a_deck_under_the_rev_rule() {
    let Some(served) = served("diapo_mcp_tools").await else { return };
    let client = client(&served, "diapo.read,diapo.write").await;
    let id = created(&served, &client).await;
    let read: Value = decode(&call(&client, "diapo_get_deck", args(json!({"deck_id": id}))).await);
    assert_eq!(
        (read["source"].clone(), read["rev"].clone(), read["truncated"].clone()),
        (json!(SOURCE), json!(1), json!(false))
    );
    let edit = |old: &str, rev: i64| json!({"deck_id": id, "old_string": old, "new_string": "Up 15%", "rev": rev});
    assert!(refused(&client, "diapo_edit_deck", edit("Up 99%", 1)).await.contains("not in the source"));
    assert!(refused(&client, "diapo_edit_deck", edit("Q3 review", 1)).await.contains("2 times"));
    assert!(refused(&client, "diapo_edit_deck", edit("Up 12%", 7)).await.contains("is at rev 1, not 7"));
    let edited: Value = decode(&call(&client, "diapo_edit_deck", args(edit("Up 12%", 1))).await);
    assert_eq!((edited["rev"].clone(), edited["url"].clone()), (json!(2), read["url"].clone()));
    let write = |source: &str, rev: i64| json!({"deck_id": id, "source": source, "rev": rev});
    assert!(refused(&client, "diapo_write_deck", write("# Lost\n", 1)).await.contains("is at rev 2, not 1"));
    let invalid = refused(&client, "diapo_write_deck", write("---\ncolour: red\n---\n\n# A\n", 2)).await;
    assert!(invalid.contains("line 2: unknown key `colour`") && invalid.contains("diapo_format_guide"), "{invalid}");
    let written: Value = decode(&call(&client, "diapo_write_deck", args(write("# Whole\n", 2))).await);
    assert_eq!((written["rev"].clone(), written["scenes"].clone()), (json!(3), json!(1)));
    let origins: Vec<String> =
        sqlx::query_scalar("SELECT origin FROM deck_revisions WHERE deck_id = $1::uuid ORDER BY rev")
            .bind(&id)
            .fetch_all(&served.app.pool)
            .await
            .unwrap();
    assert_eq!(origins, ["mcp", "mcp", "mcp"]);
    let rest = served.app.call("GET", &format!("/api/decks/{id}"), Some(&served.person), None).await.json();
    assert_eq!((rest["source"].clone(), rest["rev"].clone()), (json!("# Whole\n"), json!(3)));
}

#[tokio::test]
async fn an_agent_lists_decks_and_reads_the_guide() {
    let Some(served) = served("diapo_mcp_reads").await else { return };
    let client = client(&served, "diapo.read,diapo.write").await;
    let id = created(&served, &client).await;
    let stranger = served.app.person("stranger").await;
    let theirs = common::fixture::id(&common::fixture::deck(&served.app, &stranger, None, SOURCE).await);
    let list: Value = decode(&call(&client, "diapo_list_decks", args(json!({}))).await);
    assert_eq!((list["total"].clone(), list["decks"][0]["id"].clone()), (json!(1), json!(id)));
    assert!(list["decks"][0]["url"].as_str().unwrap().ends_with(&format!("/decks/{id}")));
    let hidden = "Deck not found. diapo_list_decks lists deck ids.";
    assert_eq!(refused(&client, "diapo_get_deck", json!({"deck_id": theirs})).await, hidden);
    let write = json!({"deck_id": theirs, "source": "# x\n", "rev": 1});
    assert_eq!(refused(&client, "diapo_write_deck", write).await, hidden);
    let guide: Value = decode(&call(&client, "diapo_format_guide", args(json!({}))).await);
    assert_eq!(guide["guide"], json!(scene::FORMAT_GUIDE));
}

#[tokio::test]
async fn a_read_token_reads_and_is_refused_every_write_tool() {
    let Some(served) = served("diapo_mcp_scopes").await else { return };
    let reader = client(&served, "diapo.read").await;
    let list: Value = decode(&call(&reader, "diapo_list_decks", args(json!({}))).await);
    assert_eq!(list["total"], json!(0));
    let token = format!("{}|diapo.read", served.person.user_id);
    for tool in ["diapo_create_deck", "diapo_write_deck", "diapo_edit_deck", "diapo_export", "diapo_delete_deck"] {
        let body =
            json!({"jsonrpc": "2.0", "id": 1, "method": "tools/call", "params": {"name": tool, "arguments": {}}});
        let (status, headers) = mcptest::post(&served.url, &token, &body.to_string(), &[]).await;
        let challenge = headers.get("www-authenticate").and_then(|value| value.to_str().ok()).unwrap_or_default();
        assert_eq!(status, 403, "{tool}");
        assert!(challenge.contains("insufficient_scope") && challenge.contains("diapo.write"), "{tool}: {challenge}");
    }
    let decks: i64 = sqlx::query_scalar("SELECT count(*) FROM decks").fetch_one(&served.app.pool).await.unwrap();
    assert_eq!(decks, 0);
}

#[tokio::test]
async fn a_delete_waits_for_the_person_and_only_their_browser_session_confirms() {
    let Some(served) = served("diapo_mcp_delete").await else { return };
    let client = client(&served, "diapo.read,diapo.write").await;
    let id = created(&served, &client).await;
    let pending: Value = decode(&call(&client, "diapo_delete_deck", args(json!({"deck_id": id}))).await);
    assert_eq!(pending["status"], json!("pending_confirmation"));
    let action = pending["confirm_url"].as_str().unwrap().rsplit('=').next().unwrap().to_owned();
    let path = format!("/api/mcp/pending/{action}/confirm");
    let deck = format!("/api/decks/{id}");
    let live = || served.app.call("GET", &deck, Some(&served.person), None);
    assert_eq!(live().await.json()["trashed_at"], Value::Null);
    let bearer = served.app.call("POST", &path, Some(&served.person), None).await;
    assert_eq!(bearer.status, 403, "{}", bearer.text());
    let login = common::http::browser_login(&served.app.kits.sessions, served.person.user_id).await;
    let router = &served.app.router;
    assert_eq!(common::http::without_csrf(router, "POST", &path, &login).await.status, 403);
    let list = common::http::with_cookie(router, "GET", "/api/mcp/pending", &login).await.json();
    assert_eq!((list["actions"][0]["id"].clone(), list["actions"][0]["target"].clone()), (json!(action), json!(id)));
    let done = common::http::with_cookie(router, "POST", &path, &login).await;
    assert_eq!(done.json(), json!({"status": "done"}), "{}", done.text());
    assert!(live().await.json()["trashed_at"].is_string());
    assert!(refused(&client, "diapo_delete_deck", json!({"deck_id": id})).await.contains("already in the trash"));
}

#[tokio::test]
async fn an_agent_exports_html_and_pptx_to_a_signed_link_and_is_told_pdf_and_mp4_wait() {
    let Some(served) = served("diapo_mcp_export").await else { return };
    let client = client(&served, "diapo.read,diapo.write").await;
    let id = created(&served, &client).await;
    let stranger = served.app.person("stranger").await;
    let theirs = common::fixture::id(&common::fixture::deck(&served.app, &stranger, None, SOURCE).await);
    let hidden = "Deck not found. diapo_list_decks lists deck ids.";
    for format in ["pdf", "mp4"] {
        let message = refused(&client, "diapo_export", json!({"deck_id": id, "format": format})).await;
        assert!(message.contains("not available yet"), "{message}");
    }
    for (format, magic) in [("html", &b"<!doctype html>"[..]), ("pptx", b"PK")] {
        let made: Value = decode(&call(&client, "diapo_export", args(json!({"deck_id": id, "format": format}))).await);
        assert_eq!(
            (made["status"].clone(), made["file_name"].clone()),
            (json!("done"), json!(format!("Plan.{format}")))
        );
        let link = made["download_url"].as_str().unwrap();
        let origin = served.url.trim_end_matches("/mcp");
        assert!(link.starts_with(&format!("{origin}/api/exports/")) && link.contains("/file?token="), "{link}");
        let file = served.app.call("GET", link.trim_start_matches(origin), None, None).await;
        assert!(file.status == 200 && file.bytes.starts_with(magic), "{format}: {}", file.status);
    }
    let theirs_export = refused(&client, "diapo_export", json!({"deck_id": theirs, "format": "html"})).await;
    assert_eq!(theirs_export, hidden);
    let unknown = call(&client, "diapo_export", args(json!({"deck_id": id, "format": "gif"}))).await;
    assert_eq!(unknown.is_error, Some(true));
}
