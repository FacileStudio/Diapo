//! Every deck route against every role: `None` is always 404, too little is 403.

mod common;

use common::fixture::{self, Roles, SOURCE};
use common::upload;
use common::{App, Person};
use serde_json::{Value, json};

#[derive(Clone, Copy, PartialEq, PartialOrd)]
enum Need {
    Viewer,
    Editor,
    Manager,
}

struct Case {
    method: &'static str,
    path: &'static str,
    body: Option<fn() -> Value>,
    need: Need,
    trashed: bool,
    ok: u16,
}

type Row = (&'static str, &'static str, Option<fn() -> Value>, Need, u16);

fn case((method, path, body, need, ok): Row) -> Case {
    Case { method, path, body, need, trashed: false, ok }
}

fn cases() -> Vec<Case> {
    let rows: [Row; 10] = [
        ("GET", "/api/decks/{id}", None, Need::Viewer, 200),
        ("GET", "/api/decks/{id}/frame", None, Need::Viewer, 200),
        ("GET", "/api/decks/{id}/frame?scene=1", None, Need::Viewer, 200),
        ("GET", "/api/decks/{id}/assets", None, Need::Viewer, 200),
        ("GET", "/api/decks/{id}/assets/assets/dot.png", None, Need::Viewer, 200),
        ("PUT", "/api/decks/{id}/source", Some(|| json!({"source": "# New\n", "rev": 1})), Need::Editor, 200),
        ("PATCH", "/api/decks/{id}", Some(|| json!({"title": "Renamed"})), Need::Editor, 200),
        ("DELETE", "/api/decks/{id}/assets/assets/dot.png", None, Need::Editor, 204),
        ("PATCH", "/api/decks/{id}", Some(|| json!({"trashed": true})), Need::Manager, 200),
        ("PATCH", "/api/decks/{id}", Some(|| json!({"space_id": null})), Need::Manager, 200),
    ];
    let mut all: Vec<Case> = rows.into_iter().map(case).collect();
    all.push(Case { trashed: true, ..case(("DELETE", "/api/decks/{id}", None, Need::Manager, 204)) });
    all
}

async fn prepared(app: &App, roles: &Roles, trashed: bool) -> String {
    let deck = fixture::id(&fixture::deck(app, &roles.owner, Some(&roles.space), SOURCE).await);
    assert_eq!(upload::png(app, &roles.owner, &deck, "assets/dot.png").await.status, 201);
    if trashed {
        let trash = Some(json!({"trashed": true}));
        assert_eq!(app.call("PATCH", &format!("/api/decks/{deck}"), Some(&roles.owner), trash).await.status, 200);
    }
    deck
}

async fn attempt(app: &App, roles: &Roles, case: &Case, who: Option<&Person>) -> u16 {
    let deck = prepared(app, roles, case.trashed).await;
    let answer = app.call(case.method, &case.path.replace("{id}", &deck), who, case.body.map(|body| body())).await;
    answer.status.as_u16()
}

fn expected(case: &Case, have: Option<Need>) -> u16 {
    match have {
        None => 404,
        Some(have) if have < case.need => 403,
        Some(_) => case.ok,
    }
}

#[tokio::test]
async fn every_route_answers_each_role_as_its_access_allows() {
    let Some(pool) = common::pool("diapo_authz").await else { return };
    let app = common::app(pool).await;
    let roles = fixture::roles(&app).await;
    let people = [
        (&roles.owner, Some(Need::Manager)),
        (&roles.admin, Some(Need::Manager)),
        (&roles.member, Some(Need::Editor)),
        (&roles.outsider, None),
    ];
    for case in cases() {
        assert_eq!(attempt(&app, &roles, &case, None).await, 401, "anonymous {} {}", case.method, case.path);
        for (person, have) in &people {
            let got = attempt(&app, &roles, &case, Some(person)).await;
            assert_eq!(got, expected(&case, *have), "{} {} as {:?}", case.method, case.path, person.subject);
        }
    }
}

#[tokio::test]
async fn a_deck_of_another_space_or_person_does_not_exist_for_the_caller() {
    let Some(pool) = common::pool("diapo_authz_scope").await else { return };
    let app = common::app(pool).await;
    let roles = fixture::roles(&app).await;
    let shared = fixture::id(&fixture::deck(&app, &roles.owner, Some(&roles.space), SOURCE).await);
    let personal = fixture::id(&fixture::deck(&app, &roles.member, None, SOURCE).await);
    let outsider = Some(&roles.outsider);
    assert_eq!(app.call("GET", &format!("/api/decks/{shared}"), outsider, None).await.status, 404);
    assert_eq!(app.call("GET", &format!("/api/decks?space_id={}", roles.space), outsider, None).await.status, 404);
    let create = json!({"source": SOURCE, "space_id": roles.space});
    assert_eq!(app.call("POST", "/api/decks", outsider, Some(create)).await.status, 404);
    assert_eq!(app.call("GET", "/api/decks", outsider, None).await.json(), json!([]));
    for who in [&roles.owner, &roles.admin, &roles.outsider] {
        assert_eq!(app.call("GET", &format!("/api/decks/{personal}"), Some(who), None).await.status, 404);
    }
    assert_eq!(app.call("GET", "/api/decks/not-a-uuid", outsider, None).await.status, 404);
    let spaces = app.call("GET", "/api/spaces", Some(&roles.member), None).await.json();
    assert_eq!(spaces, json!({"spaces": [{"id": roles.space, "name": roles.space, "role": "member"}]}));
}

#[tokio::test]
async fn a_cookie_write_needs_the_csrf_header() {
    let Some(pool) = common::pool("diapo_authz_csrf").await else { return };
    let app = common::app(pool).await;
    let ada = app.person("ada").await;
    let id = fixture::id(&fixture::deck(&app, &ada, None, SOURCE).await);
    let login = common::http::browser_login(&app.kits.sessions, ada.user_id).await;
    let path = format!("/api/decks/{id}");
    assert_eq!(common::http::without_csrf(&app.router, "DELETE", &path, &login).await.status, 403);
    assert_eq!(common::http::with_cookie(&app.router, "GET", &path, &login).await.status, 200);
    assert_eq!(common::http::with_cookie(&app.router, "DELETE", &path, &login).await.status, 412);
}
