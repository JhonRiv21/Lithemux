//! PLAN §3.4: «solo lectura sobre cmux». Where cmux keeps its workspaces is
//! known in one file, and that file has nothing that could change a file:
//! no write, create, remove, rename, copy, link, permission or time change,
//! and no program started.
//!
//! The code is read as tokens, not as lines: comments do not count, and a
//! path built over several lines (as rustfmt leaves it) is still seen.

use std::fs;
use std::path::{Path, PathBuf};

#[path = "guards/tokens.rs"]
#[allow(dead_code)]
mod tokens;

use tokens::{Kind, tokens_of};

/// Relative to `crates/`.
const READER: &str = "core/src/modules/import/infrastructure/cmux_session_file.rs";

/// Names that change a file or start a program. Besides these, any name
/// that starts with `write` (`write`, `write_all`, `writeln!`, `write_fmt`)
/// or `set_` (`set_len`, `set_permissions`, `set_modified`, `set_times`).
const CHANGES_A_FILE: [&str; 17] = [
    "create",
    "create_new",
    "create_dir",
    "create_dir_all",
    "OpenOptions",
    "options",
    "append",
    "truncate",
    "remove_file",
    "remove_dir",
    "remove_dir_all",
    "rename",
    "copy",
    "hard_link",
    "symlink",
    "persist",
    "Command",
];

fn changes_a_file(name: &str) -> bool {
    name.starts_with("write") || name.starts_with("set_") || CHANGES_A_FILE.contains(&name)
}

/// The code without its doc comments, which the lexer would keep as text.
fn without_docs(source: &str) -> String {
    source
        .lines()
        .filter(|line| {
            let line = line.trim_start();
            !line.starts_with("///") && !line.starts_with("//!")
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// Whether the code says where cmux keeps its files: its folder (`"cmux"`)
/// or the name of its file (`…cmuxterm…`). Code the lexer does not accept
/// is read as text.
fn names_the_place_of_cmux(source: &str) -> bool {
    let names_it = |text: &str| {
        let text = text.to_lowercase();
        text == "cmux" || text.contains("cmuxterm") || text.contains("application support/cmux")
    };
    match tokens_of(&without_docs(source)) {
        Some(tokens) => tokens.iter().any(|token| match &token.kind {
            Kind::Text(text) => names_it(text),
            _ => false,
        }),
        None => names_it(source) || source.to_lowercase().contains("\"cmux\""),
    }
}

/// The names in the code that change a file or start a program.
fn file_changes_in(source: &str) -> Vec<String> {
    let Some(tokens) = tokens_of(&without_docs(source)) else {
        return vec!["<not Rust the lexer accepts>".to_string()];
    };
    let mut found: Vec<String> = Vec::new();
    for token in &tokens {
        if let Some(name) = token.ident()
            && changes_a_file(name)
            && !found.iter().any(|seen| seen == name)
        {
            found.push(name.to_string());
        }
    }
    found
}

fn rust_sources(dir: &Path, files: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(dir).into_iter().flatten().flatten() {
        let path = entry.path();
        if path.is_dir() {
            rust_sources(&path, files);
        } else if path.extension().is_some_and(|ext| ext == "rs") {
            files.push(path);
        }
    }
}

fn crates_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("..")
}

// The guard itself, on what someone would write without meaning to dodge it.
#[test]
fn the_guard_recognizes_the_place_of_cmux() {
    for named in [
        r#"home.join("Library/Application Support/cmux")"#,
        r#"let name = "session-com.CMUXTERM.app.json";"#,
        r#"let dir = data_dir().join("cmux");"#,
        "let file = home\n    .join(\"Library\")\n    .join(\"Application Support\")\n    .join(\"cmux\");",
    ] {
        assert!(names_the_place_of_cmux(named), "{named}");
    }
    for not_named in [
        "// cmux keeps it in Application Support/cmux",
        "/// The parts of cmux's `session-com.cmuxterm.app.json`",
        "const CMUX_PREFIX: &str = \"CMUX_\";",
        "const SHIMS: &str = \"cmux-cli-shims\";",
    ] {
        assert!(!names_the_place_of_cmux(not_named), "{not_named}");
    }
}

#[test]
fn the_guard_recognizes_what_changes_a_file() {
    for (code, name) in [
        ("std::fs::write(&path, bytes)?;", "write"),
        ("use std::fs::write as put;", "write"),
        ("writeln!(file, \"x\")?;", "writeln"),
        ("file.write_fmt(format_args!(\"x\"))?;", "write_fmt"),
        ("file.set_modified(time)?;", "set_modified"),
        ("file.set_times(times)?;", "set_times"),
        ("File::options().append(true)", "options"),
        ("let f = File::create(p)?;", "create"),
        ("fs::rename(a, b)", "rename"),
        ("temporary.persist(&path)?;", "persist"),
        ("Command::new(\"rm\").arg(&path).status()?;", "Command"),
    ] {
        let found = file_changes_in(code);
        assert!(found.iter().any(|f| f == name), "{code}: {found:?}");
    }
    assert!(file_changes_in("let bytes = std::fs::read(&self.path)?;").is_empty());
    assert!(file_changes_in("// never fs::write here").is_empty());
    assert!(file_changes_in("let size = std::fs::metadata(&path)?.len();").is_empty());
}

#[test]
fn only_the_reader_knows_where_cmux_keeps_its_files() {
    let crates = crates_dir();
    let mut files = Vec::new();
    for crate_dir in fs::read_dir(&crates).into_iter().flatten().flatten() {
        rust_sources(&crate_dir.path().join("src"), &mut files);
    }
    assert!(files.len() > 50, "found only {} files", files.len());
    let reader = fs::canonicalize(crates.join(READER)).ok();
    let others: Vec<String> = files
        .iter()
        .filter(|file| fs::canonicalize(file).ok() != reader)
        .filter(|file| fs::read_to_string(file).is_ok_and(|text| names_the_place_of_cmux(&text)))
        .map(|file| file.display().to_string())
        .collect();
    assert!(others.is_empty(), "{}", others.join("\n"));
}

#[test]
fn the_reader_of_cmux_never_changes_a_file() {
    let text = fs::read_to_string(crates_dir().join(READER)).unwrap();

    assert!(
        names_the_place_of_cmux(&text),
        "the reader moved: update READER"
    );
    assert_eq!(file_changes_in(&text), Vec::<String>::new());
}
