use crate::palette::ranking::{Entry, Index};

fn entry(id: &str, title: &str, category: &str, role: &str, activity: Option<i64>) -> Entry {
    Entry {
        session_id: id.to_string(),
        title: title.to_string(),
        slug: title.to_lowercase().replace(' ', "-"),
        category: category.to_string(),
        category_slug: category.to_lowercase(),
        role: role.to_string(),
        last_activity_at: activity,
    }
}

fn ids(query: &str, entries: &[Entry]) -> Vec<String> {
    Index::new(entries.to_vec())
        .rank(query)
        .into_iter()
        .map(|entry| entry.session_id.clone())
        .collect()
}

// PLAN §3.5: exacto antes que prefijo, prefijo antes que aproximado
#[test]
fn an_exact_match_comes_before_a_prefix_and_a_prefix_before_a_fuzzy_one() {
    let entries = [
        entry("fuzzy", "Pruebas de lanzamiento", "Demo", "", Some(30)),
        entry("prefix", "Plan de pruebas", "Demo", "", Some(20)),
        entry("exact", "Plan", "Demo", "", Some(10)),
        entry("none", "Ejecutora", "Demo", "", Some(40)),
    ];

    assert_eq!(ids("plan", &entries), ["exact", "prefix", "fuzzy"]);
    // «pln» is in all three only as letters in order: activity decides.
    assert_eq!(ids("pln", &entries), ["fuzzy", "prefix", "exact"]);
}

// PLAN §3.5: desempate por actividad reciente
#[test]
fn among_equal_matches_the_most_recently_active_comes_first() {
    let entries = [
        entry("old", "api old", "Demo", "", Some(10)),
        entry("never", "api never", "Demo", "", None),
        entry("new", "api new", "Demo", "", Some(99)),
    ];

    assert_eq!(ids("api", &entries), ["new", "old", "never"]);
}

// PLAN §3.5: busca por nombre, slug, categoría y rol
#[test]
fn a_session_is_found_by_its_name_slug_category_and_role() {
    let entries = [
        entry("by-title", "Revisión", "Demo", "", None),
        entry("by-category", "Otra", "Selektal", "", None),
        entry("by-role", "Tercera", "Demo", "documental", None),
    ];
    let mut by_slug = entry("by-slug", "Cuarta", "Demo", "", None);
    by_slug.slug = "tablero-api".to_string();
    let entries = [entries.as_slice(), &[by_slug]].concat();

    assert_eq!(ids("revisión", &entries), ["by-title"]);
    assert_eq!(ids("selektal", &entries), ["by-category"]);
    assert_eq!(ids("documental", &entries), ["by-role"]);
    assert_eq!(ids("tablero", &entries), ["by-slug"]);
}

// PLAN §3.5: aproximada, sin importar mayúsculas ni tildes, y a través de la
// categoría y el nombre
#[test]
fn the_search_ignores_case_and_accents_and_reaches_across_category_and_name() {
    let entries = [
        entry("a", "Revisión del API", "Selektal", "", None),
        entry("b", "Planificadora", "Turnos", "", None),
    ];

    assert_eq!(ids("REVISION", &entries), ["a"]);
    assert_eq!(ids("sel rev", &entries), ["a"]);
    assert_eq!(ids("zz", &entries), Vec::<String>::new());
}

// PLAN §3.5: a word inside the name counts as a prefix
#[test]
fn the_start_of_a_word_inside_the_name_is_a_prefix() {
    let entries = [
        entry("fuzzy", "Capitán", "Demo", "", Some(50)),
        entry("word", "Revisar la api", "Demo", "", Some(1)),
    ];

    assert_eq!(ids("api", &entries), ["word", "fuzzy"]);
}

// PLAN §3.5: with nothing typed, every session, the most recent first
#[test]
fn an_empty_search_lists_every_session_by_recent_activity() {
    let entries = [
        entry("old", "Uno", "Demo", "", Some(1)),
        entry("new", "Dos", "Demo", "", Some(9)),
    ];

    assert_eq!(ids("  ", &entries), ["new", "old"]);
}

// PLAN §3.5: con 100 sesiones, sin retraso visible. The criterion: 100
// sessions, as the plan says, within one frame (16 ms), in a debug build
// and with the other tests running at once. Measured on 22-nov-2026 at
// about 1 ms (9 ms for 1,000), so the margin is wide on purpose: it fails
// on a change of the order of things, not on a busy machine.
#[test]
fn ranking_a_hundred_sessions_takes_less_than_a_frame() {
    let entries: Vec<Entry> = (0..100)
        .map(|n| {
            entry(
                &n.to_string(),
                &format!("session number {n} of the planner"),
                &format!("Category {}", n % 17),
                "implements the phases of the plan",
                Some(n),
            )
        })
        .collect();

    let started = std::time::Instant::now();
    let index = Index::new(entries);
    for query in ["p", "pl", "pla", "plan", "plnr", "category 3 session"] {
        assert!(!index.rank(query).is_empty());
    }
    let per_query = started.elapsed() / 6;
    assert!(per_query.as_millis() < 16, "{per_query:?} per query");
}

// ronda 3B punto 14: an accent written after its letter (NFD, as macOS
// may give names) is no accent either
#[test]
fn accents_written_after_their_letter_are_ignored_too() {
    let entries = [
        entry("nfd", "Revisio\u{301}n", "Demo", "", None),
        entry("nfc", "Migración", "Demo", "", None),
    ];

    assert_eq!(ids("revision", &entries), ["nfd"]);
    assert_eq!(ids("migracio\u{301}n", &entries), ["nfc"]);
}
