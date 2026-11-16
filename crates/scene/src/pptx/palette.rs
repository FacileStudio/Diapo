//! The colours of the `facile` theme, as `RRGGBB`.

use crate::model::Scheme;

/// The colours one deck is drawn in.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Palette {
    pub bg: &'static str,
    pub fg: &'static str,
    pub muted: &'static str,
    pub fill: &'static str,
    pub strong: &'static str,
}

const LIGHT: Palette = Palette { bg: "FFFFFF", fg: "1A1A1A", muted: "6B6B6B", fill: "F4F4F4", strong: "E9E9E9" };
const DARK: Palette = Palette { bg: "0D0D0D", fg: "FAFAFA", muted: "9A9A9A", fill: "1C1C1C", strong: "2A2A2A" };

/// The palette of `scheme`. A file has no viewer preference to follow, so `auto` is light.
pub(crate) fn of(scheme: Scheme) -> Palette {
    match scheme {
        Scheme::Dark => DARK,
        _ => LIGHT,
    }
}
