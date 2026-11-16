/// The longest asset name, in bytes.
pub const MAX_NAME: usize = 200;

/// The folder an upload lands in when it names none, as the format's examples write it.
pub const FOLDER: &str = "assets";

fn segment(part: &str) -> bool {
    !part.is_empty() && !part.starts_with('.') && part.bytes().all(|b| b.is_ascii_alphanumeric() || b"._-".contains(&b))
}

/// Whether `name` is an asset name: a relative path of at most [`MAX_NAME`] bytes whose
/// segments hold ASCII letters, digits, `.`, `_` and `-` and never start with a dot. So no
/// `..`, no absolute path, no backslash, no scheme, nothing a URL or a header must escape.
/// Every such name is a path the scene format accepts.
pub fn valid(name: &str) -> bool {
    name.len() <= MAX_NAME && name.split('/').all(segment)
}

/// The asset name for an upload that gives only its file name: [`FOLDER`], then the base name
/// with every other character turned into `-`.
pub fn from_file(raw: &str) -> String {
    let base = raw.rsplit(['/', '\\']).next().unwrap_or_default();
    let kept = |c: char| if c.is_ascii_alphanumeric() || "._-".contains(c) { c } else { '-' };
    let clean: String = base.chars().map(kept).collect();
    let clean: String = clean.trim_start_matches(['.', '-']).chars().take(MAX_NAME - FOLDER.len() - 1).collect();
    format!("{FOLDER}/{}", if clean.is_empty() { "file" } else { &clean })
}

/// The asset name a source reference means, and the `#fragment` it carries: a leading `./` and
/// any `?query` are dropped.
pub fn referenced(path: &str) -> (&str, &str) {
    let path = path.strip_prefix("./").unwrap_or(path);
    let (rest, fragment) = path.find('#').map_or((path, ""), |at| path.split_at(at));
    (rest.split('?').next().unwrap_or(rest), fragment)
}

#[cfg(test)]
mod tests {
    use super::*;

    const REFUSED: [&str; 12] = [
        "",
        "../secret.png",
        "assets/../../etc/passwd",
        "/etc/passwd",
        "assets//a.png",
        "assets/.env",
        "C:\\a.png",
        "assets\\a.png",
        "https://x.test/a.png",
        "a b.png",
        "a%2e%2e/b.png",
        "assets/a.png?x=1",
    ];

    #[test]
    fn names_are_plain_relative_paths() {
        for name in ["assets/chart.png", "logo.svg", "fonts/Inter-Bold_2.woff2"] {
            assert!(valid(name), "{name}");
        }
        for name in REFUSED {
            assert!(!valid(name), "{name}");
        }
        assert!(!valid(&format!("assets/{}.png", "a".repeat(MAX_NAME))));
    }

    #[test]
    fn every_valid_name_is_a_path_the_scene_format_accepts() {
        for name in ["assets/chart.png", "logo.svg", "a/b/c/d.mp4", &from_file("Capture d'écran.png")] {
            assert!(valid(name), "{name}");
            assert!(scene::parse(&format!("<img src=\"{name}\">\n")).is_ok(), "{name}");
        }
    }

    #[test]
    fn a_file_name_becomes_a_safe_name_under_assets() {
        assert_eq!(from_file("chart.png"), "assets/chart.png");
        assert_eq!(from_file("../../etc/passwd"), "assets/passwd");
        assert_eq!(from_file("C:\\Users\\me\\Capture d'écran.png"), "assets/Capture-d--cran.png");
        assert_eq!(from_file(".env"), "assets/env");
        assert_eq!(from_file(""), "assets/file");
        assert!(valid(&from_file(&"é".repeat(400))));
    }

    #[test]
    fn a_reference_names_its_asset_and_keeps_its_fragment() {
        assert_eq!(referenced("./assets/a.svg#icon"), ("assets/a.svg", "#icon"));
        assert_eq!(referenced("assets/a.png?v=2"), ("assets/a.png", ""));
        assert_eq!(referenced("a.svg?v=2#x"), ("a.svg", "#x"));
    }
}
