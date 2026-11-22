//! What a `domain/` may take from the modules. A module's door re-exports
//! use cases and adapters next to its domain types, and a domain may only
//! take the domain types: what the door re-exports with `pub use domain::…`.
//! That holds for its own module's door too.
//!
//! The code is read with `syn` and as tokens, not as text: aliases, grouped
//! imports, `super::` chains and macro bodies all resolve to the same path.

use std::collections::{BTreeMap, BTreeSet};

use syn::ext::IdentExt;
use syn::visit::Visit;
use syn::{ItemUse, UseTree, Visibility};

use super::tokens::{paths_in, tokens_of};

pub type DomainExports = BTreeMap<String, BTreeSet<String>>;

const LAYERS: [&str; 5] = ["domain", "application", "ports", "infrastructure", "error"];
const OUTER_LAYERS: [&str; 2] = ["application", "infrastructure"];

/// One name brought in by a `use`: `a::b::C as D` is `[a, b, C]` named `D`.
/// A glob has no name.
struct Import {
    segments: Vec<String>,
    name: Option<String>,
    line: usize,
    public: bool,
}

#[derive(Default)]
struct Imports(Vec<Import>);

impl<'ast> Visit<'ast> for Imports {
    fn visit_item_use(&mut self, item: &'ast ItemUse) {
        let public = !matches!(item.vis, Visibility::Inherited);
        flatten(&item.tree, &mut Vec::new(), public, &mut self.0);
    }
}

fn flatten(tree: &UseTree, prefix: &mut Vec<String>, public: bool, out: &mut Vec<Import>) {
    let mut leaf = |segments: Vec<String>, name: Option<String>, line: usize| {
        out.push(Import {
            segments,
            name,
            line,
            public,
        });
    };
    match tree {
        UseTree::Path(path) => {
            prefix.push(path.ident.unraw().to_string());
            flatten(&path.tree, prefix, public, out);
            prefix.pop();
        }
        UseTree::Group(group) => {
            for item in &group.items {
                flatten(item, prefix, public, out);
            }
        }
        UseTree::Glob(glob) => leaf(prefix.clone(), None, glob.star_token.span.start().line),
        UseTree::Name(name) => {
            let line = name.ident.span().start().line;
            let ident = name.ident.unraw().to_string();
            if ident == "self" {
                leaf(prefix.clone(), prefix.last().cloned(), line);
            } else {
                let mut segments = prefix.clone();
                segments.push(ident.clone());
                leaf(segments, Some(ident), line);
            }
        }
        UseTree::Rename(rename) => {
            let mut segments = prefix.clone();
            segments.push(rename.ident.unraw().to_string());
            let line = rename.ident.span().start().line;
            leaf(segments, Some(rename.rename.unraw().to_string()), line);
        }
    }
}

fn imports_of(source: &str) -> Option<Vec<Import>> {
    let file = syn::parse_file(source).ok()?;
    let mut imports = Imports::default();
    imports.visit_file(&file);
    Some(imports.0)
}

/// For every `src/modules/<module>/mod.rs`, the names it exposes from its
/// own domain.
pub fn domain_exports<'a>(sources: impl Iterator<Item = (&'a str, &'a str)>) -> DomainExports {
    let mut exports = DomainExports::new();
    for (relative, text) in sources {
        let Some(module) = door_of(relative) else {
            continue;
        };
        let names = exports.entry(module.to_string()).or_default();
        for import in imports_of(text).unwrap_or_default() {
            if import.public
                && import
                    .segments
                    .first()
                    .is_some_and(|first| first == "domain")
            {
                names.extend(import.name);
            }
        }
    }
    exports
}

fn door_of(relative: &str) -> Option<&str> {
    let (_, inside) = relative.split_once("/src/modules/")?;
    inside.strip_suffix("/mod.rs").filter(|m| !m.contains('/'))
}

/// The module a file is, as a path from the crate root:
/// `core/src/modules/session/domain/state.rs` is `[modules, session, domain, state]`.
fn module_of(relative: &str) -> Vec<String> {
    let inside = relative
        .split_once("/src/")
        .map_or(relative, |(_, rest)| rest);
    let mut segments: Vec<String> = inside
        .trim_end_matches(".rs")
        .split('/')
        .map(str::to_string)
        .collect();
    if segments.last().is_some_and(|last| last == "mod") {
        segments.pop();
    }
    segments
}

/// A path as written, turned into a path from the crate root. `None` for
/// what does not point into this crate (`std::…`, a local item).
fn absolute(
    segments: &[&str],
    file_module: &[String],
    aliases: &BTreeMap<String, Vec<String>>,
) -> Option<Vec<String>> {
    let (first, rest) = segments.split_first()?;
    let mut base = match *first {
        "crate" => Vec::new(),
        "self" => file_module.to_vec(),
        "super" => file_module.to_vec(),
        name => aliases.get(name)?.clone(),
    };
    let mut rest = rest;
    if *first == "super" {
        base.pop()?;
        while let Some((&"super", after)) = rest.split_first() {
            base.pop()?;
            rest = after;
        }
    }
    base.extend(rest.iter().map(|segment| segment.to_string()));
    Some(base)
}

/// Why a domain file of module `own` may not use this path; `None` when it may.
fn objection(path: &[String], own: &str, exports: &DomainExports) -> Option<String> {
    let [root, module, name, ..] = path else {
        return None;
    };
    if root != "modules" {
        return None;
    }
    if LAYERS.contains(&name.as_str()) {
        let outside = module != own || OUTER_LAYERS.contains(&name.as_str());
        return outside.then(|| format!("domain reaches into `modules::{module}::{name}`"));
    }
    let is_domain_type = exports
        .get(module)
        .is_some_and(|names| names.contains(name));
    (!is_domain_type)
        .then(|| format!("domain takes `{name}`, which is not a domain type of `{module}`"))
}

/// `(line, why)` for everything a domain file of module `own` takes from the
/// modules that it may not.
pub fn domain_violations(
    relative: &str,
    source: &str,
    own: &str,
    exports: &DomainExports,
) -> Vec<(usize, String)> {
    let (Some(imports), Some(tokens)) = (imports_of(source), tokens_of(source)) else {
        return vec![(1, "this file cannot be read as Rust".to_string())];
    };
    let file_module = module_of(relative);
    let aliases = aliases_of(&imports, &file_module);
    let mut found = BTreeSet::new();
    for import in &imports {
        let segments: Vec<&str> = import.segments.iter().map(String::as_str).collect();
        let Some(path) = absolute(&segments, &file_module, &aliases) else {
            continue;
        };
        if import.name.is_none() && path.first().is_some_and(|root| root == "modules") {
            let all_of = path.join("::");
            found.insert((import.line, format!("domain imports all of `{all_of}`")));
        }
        found.extend(objection(&path, own, exports).map(|why| (import.line, why)));
        if import.public && reexports_from_outside(&path, own) {
            let from = path.join("::");
            found.insert((import.line, format!("domain re-exports `{from}`")));
        }
    }
    for (segments, line) in paths_in(&tokens) {
        // A lone name was judged where it was imported.
        if segments.len() < 2 {
            continue;
        }
        let path = absolute(&segments, &file_module, &aliases);
        found.extend(path.and_then(|path| objection(&path, own, exports).map(|why| (line, why))));
    }
    found.into_iter().collect()
}

/// A domain may re-export from its own domain and from nowhere else in the
/// modules: otherwise a use case could reach the door dressed as a type.
fn reexports_from_outside(path: &[String], own: &str) -> bool {
    match path {
        [root, module, layer, ..] if root == "modules" => module != own || layer != "domain",
        [root, ..] => root == "modules",
        [] => false,
    }
}

/// The names this file's imports give to paths of the crate. Resolved in
/// rounds, since one import may build on the name of another.
fn aliases_of(imports: &[Import], file_module: &[String]) -> BTreeMap<String, Vec<String>> {
    let mut aliases = BTreeMap::new();
    for _ in 0..3 {
        for import in imports {
            let Some(name) = &import.name else {
                continue;
            };
            let segments: Vec<&str> = import.segments.iter().map(String::as_str).collect();
            if let Some(path) = absolute(&segments, file_module, &aliases) {
                aliases.insert(name.clone(), path);
            }
        }
    }
    aliases
}
