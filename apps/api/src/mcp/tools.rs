use tronc::mcp::{Annotations, Confirmations, Tool, ToolSpec, tool};

use super::{Context, delete, export, read, write};

const LIST: &str = "The decks you reach, newest first, 50 at a time by default: your personal decks and those \
of your spaces, or one space with space_id. Call it to find a deck id; pass next_cursor as cursor for the next \
window.";
const GET: &str = "One deck: its whole source as a planche projection (each slide is a top-level frame \
block holding positioned shapes), its rev, its slide count and its asset names. Call it before any write: \
diapo_write_deck and diapo_edit_deck need the rev it answers.";
const CREATE: &str = "A new deck at rev 1 from a planche projection, or from a source in the older scene markdown, \
which is converted to planche. Call it once per deck: each call adds another deck. An invalid source is refused \
with the line of each problem.";
const WRITE: &str = "Replaces the whole source of a deck with a planche projection (scene markdown is still \
accepted and converted). Call it to rewrite a deck; it needs the rev diapo_get_deck answered and is refused, \
changing nothing, when the deck moved on or the source is invalid. People editing the deck see the change live. \
The old source stays in the deck's revisions.";
const EDIT: &str = "Replaces one exact string in a deck's source, the planche projection diapo_get_deck answers. \
Call it for a small change. old_string must be in the source exactly once: zero or several matches are refused, \
so add surrounding text until it names one place. It needs the rev diapo_get_deck answered.";
const GUIDE: &str = "The deck format reference: the planche projection a deck is stored as (front matter, frame \
blocks as slides, shapes, steps, timing, assets), then the older scene markdown that writes still accept. Call it \
once before you write or edit a deck.";
const EXPORT: &str = "Exports a deck to a file and answers a download link that works for one hour without \
signing in: html (one offline file), pptx (editable PowerPoint), pdf (a page per slide, or per step with steps) \
or mp4 (a video). It waits up to 30 seconds; a slower export answers its job id: call again with job_id to \
read it. Without job_id each call builds a new file from the deck as it is now.";
const DELETE: &str = "Moves a deck to the trash, where it stays 30 days before it is deleted for good. The \
person confirms first and nothing changes until they do.";

fn spec(name: &str, title: &str, description: &str, annotations: Annotations) -> ToolSpec {
    ToolSpec { name: name.into(), title: title.into(), description: description.into(), annotations: Some(annotations) }
}

fn reads(ctx: &Context) -> Vec<Tool> {
    let (a, b) = (ctx.clone(), ctx.clone());
    vec![
        tool(spec("diapo_list_decks", "List decks", LIST, Annotations::READ), move |call, input| {
            read::list(a.clone(), call, input)
        }),
        tool(spec("diapo_get_deck", "Read a deck", GET, Annotations::READ), move |call, input| {
            read::get(b.clone(), call, input)
        }),
        tool(spec("diapo_format_guide", "Read the format guide", GUIDE, Annotations::READ), read::guide),
    ]
}

fn writes(ctx: &Context) -> Vec<Tool> {
    let (a, b, c, d) = (ctx.clone(), ctx.clone(), ctx.clone(), ctx.clone());
    vec![
        tool(spec("diapo_create_deck", "Create a deck", CREATE, Annotations::ADD), move |call, input| {
            write::create(a.clone(), call, input)
        }),
        tool(spec("diapo_write_deck", "Rewrite a deck", WRITE, Annotations::OVERWRITE), move |call, input| {
            write::write(b.clone(), call, input)
        }),
        tool(spec("diapo_edit_deck", "Edit a deck", EDIT, Annotations::OVERWRITE), move |call, input| {
            write::edit(c.clone(), call, input)
        }),
        tool(spec("diapo_export", "Export a deck", EXPORT, Annotations::ADD), move |call, input| {
            export::export(d.clone(), call, input)
        }),
    ]
}

/// The eight tools.
pub fn all(ctx: &Context, confirmations: &Confirmations) -> Vec<Tool> {
    let (context, confirm) = (ctx.clone(), confirmations.clone());
    let mut tools = reads(ctx);
    tools.extend(writes(ctx));
    tools.push(tool(spec("diapo_delete_deck", "Delete a deck", DELETE, Annotations::REMOVE), move |call, input| {
        delete::delete(context.clone(), confirm.clone(), call, input)
    }));
    tools
}
