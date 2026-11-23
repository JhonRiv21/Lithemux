//! PLAN §2.6: «Idioma de la interfaz… Otros ajustes: `context_language`
//! (por defecto, el de la interfaz); horas para hibernar y la válvula de
//! memoria; scrollback; fuente y tamaño; `claude_path` (opcional); tema
//! claro, oscuro o del sistema.»

use lithemux_core::modules::launch::PathKind;
use lithemux_core::modules::settings::{Preferences, SettingKey, checked_value};

use crate::i18n::{Language, Texts};
use crate::settings_window::form::{Input, rows, value_of, warning_of};

#[test]
fn the_window_has_every_setting_of_the_plan() {
    let shown: Vec<SettingKey> = rows().into_iter().map(|row| row.key).collect();
    for key in [
        SettingKey::Language,
        SettingKey::ContextLanguage,
        SettingKey::HibernateAfterHours,
        SettingKey::MemoryValve,
        SettingKey::Scrollback,
        SettingKey::FontFamily,
        SettingKey::FontSize,
        SettingKey::ClaudePath,
        SettingKey::Theme,
    ] {
        assert_eq!(
            shown.iter().filter(|shown| **shown == key).count(),
            1,
            "{key:?}"
        );
    }
    // What the launcher writes for itself is not the user's to set.
    assert!(!shown.contains(&SettingKey::LastClaudePath));
}

// Whatever the window offers to choose is a value the setting accepts; the
// empty one means «unset», which the window does by removing the setting.
#[test]
fn every_choice_is_a_value_the_setting_accepts() {
    for row in rows() {
        let Input::Choices(choices) = &row.input else {
            continue;
        };
        assert!(choices.len() >= 2, "{:?}", row.key);
        for choice in choices.iter().filter(|choice| !choice.value.is_empty()) {
            assert_eq!(
                checked_value(row.key, choice.value).as_deref(),
                Ok(choice.value),
                "{:?}",
                row.key
            );
        }
    }
}

// «tema claro, oscuro o del sistema»; «inglés por defecto, español opcional»
#[test]
fn the_choices_are_the_ones_of_the_plan() {
    let values = |key: SettingKey| -> Vec<&'static str> {
        let row = rows().into_iter().find(|row| row.key == key).unwrap();
        match row.input {
            Input::Choices(choices) => choices.iter().map(|choice| choice.value).collect(),
            Input::Text => Vec::new(),
        }
    };
    let mut themes = values(SettingKey::Theme);
    themes.sort_unstable();
    assert_eq!(themes, ["dark", "light", "system"]);
    assert_eq!(values(SettingKey::Language), ["en", "es"]);
    // «por defecto, el de la interfaz»
    assert_eq!(values(SettingKey::ContextLanguage), ["", "en", "es"]);
}

// With nothing set, each row shows the default: the value that is in use.
#[test]
fn a_row_shows_the_value_in_use() {
    let defaults = Preferences::default();
    assert_eq!(value_of(&defaults, SettingKey::Language), "en");
    assert_eq!(value_of(&defaults, SettingKey::ContextLanguage), "");
    assert_eq!(value_of(&defaults, SettingKey::Theme), "system");
    assert_eq!(value_of(&defaults, SettingKey::HibernateAfterHours), "48");
    assert_eq!(value_of(&defaults, SettingKey::MemoryValve), "on");
    assert_eq!(value_of(&defaults, SettingKey::Scrollback), "5000");
    assert_eq!(value_of(&defaults, SettingKey::ClaudePath), "");
}

// What a row shows can be stored back as it is.
#[test]
fn what_a_row_shows_is_a_value_its_setting_accepts() {
    let defaults = Preferences::default();
    for row in rows() {
        let shown = value_of(&defaults, row.key);
        if !shown.is_empty() {
            assert_eq!(
                checked_value(row.key, &shown),
                Ok(shown.clone()),
                "{:?}",
                row.key
            );
        }
    }
}

// PLAN §2.6: no text in the code; the whole interface in both languages.
#[test]
fn every_label_hint_and_choice_has_its_text_in_both_languages() {
    let (english, spanish) = (Texts::of(Language::English), Texts::of(Language::Spanish));
    for row in rows() {
        let mut keys = vec![row.label, row.hint];
        if let Input::Choices(choices) = &row.input {
            keys.extend(choices.iter().map(|choice| choice.label));
        }
        for key in keys {
            assert_ne!(english.get(key), key, "{key}");
            assert!(spanish.is_translated(key), "{key}");
        }
    }
}

// ronda 3C: Settings shows a claude_path that cannot run as it is stored,
// with why and how to fix it, in the words of the card.
#[test]
fn a_claude_path_that_cannot_run_is_shown_with_why() {
    let preferences = Preferences {
        claude_path: Some("\"/opt/claude\"".to_string()),
        ..Preferences::default()
    };
    let runs = |_: &std::path::Path| PathKind::File { executable: true };

    assert_eq!(
        value_of(&preferences, SettingKey::ClaudePath),
        "\"/opt/claude\""
    );
    let warning = warning_of(&preferences, SettingKey::ClaudePath, runs).unwrap();
    let said = Texts::of(Language::Spanish).say(&warning);
    assert!(said.contains("comillas"), "{said}");
    assert!(said.contains("vacía"), "{said}");
}

// ronda 3C: nothing to say of a path that runs, of an unset one, or of the
// other settings.
#[test]
fn a_claude_path_that_runs_or_is_unset_has_no_warning() {
    let runs = |_: &std::path::Path| PathKind::File { executable: true };
    let set = Preferences {
        claude_path: Some("/opt/claude".to_string()),
        ..Preferences::default()
    };

    assert!(warning_of(&set, SettingKey::ClaudePath, runs).is_none());
    assert!(warning_of(&Preferences::default(), SettingKey::ClaudePath, runs).is_none());
    assert!(warning_of(&set, SettingKey::Theme, |_| PathKind::Missing).is_none());
}
