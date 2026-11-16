//! `/mcp` against tronc's conformance kit, with `diapo_delete_deck` confirmed.

mod common;

use std::sync::Arc;

use common::fixture::SOURCE;
use common::mcp::{Served, args, served};
use serde_json::json;
use tronc::mcptest::{self, ClientOptions, ConfirmCase, Target, connect};

async fn runs(state: api::state::AppState, id: String) -> usize {
    let trashed: bool = sqlx::query_scalar("SELECT trashed_at IS NOT NULL FROM decks WHERE id = $1::uuid")
        .bind(&id)
        .fetch_one(&state.pool)
        .await
        .unwrap();
    usize::from(trashed)
}

async fn fresh(state: api::state::AppState, router: axum::Router, token: String) -> Target {
    let deck = common::send(&router, "POST", "/api/decks", Some(&token), Some(json!({"source": SOURCE}))).await.json();
    let id = deck["id"].as_str().unwrap().to_owned();
    let target = args(json!({"deck_id": id}));
    let count = move || -> mcptest::Fut<usize> { Box::pin(runs(state.clone(), id.clone())) };
    Target { args: target, runs: Arc::new(count) }
}

fn delete_case(served: &Served) -> ConfirmCase {
    let (state, router, token) = (served.app.state.clone(), served.app.router.clone(), served.person.token.clone());
    let setup = move || -> mcptest::Fut<Target> { Box::pin(fresh(state.clone(), router.clone(), token.clone())) };
    ConfirmCase { tool: "diapo_delete_deck".into(), setup: Arc::new(setup) }
}

#[tokio::test]
async fn the_mcp_endpoint_passes_tronc_conformance_with_the_delete_confirmed() {
    let Some(served) = served("diapo_mcp_kit").await else { return };
    let user = served.person.user_id;
    let token = Arc::new(move |scopes: &[&str]| {
        let scopes = if scopes.is_empty() { "diapo.read,diapo.write".to_owned() } else { scopes.join(",") };
        format!("{user}|{scopes}")
    });
    let confirm = vec![delete_case(&served)];
    mcptest::run(&mcptest::Config { url: served.url.clone(), token, confirm }).await;
}

#[tokio::test]
async fn the_eight_tools_are_listed_under_their_fixed_names() {
    let Some(served) = served("diapo_mcp_names").await else { return };
    let token = format!("{}|diapo.read,diapo.write", served.person.user_id);
    let client = connect(&served.url, &token, ClientOptions::default()).await;
    let tools = client.list_tools(None).await.unwrap().tools;
    let mut names: Vec<String> = tools.iter().map(|tool| tool.name.to_string()).collect();
    names.sort();
    let expected = [
        "diapo_create_deck",
        "diapo_delete_deck",
        "diapo_edit_deck",
        "diapo_export",
        "diapo_format_guide",
        "diapo_get_deck",
        "diapo_list_decks",
        "diapo_write_deck",
    ];
    assert_eq!(names, expected);
    let export = tools.iter().find(|tool| tool.name == "diapo_export").unwrap();
    let schema = serde_json::to_value(&export.input_schema).unwrap().to_string();
    for format in ["html", "pdf", "pptx", "mp4"] {
        assert!(schema.contains(&format!("\"{format}\"")), "{schema}");
    }
}
