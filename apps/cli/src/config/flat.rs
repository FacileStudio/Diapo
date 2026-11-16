//! The config file is flat YAML: one `key: value` per line. Editing it line by line keeps
//! every line the CLI does not own, comments included.

fn value_of(line: &str, key: &str) -> Option<String> {
    let rest = line.strip_prefix(key)?.strip_prefix(':')?;
    Some(unquote(rest.trim()))
}

fn unquote(value: &str) -> String {
    for quote in ['"', '\''] {
        if value.len() >= 2 && value.starts_with(quote) && value.ends_with(quote) {
            return value[1..value.len() - 1].to_owned();
        }
    }
    value.split(" #").next().unwrap_or(value).trim().to_owned()
}

/// The value of `key`, when the file sets it to something.
pub fn get(text: &str, key: &str) -> Option<String> {
    text.lines().find_map(|line| value_of(line, key)).filter(|value| !value.is_empty())
}

/// The file with `key` set to `value`: its line replaced in place, or added at the end.
pub fn set(text: &str, key: &str, value: &str) -> String {
    let line = format!("{key}: {value}");
    let mut lines: Vec<String> = text.lines().map(str::to_owned).collect();
    match lines.iter().position(|existing| value_of(existing, key).is_some()) {
        Some(index) => lines[index] = line,
        None => lines.push(line),
    }
    lines.join("\n") + "\n"
}

/// The file without the line that sets `key`.
pub fn remove(text: &str, key: &str) -> String {
    let lines: Vec<&str> = text.lines().filter(|line| value_of(line, key).is_none()).collect();
    if lines.is_empty() { String::new() } else { lines.join("\n") + "\n" }
}
