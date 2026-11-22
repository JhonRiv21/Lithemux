//! The working directory as a card shows it: short, with `~` for the home.

use std::path::Path;

const KEPT_AT_THE_END: usize = 2;
const ELLIPSIS: &str = "…";

pub fn cwd_label(cwd: &str, home: Option<&Path>) -> String {
    let path = Path::new(cwd);
    let (prefix, inside) = match home.and_then(|home| path.strip_prefix(home).ok()) {
        Some(inside) => ("~", inside),
        None => ("", path.strip_prefix("/").unwrap_or(path)),
    };
    let parts: Vec<String> = inside
        .components()
        .map(|part| part.as_os_str().to_string_lossy().into_owned())
        .collect();
    if parts.is_empty() {
        return if prefix.is_empty() { "/" } else { prefix }.to_string();
    }
    if parts.len() <= KEPT_AT_THE_END + 1 {
        return format!("{prefix}/{}", parts.join("/"));
    }
    let end = parts[parts.len() - KEPT_AT_THE_END..].join("/");
    format!("{prefix}/{ELLIPSIS}/{end}")
}
