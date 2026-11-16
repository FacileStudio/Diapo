//! The few XML pieces every part shares.

/// The declaration every part starts with.
pub(crate) const HEAD: &str = "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\n";

/// The namespaces of a presentation part.
pub(crate) const NS: &str = "xmlns:a=\"http://schemas.openxmlformats.org/drawingml/2006/main\" \
    xmlns:r=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships\" \
    xmlns:p=\"http://schemas.openxmlformats.org/presentationml/2006/main\"";

const REL: &str = "http://schemas.openxmlformats.org/officeDocument/2006/relationships";

/// `text` safe as XML text or as an attribute value. Characters XML 1.0 forbids are dropped.
pub(crate) fn escape(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for c in text.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\t' | '\n' | '\r' => out.push(' '),
            c if (c as u32) < 0x20 || c == '\u{fffe}' || c == '\u{ffff}' => {}
            c => out.push(c),
        }
    }
    out
}

/// A relationships part: each entry is an id, a type name under the office namespace, and a
/// target.
pub(crate) fn rels(entries: &[(String, &str, String)]) -> String {
    let mut out =
        format!("{HEAD}<Relationships xmlns=\"http://schemas.openxmlformats.org/package/2006/relationships\">");
    for (id, kind, target) in entries {
        out.push_str(&format!("<Relationship Id=\"{id}\" Type=\"{REL}/{kind}\" Target=\"{}\"/>", escape(target)));
    }
    out.push_str("</Relationships>");
    out
}

/// The empty group every shape tree starts with.
pub(crate) const TREE: &str = "<p:nvGrpSpPr><p:cNvPr id=\"1\" name=\"\"/><p:cNvGrpSpPr/><p:nvPr/></p:nvGrpSpPr>\
    <p:grpSpPr><a:xfrm><a:off x=\"0\" y=\"0\"/><a:ext cx=\"0\" cy=\"0\"/><a:chOff x=\"0\" y=\"0\"/>\
    <a:chExt cx=\"0\" cy=\"0\"/></a:xfrm></p:grpSpPr>";

#[cfg(test)]
mod tests {
    #[test]
    fn text_is_escaped_and_control_characters_are_dropped() {
        assert_eq!(super::escape("a < b & \"c\"\u{1}\n"), "a &lt; b &amp; &quot;c&quot; ");
    }
}
