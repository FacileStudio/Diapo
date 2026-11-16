//! Registre's side: a directory snapshot mirrored into spaces, and a person erased.

mod common;

use api::auth::{AppData, SpacesApply};
use common::directory::{base, snapshot, space};
use common::fixture::{self, SOURCE};
use porte::directory::PersonDeleted;

#[tokio::test]
async fn a_snapshot_replaces_spaces_and_members_once_per_version() {
    let Some(pool) = common::pool("diapo_directory").await else { return };
    let apply = SpacesApply::new(pool.clone());
    let base = base();
    assert!(
        apply.apply_snapshot(&snapshot(base, vec![space("a", &[("s1", "owner")]), space("b", &[])])).await.unwrap()
    );
    assert!(!apply.apply_snapshot(&snapshot(base, vec![space("x", &[])])).await.unwrap());
    assert!(apply.apply_snapshot(&snapshot(base + 1, vec![space("a", &[("s2", "member")])])).await.unwrap());
    let spaces: Vec<String> = sqlx::query_scalar("SELECT id FROM spaces WHERE deleted_at IS NULL ORDER BY id")
        .fetch_all(&pool)
        .await
        .unwrap();
    assert_eq!(spaces, ["a"]);
    let members: Vec<String> = sqlx::query_scalar("SELECT subject FROM space_members").fetch_all(&pool).await.unwrap();
    assert_eq!(members, ["s2"]);
    assert!(apply.apply_snapshot(&snapshot(base + 2, vec![])).await.is_err());
}

#[tokio::test]
async fn a_space_that_leaves_the_directory_has_its_decks_trashed_not_deleted() {
    let Some(pool) = common::pool("diapo_directory_retire").await else { return };
    let app = common::app(pool.clone()).await;
    let roles = fixture::roles(&app).await;
    let deck = fixture::id(&fixture::deck(&app, &roles.owner, Some(&roles.space), SOURCE).await);
    let kept = space(&roles.other_space, &[(&roles.outsider.subject, "owner")]);
    assert!(SpacesApply::new(pool.clone()).apply_snapshot(&snapshot(base(), vec![kept])).await.unwrap());
    assert_eq!(app.call("GET", &format!("/api/decks/{deck}"), Some(&roles.owner), None).await.status, 404);
    let trashed: bool = sqlx::query_scalar("SELECT trashed_at IS NOT NULL FROM decks WHERE id = $1::uuid")
        .bind(&deck)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert!(trashed);
}

#[tokio::test]
async fn erasing_a_person_removes_their_decks_memberships_and_account() {
    let Some(pool) = common::pool("diapo_erase").await else { return };
    let app = common::app(pool.clone()).await;
    let roles = fixture::roles(&app).await;
    let personal = fixture::id(&fixture::deck(&app, &roles.member, None, SOURCE).await);
    let shared = fixture::id(&fixture::deck(&app, &roles.member, Some(&roles.space), SOURCE).await);
    AppData::new(pool.clone(), app.state.data_dir.as_ref().clone())
        .on_person_deleted(common::ISSUER, &roles.member.subject)
        .await
        .unwrap();
    let ids = [uuid::Uuid::parse_str(&personal).unwrap(), uuid::Uuid::parse_str(&shared).unwrap()];
    let left: Vec<uuid::Uuid> =
        sqlx::query_scalar("SELECT id FROM decks WHERE id = ANY($1)").bind(&ids[..]).fetch_all(&pool).await.unwrap();
    assert_eq!(left, [ids[1]]);
    let users: i64 = sqlx::query_scalar("SELECT count(*) FROM porte_users WHERE id = $1")
        .bind(roles.member.user_id)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(users, 0);
    assert_eq!(app.call("GET", &format!("/api/decks/{shared}"), Some(&roles.owner), None).await.status, 200);
    assert_eq!(app.call("GET", "/api/spaces", Some(&roles.member), None).await.status, 401);
}
