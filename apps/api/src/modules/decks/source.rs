use planche::Document;
use tronc::Error;

use super::error::DeckError;

/// The largest deck source, in bytes, unless `SOURCE_MAX_BYTES` says otherwise. It caps the
/// source a write sends and the projection the deck is stored as.
pub const SOURCE_MAX_BYTES: usize = 512 << 10;

/// The largest JSON body of a route that carries a source: room for the default cap once every
/// byte is escaped, and for the envelope around it.
pub const BODY_MAX_BYTES: usize = 4 << 20;

/// A source ready to store: the planche document, its projection and the compiled project.
pub struct Prepared {
    /// The planche document.
    pub document: Document,
    /// Its projection: what `decks.source` holds.
    pub source: String,
    /// The projection, parsed for the readers.
    pub project: scene::Project,
}

fn capped(length: usize, max: usize) -> Result<(), DeckError> {
    match length > max {
        true => Err(Error::too_large(format!("the source is over the {max}-byte limit")).into()),
        false => Ok(()),
    }
}

/// Reads `source` in either format, within `max` bytes: `413` past the cap, the typed problems
/// with their lines when it does not read. A legacy scene source is converted to planche.
pub fn checked(source: &str, max: usize) -> Result<Prepared, DeckError> {
    capped(source.len(), max)?;
    let document = scene::planche::convert(source).map_err(DeckError::Invalid)?;
    prepared(document, max)
}

/// `document` as it would be stored: its projection must fit in `max` bytes and compile.
pub fn prepared(document: Document, max: usize) -> Result<Prepared, DeckError> {
    let source = planche::projection::write(&document);
    capped(source.len(), max)?;
    let project = scene::planche::project(&source).map_err(DeckError::Invalid)?;
    Ok(Prepared { document, source, project })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_source_is_capped_then_read_in_either_format() {
        let starter = checked(scene::planche::STARTER, SOURCE_MAX_BYTES).unwrap();
        assert_eq!(starter.project.scenes().len(), 1);
        let legacy = checked("# A\n\n---\n\n# B\n", SOURCE_MAX_BYTES).unwrap();
        assert!(scene::planche::is_planche(&legacy.source));
        assert_eq!(legacy.project.scenes().len(), 2);
        assert!(
            matches!(checked("# A\n", 2), Err(DeckError::Api(error)) if error.code == tronc::Code::ResourceExhausted)
        );
        let Err(DeckError::Invalid(errors)) = checked("![](https://x.test/a.png)\n", 100) else { panic!() };
        assert_eq!(errors.0[0].line, 1);
    }

    #[test]
    fn a_projection_over_the_cap_is_refused() {
        let legacy = checked("# A\n\n---\n\n# B\n", SOURCE_MAX_BYTES).unwrap();
        let refused = prepared(legacy.document, legacy.source.len() - 1);
        assert!(matches!(refused, Err(DeckError::Api(error)) if error.code == tronc::Code::ResourceExhausted));
    }
}
