/// The largest single asset, in bytes, unless `ASSET_MAX_BYTES` says otherwise.
pub const ASSET_MAX_BYTES: usize = 25 << 20;

/// The most bytes the assets of one deck hold together, unless `DECK_ASSETS_MAX_BYTES` says
/// otherwise.
pub const DECK_ASSETS_MAX_BYTES: usize = 200 << 20;

fn boxed(bytes: &[u8]) -> Option<&'static str> {
    match bytes.get(8..12)? {
        b"avif" | b"avis" => Some("image/avif"),
        b"M4A " => Some("audio/mp4"),
        _ => Some("video/mp4"),
    }
}

fn svg(bytes: &[u8]) -> bool {
    let head = String::from_utf8_lossy(&bytes[..bytes.len().min(2048)]);
    let head = head.trim_start_matches('\u{feff}').trim_start();
    let opens = ["<svg", "<?xml", "<!--", "<!DOCTYPE svg", "<!doctype svg"].iter().any(|open| head.starts_with(open));
    opens && head.contains("<svg")
}

fn media(bytes: &[u8]) -> Option<&'static str> {
    match bytes {
        [_, _, _, _, b'f', b't', b'y', b'p', ..] => boxed(bytes),
        [0x1a, 0x45, 0xdf, 0xa3, ..] => Some("video/webm"),
        [b'I', b'D', b'3', ..] | [0xff, 0xfb | 0xf3 | 0xf2, ..] => Some("audio/mpeg"),
        [b'R', b'I', b'F', b'F', _, _, _, _, b'W', b'A', b'V', b'E', ..] => Some("audio/wav"),
        [b'O', b'g', b'g', b'S', ..] => Some("audio/ogg"),
        _ => None,
    }
}

fn font(bytes: &[u8]) -> Option<&'static str> {
    match bytes {
        [b'w', b'O', b'F', b'2', ..] => Some("font/woff2"),
        [b'w', b'O', b'F', b'F', ..] => Some("font/woff"),
        [0, 1, 0, 0, ..] => Some("font/ttf"),
        [b'O', b'T', b'T', b'O', ..] => Some("font/otf"),
        _ => None,
    }
}

fn image(bytes: &[u8]) -> Option<&'static str> {
    match bytes {
        [0x89, b'P', b'N', b'G', 0x0d, 0x0a, 0x1a, 0x0a, ..] => Some("image/png"),
        [0xff, 0xd8, 0xff, ..] => Some("image/jpeg"),
        [b'G', b'I', b'F', b'8', b'7' | b'9', b'a', ..] => Some("image/gif"),
        [b'R', b'I', b'F', b'F', _, _, _, _, b'W', b'E', b'B', b'P', ..] => Some("image/webp"),
        _ => svg(bytes).then_some("image/svg+xml"),
    }
}

/// The media type the bytes declare by their content, among those a slide loads: images, video,
/// audio and fonts. Anything else answers `None` and is refused; the type an upload declares is
/// never trusted. SVG is served with a `sandbox` policy, so opened on its own it runs nothing.
pub fn sniff(bytes: &[u8]) -> Option<&'static str> {
    image(bytes).or_else(|| media(bytes)).or_else(|| font(bytes))
}

#[cfg(test)]
mod tests {
    use super::sniff;

    #[test]
    fn images_video_audio_and_fonts_are_recognised_by_their_bytes() {
        assert_eq!(sniff(b"\x89PNG\r\n\x1a\nrest"), Some("image/png"));
        assert_eq!(sniff(b"\xff\xd8\xff\xe0"), Some("image/jpeg"));
        assert_eq!(sniff(b"GIF89a.."), Some("image/gif"));
        assert_eq!(sniff(b"RIFF\0\0\0\0WEBPVP8 "), Some("image/webp"));
        assert_eq!(sniff(b"\0\0\0\x1cftypavif\0\0"), Some("image/avif"));
        assert_eq!(sniff(b"<svg xmlns=\"http://www.w3.org/2000/svg\"/>"), Some("image/svg+xml"));
        assert_eq!(sniff(b"\xef\xbb\xbf<?xml version=\"1.0\"?>\n<svg/>"), Some("image/svg+xml"));
        assert_eq!(sniff(b"\0\0\0\x18ftypisom\0\0"), Some("video/mp4"));
        assert_eq!(sniff(b"\0\0\0\x18ftypM4A \0\0"), Some("audio/mp4"));
        assert_eq!(sniff(b"\x1a\x45\xdf\xa3...."), Some("video/webm"));
        assert_eq!(sniff(b"ID3\x04\0"), Some("audio/mpeg"));
        assert_eq!(sniff(b"RIFF\0\0\0\0WAVEfmt "), Some("audio/wav"));
        assert_eq!(sniff(b"OggS\0"), Some("audio/ogg"));
        assert_eq!(sniff(b"wOF2\0"), Some("font/woff2"));
        assert_eq!(sniff(b"OTTO\0"), Some("font/otf"));
    }

    #[test]
    fn markup_scripts_and_unknown_bytes_are_refused() {
        for bytes in [
            &b"<!doctype html><script>1</script>"[..],
            b"<html><svg/></html>",
            b"alert(1)",
            b"%PDF-1.7",
            b"",
            b"PK\x03\x04",
        ] {
            assert_eq!(sniff(bytes), None, "{:?}", String::from_utf8_lossy(bytes));
        }
    }
}
