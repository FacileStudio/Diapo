//! Joins one-page PDFs into one file, in order.

use std::collections::BTreeMap;

use lopdf::{Dictionary, Document, Object, ObjectId};

use crate::Error;

fn failed(error: impl std::fmt::Display) -> Error {
    Error::Chrome(format!("the PDF pages could not be joined: {error}"))
}

/// Every object of `files` under ids that do not collide, and their pages in order.
fn gather(files: &[Vec<u8>]) -> Result<(BTreeMap<ObjectId, Object>, Vec<ObjectId>), Error> {
    let (mut objects, mut pages, mut next) = (BTreeMap::new(), Vec::new(), 1);
    for bytes in files {
        let mut document = Document::load_mem(bytes).map_err(failed)?;
        document.renumber_objects_with(next);
        next = document.max_id + 1;
        pages.extend(document.get_pages().into_values());
        objects.extend(document.objects);
    }
    Ok((objects, pages))
}

fn kind(object: &Object) -> Vec<u8> {
    let name = object.as_dict().ok().and_then(|dict| dict.get(b"Type").ok()).and_then(|name| name.as_name().ok());
    name.map(<[u8]>::to_vec).unwrap_or_default()
}

/// One PDF holding the pages of `files`, in order. Each file keeps its own fonts and pictures.
pub(crate) fn join(files: &[Vec<u8>]) -> Result<Vec<u8>, Error> {
    let (objects, pages) = gather(files)?;
    let mut document = Document::with_version("1.5");
    let (tree, catalog) = (document.new_object_id(), document.new_object_id());
    let offset = catalog.0;
    let moved = |id: ObjectId| (id.0 + offset, id.1);
    for (id, mut object) in objects {
        let structural = [&b"Catalog"[..], b"Pages", b"Outlines", b"Outline"].contains(&kind(&object).as_slice());
        if structural {
            continue;
        }
        shift(&mut object, offset);
        if let (b"Page", Ok(page)) = (kind(&object).as_slice(), object.as_dict_mut()) {
            page.set("Parent", tree);
        }
        document.objects.insert(moved(id), object);
    }
    let kids: Vec<Object> = pages.iter().map(|id| Object::Reference(moved(*id))).collect();
    roots(&mut document, tree, catalog, kids);
    document.max_id = document.objects.keys().map(|id| id.0).max().unwrap_or(catalog.0);
    let mut out = Vec::new();
    document.save_to(&mut out).map_err(failed)?;
    Ok(out)
}

/// Adds `offset` to every reference inside `object`.
fn shift(object: &mut Object, offset: u32) {
    match object {
        Object::Reference(id) => id.0 += offset,
        Object::Array(items) => items.iter_mut().for_each(|item| shift(item, offset)),
        Object::Dictionary(dict) => dict.iter_mut().for_each(|(_, value)| shift(value, offset)),
        Object::Stream(stream) => stream.dict.iter_mut().for_each(|(_, value)| shift(value, offset)),
        _ => {}
    }
}

/// The page tree holding `kids` and the catalog that names it.
fn roots(document: &mut Document, tree: ObjectId, catalog: ObjectId, kids: Vec<Object>) {
    let mut root = Dictionary::new();
    root.set("Type", "Pages");
    root.set("Count", kids.len() as i64);
    root.set("Kids", kids);
    document.objects.insert(tree, Object::Dictionary(root));
    let mut entry = Dictionary::new();
    entry.set("Type", "Catalog");
    entry.set("Pages", tree);
    document.objects.insert(catalog, Object::Dictionary(entry));
    document.trailer.set("Root", catalog);
}
