//! The theme part: the facile colours and one plain font.

use super::palette::Palette;
use super::xml::HEAD;

/// The font of every slide: present on every system that opens a PowerPoint file.
pub(crate) const FONT: &str = "Arial";

const COLORS: &str = "<a:clrScheme name=\"Facile\"><a:dk1><a:srgbClr val=\"{dark}\"/></a:dk1><a:lt1>\
<a:srgbClr val=\"{light}\"/></a:lt1><a:dk2><a:srgbClr val=\"{dark2}\"/></a:dk2><a:lt2><a:srgbClr val=\"{light2}\"/>\
</a:lt2><a:accent1><a:srgbClr val=\"4F46E5\"/></a:accent1><a:accent2><a:srgbClr val=\"0EA5E9\"/></a:accent2>\
<a:accent3><a:srgbClr val=\"10B981\"/></a:accent3><a:accent4><a:srgbClr val=\"F59E0B\"/></a:accent4><a:accent5>\
<a:srgbClr val=\"EF4444\"/></a:accent5><a:accent6><a:srgbClr val=\"8B5CF6\"/></a:accent6><a:hlink>\
<a:srgbClr val=\"2563EB\"/></a:hlink><a:folHlink><a:srgbClr val=\"7C3AED\"/></a:folHlink></a:clrScheme>";

const FONTS: &str = "<a:fontScheme name=\"Facile\"><a:majorFont><a:latin typeface=\"{font}\"/><a:ea typeface=\"\"/>\
<a:cs typeface=\"\"/></a:majorFont><a:minorFont><a:latin typeface=\"{font}\"/><a:ea typeface=\"\"/>\
<a:cs typeface=\"\"/></a:minorFont></a:fontScheme>";

const FILL: &str = "<a:solidFill><a:schemeClr val=\"phClr\"/></a:solidFill>";

const EFFECT: &str = "<a:effectStyle><a:effectLst/></a:effectStyle>";

fn formats() -> String {
    let line = |width: u32| format!("<a:ln w=\"{width}\">{FILL}</a:ln>");
    format!(
        "<a:fmtScheme name=\"Facile\"><a:fillStyleLst>{FILL}{FILL}{FILL}</a:fillStyleLst><a:lnStyleLst>{}{}{}\
         </a:lnStyleLst><a:effectStyleLst>{EFFECT}{EFFECT}{EFFECT}</a:effectStyleLst><a:bgFillStyleLst>{FILL}{FILL}\
         {FILL}</a:bgFillStyleLst></a:fmtScheme>",
        line(6350),
        line(12700),
        line(19050)
    )
}

/// Orders the two ends of `palette` as a theme names them: `(dark, light, dark 2, light 2)`.
fn ends(palette: &Palette) -> [&'static str; 4] {
    match palette.bg < palette.fg {
        true => [palette.bg, palette.fg, palette.fill, palette.muted],
        false => [palette.fg, palette.bg, palette.muted, palette.fill],
    }
}

/// Whether `palette` is light text on a dark stage, so the master maps `bg1` to the dark end.
pub(crate) fn dark(palette: &Palette) -> bool {
    palette.bg < palette.fg
}

/// The theme part for `palette`.
pub(crate) fn part(palette: &Palette) -> String {
    let [dark, light, dark2, light2] = ends(palette);
    let colors =
        COLORS.replace("{dark}", dark).replace("{light}", light).replace("{dark2}", dark2).replace("{light2}", light2);
    format!(
        "{HEAD}<a:theme xmlns:a=\"http://schemas.openxmlformats.org/drawingml/2006/main\" name=\"Facile\">\
         <a:themeElements>{colors}{}{}</a:themeElements></a:theme>",
        FONTS.replace("{font}", FONT),
        formats()
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::Scheme;
    use crate::pptx::palette;

    #[test]
    fn the_theme_names_the_dark_and_light_ends_of_each_scheme() {
        let light = part(&palette::of(Scheme::Light));
        assert!(
            light.contains("<a:dk1><a:srgbClr val=\"1A1A1A\"/></a:dk1><a:lt1><a:srgbClr val=\"FFFFFF\"/>"),
            "{light}"
        );
        let night = palette::of(Scheme::Dark);
        assert!(dark(&night) && !dark(&palette::of(Scheme::Auto)));
        assert!(part(&night).contains("<a:dk1><a:srgbClr val=\"0D0D0D\"/></a:dk1><a:lt1><a:srgbClr val=\"FAFAFA\"/>"));
    }
}
