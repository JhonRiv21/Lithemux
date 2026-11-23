use super::fixtures::{HOME, category, claude_panel, conversation, session};
use crate::modules::category::Category;
use crate::modules::import::domain::cmux_panel::CmuxPanel;
use crate::modules::import::domain::proposal::{Destination, Proposal};
use crate::modules::import::domain::proposal_policy::{Known, propose};
use crate::modules::session::Session;

fn every_conversation_is_kept(_: &str) -> bool {
    true
}

fn propose_against(
    panels: &[CmuxPanel],
    categories: &[Category],
    sessions: &[Session],
) -> Proposal {
    propose(
        panels,
        &Known {
            categories,
            sessions,
            home: Some(HOME),
            has_conversation: &every_conversation_is_kept,
        },
    )
}

// PLAN §3.4: nombre = customTitle; si no hay, processTitle; si no, la carpeta
#[test]
fn the_name_is_the_custom_title_then_the_process_title_then_the_folder() {
    let mut named = claude_panel(1, "/work/alpha");
    named.custom_title = Some("Planner".to_string());
    let titled = claude_panel(2, "/work/alpha");
    let mut bare = claude_panel(3, "/work/beta-app");
    bare.process_title = None;

    let proposal = propose_against(&[named, titled, bare], &[], &[]);

    let names: Vec<_> = proposal.sessions.iter().map(|s| s.name.as_str()).collect();
    assert_eq!(names, ["Planner", "task 2", "beta-app"]);
    let slugs: Vec<_> = proposal.sessions.iter().map(|s| s.slug.as_str()).collect();
    assert_eq!(slugs, ["planner", "task-2", "beta-app"]);
}

// PLAN §3.4: an empty title is no title; the spinner Claude puts in front of
// its title is not part of the name
#[test]
fn a_blank_title_falls_through_and_a_leading_symbol_is_dropped() {
    let mut blank = claude_panel(1, "/work/alpha");
    blank.custom_title = Some("   ".to_string());
    blank.process_title = Some("✳ Fix the login".to_string());

    let proposal = propose_against(&[blank], &[], &[]);

    assert_eq!(proposal.sessions[0].name, "Fix the login");
}

// PLAN §3.4: si la carpeta cae dentro de la raíz de una categoría, esa
#[test]
fn a_folder_inside_a_category_goes_to_that_category_the_deepest_one() {
    let work = category("work", "/work");
    let alpha = category("alpha", "/work/alpha");
    let panels = [
        claude_panel(1, "/work/alpha/api"),
        claude_panel(2, "/work/gamma"),
        claude_panel(3, "/work/alpha"),
    ];

    let proposal = propose_against(&panels, &[work, alpha], &[]);

    let destinations: Vec<_> = proposal.sessions.iter().map(|s| &s.destination).collect();
    let existing = |id: &str| Destination::Existing {
        category_id: id.to_string(),
    };
    assert_eq!(
        destinations,
        [
            &existing("id-alpha"),
            &existing("id-work"),
            &existing("id-alpha")
        ]
    );
    assert!(proposal.new_categories.is_empty());
}

// PLAN §3.4: a folder that only looks like a category's root by its letters
// is not inside it
#[test]
fn a_folder_that_only_shares_letters_with_a_root_is_not_inside_it() {
    let alpha = category("alpha", "/work/alpha");

    let proposal = propose_against(&[claude_panel(1, "/work/alphabet")], &[alpha], &[]);

    assert_eq!(
        proposal.sessions[0].destination,
        Destination::New {
            root_dir: "/work/alphabet".to_string()
        }
    );
}

// PLAN §3.4: si no, una nueva por carpeta
#[test]
fn a_folder_outside_every_category_gets_one_new_category_for_all_its_sessions() {
    let existing = category("beta", "/elsewhere");
    let panels = [
        claude_panel(1, "/work/beta"),
        claude_panel(2, "/work/beta"),
        claude_panel(3, "/work/delta"),
    ];

    let proposal = propose_against(&panels, &[existing], &[]);

    let new: Vec<_> = proposal
        .new_categories
        .iter()
        .map(|c| (c.slug.as_str(), c.name.as_str(), c.root_dir.as_str()))
        .collect();
    // «beta» is taken by the existing category.
    assert_eq!(
        new,
        [
            ("beta-2", "beta", "/work/beta"),
            ("delta", "delta", "/work/delta")
        ]
    );
    assert_eq!(
        proposal.sessions[1].destination,
        Destination::New {
            root_dir: "/work/beta".to_string()
        }
    );
}

// PLAN §3.4: si la carpeta es el home, sin categoría y el usuario elige
#[test]
fn a_session_of_the_home_folder_has_no_category_until_the_user_chooses() {
    let home_category = category("everything", "/Users");
    let panels = [claude_panel(1, HOME), claude_panel(2, &format!("{HOME}/"))];

    let proposal = propose_against(&panels, &[home_category], &[]);

    assert!(
        proposal
            .sessions
            .iter()
            .all(|s| s.destination == Destination::Unassigned)
    );
    assert!(proposal.new_categories.is_empty());
}

// PLAN §3.4: reimportar no duplica
#[test]
fn a_conversation_lithemux_already_has_is_left_out_and_counted() {
    let alpha = category("alpha", "/work/alpha");
    let there = session(&alpha, "planner", Some(conversation(1)));
    let panels = [
        claude_panel(1, "/work/alpha"),
        claude_panel(2, "/work/alpha"),
        claude_panel(2, "/work/alpha"),
    ];

    let proposal = propose_against(&panels, &[alpha], &[there]);

    let ids: Vec<_> = proposal
        .sessions
        .iter()
        .map(|s| s.claude_session_id.clone())
        .collect();
    assert_eq!(ids, [conversation(2)]);
    assert_eq!(proposal.already_imported, 2);
}

// PLAN §3.4: se omiten los paneles sin sessionId (and those without Claude)
#[test]
fn panels_without_a_conversation_or_without_claude_are_left_out_and_counted() {
    let mut without_id = claude_panel(1, "/work/alpha");
    without_id.claude_session_id = None;
    let mut not_an_id = claude_panel(2, "/work/alpha");
    not_an_id.claude_session_id = Some("not-a-uuid".to_string());
    let mut a_shell = claude_panel(3, "/work/alpha");
    a_shell.agent = None;
    let mut codex = claude_panel(4, "/work/alpha");
    codex.agent = Some("codex".to_string());

    let proposal = propose_against(
        &[
            without_id,
            not_an_id,
            a_shell,
            codex,
            claude_panel(5, "/work/alpha"),
        ],
        &[],
        &[],
    );

    assert_eq!(proposal.sessions.len(), 1);
    assert_eq!(proposal.without_conversation_id, 2);
    assert_eq!(proposal.not_claude, 2);
    // Numbers follow cmux's file, so the CLI names them the same each time.
    assert_eq!(proposal.sessions[0].number, 5);
}

// PLAN §3.4: las que no tienen conversación guardada salen marcadas y
// desmarcadas por defecto
#[test]
fn a_session_without_a_saved_conversation_is_marked_and_not_selected() {
    let kept = conversation(1);
    let has_conversation = |id: &str| id == kept;

    let proposal = propose(
        &[claude_panel(1, "/work/a"), claude_panel(2, "/work/a")],
        &Known {
            categories: &[],
            sessions: &[],
            home: Some(HOME),
            has_conversation: &has_conversation,
        },
    );

    let flags: Vec<_> = proposal
        .sessions
        .iter()
        .map(|s| (s.has_conversation, s.selected))
        .collect();
    assert_eq!(flags, [(true, true), (false, false)]);
}

// PLAN §3.4: si el workspace estaba fijado, la sesión sale fijada; la carpeta
// es la de trabajo
#[test]
fn a_pinned_workspace_gives_a_pinned_session_in_its_folder() {
    let mut pinned = claude_panel(1, "/work/alpha/api");
    pinned.pinned = true;

    let proposal = propose_against(&[pinned, claude_panel(2, "/work/b")], &[], &[]);

    assert!(proposal.sessions[0].pinned);
    assert!(!proposal.sessions[1].pinned);
    assert_eq!(proposal.sessions[0].folder, "/work/alpha/api");
}

// PLAN §3.4: el slug no choca con los existentes, ni entre las importadas,
// ni en el nombre con que Claude las direcciona
#[test]
fn slugs_never_collide_with_what_exists_or_with_each_other() {
    let alpha = category("alpha", "/work/alpha");
    let alpha_x = category("alpha-x", "/work/elsewhere");
    let mut panels = Vec::new();
    for n in 1..=3 {
        let mut panel = claude_panel(n, "/work/alpha");
        panel.custom_title = Some("Planner".to_string());
        panels.push(panel);
    }
    // «alpha-x-y», what alpha/x-y would be called, is alpha-x/y already.
    let mut clash = claude_panel(4, "/work/alpha");
    clash.custom_title = Some("x y".to_string());
    panels.push(clash);
    let sessions = [
        session(&alpha, "planner", None),
        session(&alpha_x, "y", None),
    ];

    let proposal = propose_against(&panels, &[alpha, alpha_x], &sessions);

    let slugs: Vec<_> = proposal.sessions.iter().map(|s| s.slug.as_str()).collect();
    assert_eq!(slugs, ["planner-2", "planner-3", "planner-4", "x-y-2"]);
}

// ronda 3B, punto 10: a category rooted at the home folder (or above it)
// holds everything; a folder inside it still gets its own category, as if
// the home category were not there
#[test]
fn a_category_rooted_at_the_home_folder_does_not_swallow_its_subfolders() {
    let home = category("trabajo", HOME);
    let above = category("todo", "/Users");
    let inside = format!("{HOME}/work/memory");

    let proposal = propose_against(&[claude_panel(1, &inside)], &[home, above], &[]);

    assert_eq!(
        proposal.sessions[0].destination,
        Destination::New { root_dir: inside }
    );
}

// ronda 3B bug 1: names long enough to reach the limit still give two
// different slugs that can be opened
#[test]
fn two_long_names_give_two_valid_slugs_within_the_limit() {
    use crate::modules::category::Slug;
    let mut panels = Vec::new();
    for n in 1..=2 {
        let mut panel = claude_panel(n, "/work/alpha");
        panel.custom_title = Some("a very long name for a session ".repeat(3));
        panels.push(panel);
    }

    let proposal = propose_against(&panels, &[], &[]);

    let slugs: Vec<&str> = proposal.sessions.iter().map(|s| s.slug.as_str()).collect();
    assert_ne!(slugs[0], slugs[1]);
    assert!(
        slugs.iter().all(|slug| Slug::parse(slug).is_ok()),
        "{slugs:?}"
    );
}
