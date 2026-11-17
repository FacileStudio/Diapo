use serde_json::{Value, json};

use super::{App, Person};

pub const SOURCE: &str = "---\ntitle: Q3 review\n---\n\n# Q3 review\n\n---\n\n## Revenue\n\n- Up 12% {step=1}\n";

pub struct Roles {
    pub owner: Person,
    pub admin: Person,
    pub member: Person,
    pub outsider: Person,
    pub space: String,
    pub other_space: String,
}

pub async fn roles(app: &App) -> Roles {
    let (owner, admin, member, outsider) = (
        app.person("owner").await,
        app.person("admin").await,
        app.person("member").await,
        app.person("outsider").await,
    );
    let space = app.space(&[(&owner, "owner"), (&admin, "admin"), (&member, "member")]).await;
    let other_space = app.space(&[(&outsider, "owner")]).await;
    Roles { owner, admin, member, outsider, space, other_space }
}

pub async fn deck(app: &App, who: &Person, space: Option<&str>, source: &str) -> Value {
    let mut body = json!({"source": source});
    if let Some(space) = space {
        body["space_id"] = json!(space);
    }
    let answer = app.call("POST", "/api/decks", Some(who), Some(body)).await;
    assert_eq!(answer.status, 201, "{}", answer.text());
    answer.json()
}

pub fn projected(source: &str) -> String {
    planche::projection::write(&scene::planche::convert(source).unwrap())
}

pub fn id(deck: &Value) -> String {
    deck["id"].as_str().unwrap().to_owned()
}

pub const NOTED: &str = "---\ntitle: Launch \"plan\" <b>\n---\n\n# Launch\n\n![](assets/dot.png)\n\n<!-- SECRET NOTE -->\n\n---\n\n\
## Next\n\n- One {step=1}\n- Two {step=2}\n\n<!-- ANOTHER SECRET -->\n";

pub async fn shared(app: &App, who: &Person, space: Option<&str>) -> (String, String) {
    let id = id(&deck(app, who, space, NOTED).await);
    let link = app.call("POST", &format!("/api/decks/{id}/share"), Some(who), None).await;
    assert_eq!(link.status, 201, "{}", link.text());
    let url = link.json()["url"].as_str().unwrap().to_owned();
    (id, url.rsplit('/').next().unwrap().to_owned())
}

pub async fn serve(app: &App) -> String {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let router = app.router.clone().into_make_service_with_connect_info::<std::net::SocketAddr>();
    tokio::spawn(async move { axum::serve(listener, router).await.unwrap() });
    format!("ws://{addr}")
}
