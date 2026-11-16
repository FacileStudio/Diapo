use tronc::Error;

use super::error::DeckError;

/// The largest deck source, in bytes, unless `SOURCE_MAX_BYTES` says otherwise.
pub const SOURCE_MAX_BYTES: usize = 512 << 10;

/// The largest JSON body of a route that carries a source: room for the default cap once every
/// byte is escaped, and for the envelope around it.
pub const BODY_MAX_BYTES: usize = 4 << 20;

/// What a new deck holds when the request gives no source.
pub const STARTER: &str = "# New deck\n\nOne idea per slide.\n";

/// Parses `source` as the scene format, within `max` bytes: `413` past the cap, the typed
/// problems with their lines when it does not parse.
pub fn checked(source: &str, max: usize) -> Result<scene::Project, DeckError> {
    if source.len() > max {
        return Err(Error::too_large(format!("the source is over the {max}-byte limit")).into());
    }
    scene::parse(source).map_err(DeckError::Invalid)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_source_is_capped_then_parsed() {
        assert!(checked(STARTER, SOURCE_MAX_BYTES).is_ok());
        assert!(
            matches!(checked("# A\n", 2), Err(DeckError::Api(error)) if error.code == tronc::Code::ResourceExhausted)
        );
        let Err(DeckError::Invalid(errors)) = checked("![](https://x.test/a.png)\n", 100) else { panic!() };
        assert_eq!(errors.0[0].line, 1);
    }
}
