use crate::i18n::{Language, Texts};

// PLAN §2.6: English by default, Spanish optional.
#[test]
fn the_same_key_reads_in_each_language() {
    assert_eq!(
        Texts::of(Language::English).get("status.opening"),
        "opening"
    );
    assert_eq!(
        Texts::of(Language::Spanish).get("status.opening"),
        "abriendo"
    );
}

// PLAN §2.2 (carried from 2.1): «closing» / «cerrando».
#[test]
fn closing_has_its_text_in_both_languages() {
    assert_eq!(
        Texts::of(Language::English).get("status.closing"),
        "closing"
    );
    assert_eq!(
        Texts::of(Language::Spanish).get("status.closing"),
        "cerrando"
    );
}

// PLAN §2.6: a key missing in Spanish uses the English one.
#[test]
fn a_text_missing_in_the_chosen_language_reads_in_english() {
    let texts = Texts::from_files(
        "[menu]\nopen = \"Open\"\nclose = \"Close\"\n",
        "[menu]\nopen = \"Abrir\"\n",
    );
    assert_eq!(texts.get("menu.open"), "Abrir");
    assert!(!texts.is_translated("menu.close"));
    assert_eq!(texts.get("menu.close"), "Close");
    // A key nobody has shows itself, so that the gap is seen.
    assert_eq!(texts.get("menu.missing"), "menu.missing");
}

// PLAN §2.6: the whole interface works in Spanish and in English.
#[test]
fn every_text_is_translated() {
    let spanish = Texts::of(Language::Spanish);
    let mut missing: Vec<&str> = spanish
        .keys()
        .filter(|key| !spanish.is_translated(key))
        .collect();
    missing.sort_unstable();
    assert_eq!(missing, Vec::<&str>::new());
}

// PLAN §2.6
#[test]
fn values_take_their_place_in_the_text() {
    let english = Texts::of(Language::English);
    assert_eq!(
        english.with("status.open_elsewhere_pid", &[("pid", "42")]),
        "open in another terminal (pid 42)"
    );
}

// PLAN §2.6
#[test]
fn an_unknown_language_setting_is_english() {
    assert_eq!(Language::from_setting(None), Language::English);
    assert_eq!(Language::from_setting(Some("fr")), Language::English);
    assert_eq!(Language::from_setting(Some("es")), Language::Spanish);
}
