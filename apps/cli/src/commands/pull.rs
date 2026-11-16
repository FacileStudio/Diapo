//! `diapo pull`: a deck's source and assets into a folder, without losing local edits.

use std::fs;
use std::path::{Path, PathBuf};

use serde_json::json;

use crate::api::{Client, Deck};
use crate::cli::{Globals, PullArgs};
use crate::commands::{linked_session, print_json, remote_assets, session};
use crate::failure::{Failure, Outcome};
use crate::project::ENTRY;
use crate::sync::plan::{self, PullPlan, SourcePull};
use crate::sync::{Hashes, LINK_FILE, Link, asset_path, local_assets, sha256};
use crate::{ui, url};

/// What a pull works on: the deck as the server holds it, the folder, and the folder's link.
struct Wanted {
    client: Client,
    deck: Deck,
    dir: PathBuf,
    base: Link,
}

fn slug(title: &str, id: &str) -> String {
    let dashed = |text: &str| -> String {
        let mapped: String =
            text.chars().map(|c| if c.is_alphanumeric() { c.to_ascii_lowercase() } else { '-' }).collect();
        mapped.split('-').filter(|part| !part.is_empty()).collect::<Vec<_>>().join("-")
    };
    [dashed(title), dashed(id)].into_iter().find(|name| !name.is_empty()).unwrap_or("deck".to_owned())
}

fn linked(dir: &Path, globals: &Globals) -> Outcome<Wanted> {
    let Some(base) = Link::load(dir)? else {
        let hint = "run `diapo pull <id>`, `diapo list` shows the ids";
        return Err(Failure::new("not_linked", format!("{} is not linked to a deck — {hint}", dir.display())));
    };
    let client = linked_session(globals.server.as_deref(), &base)?;
    let deck = client.deck(&base.deck)?;
    Ok(Wanted { client, deck, dir: dir.to_owned(), base })
}

fn named(reference: &str, args: &PullArgs, globals: &Globals) -> Outcome<Wanted> {
    let (origin, id) = url::deck_reference(reference);
    let client = session(globals.server.as_deref().or(origin.as_deref()))?;
    let deck = client.deck(&id)?;
    let dir = args.dir.clone().unwrap_or_else(|| PathBuf::from(slug(&deck.title, &deck.id)));
    let fresh = Link { deck: deck.id.clone(), server: client.server().to_owned(), ..Link::default() };
    let base = Link::load(&dir)?.unwrap_or_else(|| fresh.clone());
    if (&base.deck, &base.server) != (&fresh.deck, &fresh.server) {
        let message = format!("{} is linked to another deck — pull into another folder", dir.display());
        return Err(Failure::new("other_deck", message));
    }
    Ok(Wanted { client, deck, dir, base })
}

fn wanted(args: &PullArgs, globals: &Globals) -> Outcome<Wanted> {
    match &args.deck {
        Some(folder) if args.dir.is_none() && Path::new(folder).join(LINK_FILE).is_file() => {
            linked(Path::new(folder), globals)
        }
        Some(reference) => named(reference, args, globals),
        None => linked(Path::new("."), globals),
    }
}

fn decide(wanted: &Wanted, remote: &Hashes, force: bool) -> Outcome<(SourcePull, PullPlan)> {
    let here = fs::read(wanted.dir.join(ENTRY)).ok().map(|bytes| sha256(&bytes));
    let source = plan::pull_source(&wanted.base.source_sha256, here.as_deref(), &sha256(wanted.deck.source.as_bytes()));
    let assets = plan::pull(&wanted.base.assets, &local_assets(&wanted.dir, &wanted.base.assets)?, remote);
    let changed: Vec<&str> = (source == SourcePull::LocalChanges)
        .then_some(ENTRY)
        .into_iter()
        .chain(assets.conflicts.iter().map(String::as_str))
        .collect();
    if force || changed.is_empty() {
        return Ok((source, assets));
    }
    let hint = "run `diapo push` to send them, or `diapo pull --force` to discard them";
    Err(Failure::new("local_changes", format!("{} changed here since the last sync — {hint}", changed.join(", "))))
}

fn apply(wanted: &Wanted, remote: &Hashes, source: SourcePull, assets: &PullPlan) -> Outcome<Link> {
    let (dir, deck) = (&wanted.dir, &wanted.deck);
    fs::create_dir_all(dir)?;
    if source != SourcePull::UpToDate {
        fs::write(dir.join(ENTRY), &deck.source)?;
    }
    for name in &assets.download {
        let path = asset_path(dir, name);
        fs::create_dir_all(path.parent().unwrap_or(dir))?;
        fs::write(&path, wanted.client.download_asset(&deck.id, name)?)?;
    }
    assets.remove.iter().for_each(|name| drop(fs::remove_file(asset_path(dir, name))));
    let held = local_assets(dir, remote)?.into_iter().filter(|(name, _)| remote.contains_key(name)).collect();
    let source_sha256 = sha256(deck.source.as_bytes());
    let link = Link { rev: deck.rev, source_sha256, assets: held, ..wanted.base.clone() };
    link.save(dir)?;
    Ok(link)
}

fn report(wanted: &Wanted, link: &Link, source: SourcePull, assets: &PullPlan) {
    let (dir, updated) = (wanted.dir.display().to_string(), source != SourcePull::UpToDate);
    if ui::json() {
        let assets = json!({ "downloaded": assets.download, "removed": assets.remove });
        let deck = json!({ "deck": link.deck, "title": wanted.deck.title, "rev": link.rev });
        return print_json(&json!({ "dir": dir, "source_updated": updated, "assets": assets, "deck": deck }));
    }
    if !ui::json() {
        assets.download.iter().for_each(|name| println!("\u{2193} {name}"));
        assets.remove.iter().for_each(|name| println!("- {name}"));
    }
    match updated || !assets.download.is_empty() || !assets.remove.is_empty() {
        true => ui::success(&format!("Pulled deck {} into {dir} (revision {})", link.deck, link.rev)),
        false => ui::success(&format!("{dir} is up to date (revision {})", link.rev)),
    }
}

/// Fetches the deck, refuses to overwrite local edits without `--force`, then writes the
/// source, the changed assets and the link.
pub fn run(args: &PullArgs, globals: &Globals) -> Outcome {
    let wanted = wanted(args, globals)?;
    let remote = remote_assets(&wanted.client, &wanted.deck.id)?;
    let (source, assets) = decide(&wanted, &remote, args.force)?;
    let link = apply(&wanted, &remote, source, &assets)?;
    report(&wanted, &link, source, &assets);
    Ok(())
}
