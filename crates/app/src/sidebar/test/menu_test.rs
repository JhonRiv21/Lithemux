//! PLAN §2.2, last point: «Crear, renombrar, reordenar y borrar categorías y
//! sesiones. Marcar una sesión como coordinadora (respetando una por categoría).
//! Activar `keep_alive`.» Plus: pinned sessions, and shell sessions have a
//! card too. Each test is one of those sentences, asked of the menu.

use lithemux_core::modules::session::Profile;

use crate::i18n::{Language, Texts};
use crate::sidebar::menu::{MenuCommand, MenuItem, card_menu, category_menu};
use crate::sidebar::status::Status;
use crate::sidebar::tree::{Card, Group};

fn card() -> Card {
    Card {
        session_id: "s1".to_string(),
        address: "demo/api".to_string(),
        category_id: "c1".to_string(),
        title: "api".to_string(),
        preview: None,
        status: Status::Ended,
        unread: 0,
        cwd: "~/demo".to_string(),
        ports: Vec::new(),
        is_coordinator: false,
        is_pinned: false,
        keep_alive: false,
        is_shell: false,
    }
}

fn group() -> Group {
    Group {
        category_id: "c1".to_string(),
        name: "Demo".to_string(),
        collapsed: false,
        header: None,
        cards: Vec::new(),
    }
}

/// Whether the menu lets the user do something, said as a pattern over
/// what the entry asks for.
fn offers(menu: &[MenuItem], wanted: impl Fn(&MenuCommand) -> bool) -> bool {
    menu.iter().any(|item| wanted(&item.command))
}

// «renombrar… y borrar… sesiones»
#[test]
fn a_session_can_be_renamed_and_deleted_from_its_card() {
    let menu = card_menu(&card());
    assert!(offers(
        &menu,
        |c| matches!(c, MenuCommand::RenameSession(id) if id == "s1")
    ));
    assert!(offers(
        &menu,
        |c| matches!(c, MenuCommand::DeleteSession(id) if id == "s1")
    ));
}

// «reordenar… sesiones»: both ways.
#[test]
fn a_session_can_be_moved_up_and_down_from_its_card() {
    let menu = card_menu(&card());
    assert!(offers(
        &menu,
        |c| matches!(c, MenuCommand::ShiftSession(id, by) if id == "s1" && *by < 0)
    ));
    assert!(offers(
        &menu,
        |c| matches!(c, MenuCommand::ShiftSession(id, by) if id == "s1" && *by > 0)
    ));
}

// «Marcar una sesión como coordinadora»: what is offered is the change from what
// the session is now, never both.
#[test]
fn a_session_can_be_made_coordinator_and_a_coordinator_can_stop_being_one() {
    let child = card_menu(&card());
    assert!(offers(&child, |c| matches!(
        c,
        MenuCommand::SetCoordinator(_, true)
    )));
    assert!(!offers(&child, |c| matches!(
        c,
        MenuCommand::SetCoordinator(_, false)
    )));
    let coordinator = card_menu(&Card {
        is_coordinator: true,
        ..card()
    });
    assert!(offers(&coordinator, |c| matches!(
        c,
        MenuCommand::SetCoordinator(_, false)
    )));
    assert!(!offers(&coordinator, |c| matches!(
        c,
        MenuCommand::SetCoordinator(_, true)
    )));
}

// «Activar `keep_alive`», and turning it off again.
#[test]
fn keep_alive_can_be_turned_on_and_off() {
    let off = card_menu(&card());
    assert!(offers(&off, |c| matches!(
        c,
        MenuCommand::SetKeepAlive(_, true)
    )));
    let on = card_menu(&Card {
        keep_alive: true,
        ..card()
    });
    assert!(offers(&on, |c| matches!(
        c,
        MenuCommand::SetKeepAlive(_, false)
    )));
    assert!(!offers(&on, |c| matches!(
        c,
        MenuCommand::SetKeepAlive(_, true)
    )));
}

// «Sesiones fijadas (pin) arriba»: a session gets there, and back, from
// its card.
#[test]
fn a_session_can_be_pinned_and_unpinned() {
    let free = card_menu(&card());
    assert!(offers(&free, |c| matches!(
        c,
        MenuCommand::SetPinned(_, true)
    )));
    let pinned = card_menu(&Card {
        is_pinned: true,
        ..card()
    });
    assert!(offers(&pinned, |c| matches!(
        c,
        MenuCommand::SetPinned(_, false)
    )));
    assert!(!offers(&pinned, |c| matches!(
        c,
        MenuCommand::SetPinned(_, true)
    )));
}

// «Las sesiones de shell también tienen tarjeta.» The coordinator coordinates
// Claude sessions (§0.2) and keep_alive exempts one from hibernation
// (§2.4): neither means anything for a shell.
#[test]
fn a_shell_card_offers_what_makes_sense_for_a_shell() {
    let menu = card_menu(&Card {
        is_shell: true,
        ..card()
    });
    assert!(!offers(&menu, |c| matches!(
        c,
        MenuCommand::SetCoordinator(..)
    )));
    assert!(!offers(&menu, |c| matches!(
        c,
        MenuCommand::SetKeepAlive(..)
    )));
    assert!(offers(&menu, |c| matches!(
        c,
        MenuCommand::RenameSession(_)
    )));
    assert!(offers(&menu, |c| matches!(
        c,
        MenuCommand::DeleteSession(_)
    )));
    assert!(offers(&menu, |c| matches!(c, MenuCommand::SetPinned(..))));
}

// A card opens its session (§2.3); closing is for one that has a program.
#[test]
fn a_card_opens_its_session_and_closes_it_only_while_it_has_a_program() {
    let closes = |status: Status| {
        let menu = card_menu(&Card { status, ..card() });
        assert!(offers(
            &menu,
            |c| matches!(c, MenuCommand::Open(id) if id == "s1")
        ));
        offers(&menu, |c| matches!(c, MenuCommand::Close(_)))
    };
    assert!(closes(Status::Idle));
    assert!(closes(Status::Working));
    assert!(closes(Status::Opening));
    assert!(!closes(Status::Ended));
    assert!(!closes(Status::Hibernated));
    assert!(!closes(Status::NotStarted));
}

// «Crear… sesiones»: in a category, of either kind.
#[test]
fn sessions_of_both_kinds_are_created_from_their_category() {
    let menu = category_menu(&group());
    assert!(offers(
        &menu,
        |c| matches!(c, MenuCommand::NewSession(id, Profile::Claude) if id == "c1")
    ));
    assert!(offers(
        &menu,
        |c| matches!(c, MenuCommand::NewSession(id, Profile::Shell) if id == "c1")
    ));
}

// «renombrar, reordenar y borrar categorías»
#[test]
fn a_category_can_be_renamed_moved_and_deleted() {
    let menu = category_menu(&group());
    assert!(offers(
        &menu,
        |c| matches!(c, MenuCommand::RenameCategory(id) if id == "c1")
    ));
    assert!(offers(
        &menu,
        |c| matches!(c, MenuCommand::ShiftCategory(_, by) if *by < 0)
    ));
    assert!(offers(
        &menu,
        |c| matches!(c, MenuCommand::ShiftCategory(_, by) if *by > 0)
    ));
    assert!(offers(
        &menu,
        |c| matches!(c, MenuCommand::DeleteCategory(id) if id == "c1")
    ));
}

// «Categorías colapsables»
#[test]
fn a_category_can_be_collapsed_and_expanded_again() {
    let open = category_menu(&group());
    assert!(offers(&open, |c| matches!(
        c,
        MenuCommand::SetCollapsed(_, true)
    )));
    let folded = category_menu(&Group {
        collapsed: true,
        ..group()
    });
    assert!(offers(&folded, |c| matches!(
        c,
        MenuCommand::SetCollapsed(_, false)
    )));
    assert!(!offers(&folded, |c| matches!(
        c,
        MenuCommand::SetCollapsed(_, true)
    )));
}

// PLAN §2.6: no text in the code. Whatever a menu shows, in any state, has
// its text in both languages.
#[test]
fn every_entry_of_every_menu_has_its_text_in_both_languages() {
    let cards = [
        card(),
        Card {
            is_shell: true,
            ..card()
        },
        Card {
            is_pinned: true,
            is_coordinator: true,
            keep_alive: true,
            status: Status::Idle,
            ..card()
        },
    ];
    let menus = cards
        .iter()
        .map(card_menu)
        .chain([category_menu(&group())])
        .chain([category_menu(&Group {
            collapsed: true,
            ..group()
        })]);
    let (english, spanish) = (Texts::of(Language::English), Texts::of(Language::Spanish));
    for item in menus.flatten() {
        assert_ne!(english.get(item.label), item.label, "{}", item.label);
        assert!(spanish.is_translated(item.label), "{}", item.label);
    }
}
