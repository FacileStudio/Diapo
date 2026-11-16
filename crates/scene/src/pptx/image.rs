//! What a picture file is and how large, read from its first bytes.

/// A picture a slide can embed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Picture {
    /// The file extension, which is also its content type default: `png`, `jpeg` or `gif`.
    pub extension: &'static str,
    /// The size in pixels.
    pub width: u32,
    pub height: u32,
}

fn be(bytes: &[u8], at: usize) -> Option<u32> {
    Some(u32::from(u16::from_be_bytes([*bytes.get(at)?, *bytes.get(at + 1)?])))
}

fn jpeg(bytes: &[u8]) -> Option<(u32, u32)> {
    let mut at = 2;
    loop {
        let marker = *bytes.get(at + 1)?;
        if *bytes.get(at)? != 0xff {
            return None;
        }
        let frame = (0xc0..=0xcf).contains(&marker) && ![0xc4, 0xc8, 0xcc].contains(&marker);
        if frame {
            return Some((be(bytes, at + 7)?, be(bytes, at + 5)?));
        }
        at += 2 + be(bytes, at + 2)? as usize;
    }
}

/// The kind and size of `bytes`, for PNG, JPEG and GIF. Anything else, SVG included, is not
/// embedded: PowerPoint needs a bitmap.
pub(crate) fn picture(bytes: &[u8]) -> Option<Picture> {
    let (extension, (width, height)) = match bytes {
        [0x89, b'P', b'N', b'G', ..] => ("png", (be(bytes, 18)?, be(bytes, 22)?)),
        [b'G', b'I', b'F', b'8', ..] => {
            let le = |at: usize| Some(u32::from(u16::from_le_bytes([*bytes.get(at)?, *bytes.get(at + 1)?])));
            ("gif", (le(6)?, le(8)?))
        }
        [0xff, 0xd8, ..] => ("jpeg", jpeg(bytes)?),
        _ => return None,
    };
    (width > 0 && height > 0).then_some(Picture { extension, width, height })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn png_gif_and_jpeg_sizes_are_read_and_svg_is_refused() {
        let png = b"\x89PNG\r\n\x1a\n\0\0\0\rIHDR\0\0\x07\x80\0\0\x04\x38\x08\x06\0\0\0";
        assert_eq!(picture(png), Some(Picture { extension: "png", width: 1920, height: 1080 }));
        assert_eq!(picture(b"GIF89a\x10\0\x08\0rest").map(|p| (p.width, p.height)), Some((16, 8)));
        let jpeg = b"\xff\xd8\xff\xe0\0\x04\0\0\xff\xc0\0\x11\x08\0\x20\0\x40\x03";
        assert_eq!(picture(jpeg), Some(Picture { extension: "jpeg", width: 64, height: 32 }));
        assert_eq!(picture(b"<svg xmlns=\"http://www.w3.org/2000/svg\"/>"), None);
        assert_eq!(picture(b"\x89PNG"), None);
    }
}
