//! `diapo push`: the folder's source over the revision it was pulled at, then its assets.

use std::fs;

use scene::Project;
use serde_json::json;

use crate::api::{ApiError, Client};
use crate::cli::{Globals, PushArgs};
use crate::commands::{linked_session, print_json, remote_assets, session};
use crate::failure::{Failure, Outcome};
use crate::project::Folder;
use crate::sync::plan::{self, PushPlan};
use crate::sync::{Link, asset_path, local_assets, sha256};
use crate::{diagnose, mime, ui};

fn create(globals: &Globals, folder: &Folder, project: &Project) -> Outcome<(Client, Link)> {
    let client = session(globals.server.as_deref())?;
    let title = Some(project.config().title.clone()).filter(|title| !title.is_empty()).unwrap_or_else(|| folder.name());
    let deck = client.create_deck(&title)?;
    let rev = match deck.rev {
        Some(rev) => rev,
        None => client.deck(&deck.id)?.rev,
    };
    ui::step(&format!("Created deck {}", deck.id));
    let link = Link { deck: deck.id, server: client.server().to_owned(), rev, ..Link::default() };
    link.save(&folder.dir)?;
    Ok((client, link))
}

fn target(args: &PushArgs, globals: &Globals, folder: &Folder, project: &Project) -> Outcome<(Client, Link)> {
    match Link::load(&folder.dir)? {
        Some(link) => Ok((linked_session(globals.server.as_deref(), &link)?, link)),
        None if args.create => create(globals, folder, project),
        None => {
            let hint = "run `diapo push --create` to make one, or `diapo pull <id>` to fetch one";
            Err(Failure::new("not_linked", format!("{} is not linked to a deck — {hint}", folder.dir.display())))
        }
    }
}

fn refused(error: ApiError, rev: i64) -> Failure {
    if !error.is_conflict() {
        return error.into();
    }
    Failure::new(
        "conflict",
        format!(
            "the deck changed on the server since revision {rev} — run `diapo pull` to fetch it, \
             or `diapo push --force` to overwrite it"
        ),
    )
}

fn assets(client: &Client, folder: &Folder, link: &mut Link) -> Outcome<PushPlan> {
    let local = local_assets(&folder.dir)?;
    let remote = remote_assets(client, &link.deck)?;
    let plan = plan::push(&link.assets, &local, &remote.hashes);
    for name in &plan.upload {
        let path = asset_path(&folder.dir, name);
        let kind = mime::of(&path).split(';').next().unwrap_or_default();
        client.upload_asset(&link.deck, name, kind, &fs::read(&path)?)?;
    }
    for name in &plan.delete {
        client.delete_asset(&link.deck, remote.spelled.get(name).unwrap_or(name))?;
    }
    link.assets = local;
    Ok(plan)
}

fn report(folder: &Folder, link: &Link, plan: &PushPlan) {
    if ui::json() {
        let (uploaded, deleted) = (&plan.upload, &plan.delete);
        let assets = json!({ "uploaded": uploaded, "deleted": deleted, "remote_only": plan.remote_only });
        return print_json(&json!({ "deck": link.deck, "rev": link.rev, "assets": assets }));
    }
    plan.upload.iter().for_each(|name| println!("\u{2191} {name}"));
    plan.delete.iter().for_each(|name| println!("- {name}"));
    ui::success(&format!("Pushed {} to deck {} (revision {})", folder.label(), link.deck, link.rev));
    if !plan.remote_only.is_empty() {
        ui::hint(&format!("{} only on the server, run `diapo pull` to fetch", plan.remote_only.join(", ")));
    }
}

/// Validates the deck, writes its source over the linked revision, then syncs the assets. A
/// stale revision stops the push before anything is written.
pub fn run(args: &PushArgs, globals: &Globals) -> Outcome {
    let folder = Folder::locate(args.dir.as_deref())?;
    let (source, project) = diagnose::require(&folder)?;
    let (client, mut link) = target(args, globals, &folder, &project)?;
    let rev = if args.force { client.deck(&link.deck)?.rev } else { link.rev };
    link.rev = client.put_source(&link.deck, &source, rev).map_err(|error| refused(error, rev))?;
    link.source_sha256 = sha256(source.as_bytes());
    link.save(&folder.dir)?;
    let plan = assets(&client, &folder, &mut link)?;
    link.save(&folder.dir)?;
    report(&folder, &link, &plan);
    Ok(())
}
