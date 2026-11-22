//! Source-level boundary rules of CLAUDE.md «Arquitectura». Text based on
//! purpose: it has to run on fixtures that are never compiled.

use std::fs;
use std::path::{Path, PathBuf};

use super::foreign_domain_types::{DomainExports, domain_exports, domain_violations};

const LAYERS: [&str; 5] = ["domain", "application", "ports", "infrastructure", "error"];
const JUNK_DRAWERS: [&str; 4] = ["utils", "helpers", "common", "misc"];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Violation {
    pub rule: &'static str,
    pub file: String,
    pub line: usize,
    pub detail: String,
}

/// Checks every `.rs` file under `<root>/*/src` and `<root>/*/tests`.
pub fn check(root: &Path) -> Vec<Violation> {
    let mut files = Vec::new();
    collect(root, &mut files);
    files.sort();
    let sources: Vec<(String, String)> = files
        .iter()
        .filter_map(|path| {
            let relative = path
                .strip_prefix(root)
                .unwrap_or(path)
                .to_string_lossy()
                .replace('\\', "/");
            Some((relative, fs::read_to_string(path).ok()?))
        })
        .collect();
    let exports = domain_exports(sources.iter().map(|(file, text)| (&**file, &**text)));
    let mut violations = Vec::new();
    for (relative, text) in &sources {
        check_file(relative, text, &exports, &mut violations);
    }
    violations
}

pub fn collect(dir: &Path, files: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();
        if path.is_dir() {
            // `arch` is this checker: its rule strings would trip the rules.
            if !matches!(
                name.as_str(),
                "target" | "arch-fixtures" | "guard-fixtures" | "arch" | "snapshots"
            ) {
                collect(&path, files);
            }
        } else if name.ends_with(".rs") {
            files.push(path);
        }
    }
}

struct FileKind<'a> {
    relative: &'a str,
    /// `Some("session")` for a file inside `modules/session/`.
    module: Option<&'a str>,
    is_test_code: bool,
    in_domain: bool,
    in_platform: bool,
}

fn classify(relative: &str) -> FileKind<'_> {
    let module = relative
        .split_once("/src/modules/")
        .and_then(|(_, rest)| rest.split_once('/'))
        .map(|(module, _)| module);
    let is_test_code = relative.contains("/test/") || relative.contains("/tests/");
    FileKind {
        relative,
        module,
        is_test_code,
        in_domain: module.is_some() && relative.contains("/domain/") && !is_test_code,
        in_platform: relative.contains("/src/platform/"),
    }
}

fn check_file(relative: &str, text: &str, exports: &DomainExports, out: &mut Vec<Violation>) {
    let kind = classify(relative);
    let mut report = |rule: &'static str, line: usize, detail: String| {
        out.push(Violation {
            rule,
            file: relative.to_string(),
            line,
            detail,
        });
    };

    let stem = relative
        .rsplit('/')
        .next()
        .unwrap_or(relative)
        .trim_end_matches(".rs");
    if JUNK_DRAWERS.contains(&stem)
        || JUNK_DRAWERS
            .iter()
            .any(|j| relative.contains(&format!("/{j}/")))
    {
        report(
            "no-junk-drawers",
            1,
            format!("`{stem}` says nothing about what lives there"),
        );
    }

    if let (true, Some(own)) = (kind.in_domain, kind.module) {
        for (line, why) in domain_violations(relative, text, own, exports) {
            report("domain-purity", line, why);
        }
    }

    let declares_only = is_declaration_only_file(&kind);
    let mut previous = "";
    for (index, raw) in text.lines().enumerate() {
        let line = raw.trim();
        let number = index + 1;
        if line.starts_with("//") {
            continue;
        }
        if kind.in_domain {
            for forbidden in ["rusqlite", "std::fs", "std::process", "use std::{"] {
                if line.contains(forbidden) {
                    report(
                        "domain-purity",
                        number,
                        format!("domain uses `{forbidden}`"),
                    );
                }
            }
            for layer in ["application", "infrastructure"] {
                if line.contains(&format!("::{layer}::")) || line.contains(&format!("::{layer};")) {
                    report("domain-purity", number, format!("domain imports `{layer}`"));
                }
            }
        }
        if kind.in_platform && line.contains("modules::") {
            report(
                "platform-independence",
                number,
                "platform imports a module".to_string(),
            );
        }
        for foreign in foreign_internals(line, kind.module) {
            report(
                "module-internals",
                number,
                format!("reaches into `{foreign}`"),
            );
        }
        if line.contains("modules::{") {
            report(
                "module-internals",
                number,
                "grouped `modules::{…}` import hides which door is used".to_string(),
            );
        }
        if !kind.is_test_code {
            if line.contains("::test::") || line.contains("tests::support") {
                report(
                    "test-isolation",
                    number,
                    "production code imports test code".to_string(),
                );
            }
            if (line == "mod test;" || line == "pub mod test;") && previous != "#[cfg(test)]" {
                report(
                    "test-isolation",
                    number,
                    "`mod test` without `#[cfg(test)]`".to_string(),
                );
            }
        }
        if line.starts_with("pub use ") && line.ends_with("::*;") {
            report("no-barrels", number, "glob re-export".to_string());
        }
        if line.contains("mod prelude") {
            report("no-barrels", number, "prelude module".to_string());
        }
        if declares_only && !is_module_declaration(line) {
            report(
                "no-barrels",
                number,
                "this `mod.rs` may only declare submodules".to_string(),
            );
        }
        if !line.is_empty() {
            previous = line;
        }
    }
}

/// `modules::<name>::<layer>` paths that point inside a module other than
/// the one the file belongs to.
fn foreign_internals(line: &str, own_module: Option<&str>) -> Vec<String> {
    let mut found = Vec::new();
    let mut rest = line;
    while let Some(position) = rest.find("modules::") {
        rest = &rest[position + "modules::".len()..];
        let mut parts = rest.split("::");
        let (Some(module), Some(next)) = (parts.next(), parts.next()) else {
            continue;
        };
        let layer: String = next
            .chars()
            .take_while(|c| c.is_ascii_alphanumeric() || *c == '_')
            .collect();
        if LAYERS.contains(&layer.as_str()) && Some(module) != own_module {
            found.push(format!("modules::{module}::{layer}"));
        }
    }
    found
}

// A module's own `mod.rs` is its door and may re-export concrete names, and so
// is the `mod.rs` of an app feature (`app/src/<feature>/mod.rs`); every other
// `mod.rs` under `src/`, and `lib.rs`, only declare submodules.
fn is_declaration_only_file(kind: &FileKind<'_>) -> bool {
    let relative = kind.relative;
    if !relative.contains("/src/") {
        return false;
    }
    if relative.ends_with("/src/lib.rs") {
        return true;
    }
    if !relative.ends_with("/mod.rs") {
        return false;
    }
    let is_module_door = kind
        .module
        .is_some_and(|module| relative.ends_with(&format!("/src/modules/{module}/mod.rs")));
    !is_module_door && !is_app_feature_door(relative)
}

// `relative` starts at `crates/`, so only the `app` crate itself qualifies:
// not `tauri-app/src/...` nor an `app/src/` nested inside another crate.
fn is_app_feature_door(relative: &str) -> bool {
    relative
        .strip_prefix("app/src/")
        .and_then(|inside_app| inside_app.strip_suffix("/mod.rs"))
        .is_some_and(|feature| !feature.contains('/'))
}

fn is_module_declaration(line: &str) -> bool {
    line.is_empty()
        || line == "#[cfg(test)]"
        || line.starts_with("#![")
        || ((line.starts_with("mod ") || line.starts_with("pub mod ")) && line.ends_with(';'))
}
