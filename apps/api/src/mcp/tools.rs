use tronc::mcp::{Annotations, Confirmations, Tool, ToolSpec, tool};

use super::{Context, delete, export, read, write};

const LIST: &str = "The decks you reach, newest first, 50 at a time by default: your personal decks and those \
of your spaces, or one space with space_id. Call it to find a deck id; pass next_cursor as cursor for the next \
window.";
const GET: &str = "One deck: its whole source in the scene format, its rev, its scene count and its asset \
names. Call it before any write: diapo_write_deck and diapo_edit_deck need the rev it answers.";
const CREATE: &str = "A new deck from a source in the scene format, at rev 1. Call it once per deck: each call \
adds another deck. An invalid source is refused with the line of each problem.";
const WRITE: &str = "Replaces the whole source of a deck. Call it to rewrite a deck; it needs the rev \
diapo_get_deck answered and is refused, changing nothing, when the deck moved on or the source is invalid. \
The old source stays in the deck's revisions.";
const EDIT: &str = "Replaces one exact string in a deck's source. Call it for a small change. old_string must \
be in the source exactly once: zero or several matches are refused, so add surrounding text until it names \
one place. It needs the rev diapo_get_deck answered.";
const GUIDE: &str = "The scene format reference: frontmatter keys, layouts, step reveals, timelines, assets and \
the errors the parser answers. Call it once before you write or edit a deck.";
const EXPORT: &str = "Exports a deck to a file: html, pdf, pptx or mp4. Exports are not available yet: the \
call answers an error and exports nothing.";
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
