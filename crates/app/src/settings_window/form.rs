//! What the settings window shows: one row per setting, as a choice among
//! a few values or as a text to type. No gpui.

use std::path::Path;

use lithemux_core::modules::launch::{PathKind, claude_path_problem};
use lithemux_core::modules::settings::{InterfaceLanguage, Preferences, SettingKey, ThemeChoice};

use crate::i18n::{Message, claude_path_message};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Choice {
    /// What is stored; empty unsets the setting.
    pub value: &'static str,
    /// The key of its text in `locales/*.toml`.
    pub label: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Input {
    Choices(Vec<Choice>),
    /// Typed, and applied with Enter. Empty goes back to the default.
    Text,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Row {
    pub key: SettingKey,
    pub label: &'static str,
    /// A line under the label that says what the setting does.
    pub hint: &'static str,
    pub input: Input,
}

fn choices(
    key: SettingKey,
    label: &'static str,
    hint: &'static str,
    of: &[(&'static str, &'static str)],
) -> Row {
    let options = of.iter().map(|(value, label)| Choice { value, label });
    Row {
        key,
        label,
        hint,
        input: Input::Choices(options.collect()),
    }
}

fn text(key: SettingKey, label: &'static str, hint: &'static str) -> Row {
    Row {
        key,
        label,
        hint,
        input: Input::Text,
    }
}

const LANGUAGES: [(&str, &str); 2] = [("en", "settings.english"), ("es", "settings.spanish")];
const CONTEXT_LANGUAGES: [(&str, &str); 3] = [
    ("", "settings.same_as_interface"),
    ("en", "settings.english"),
    ("es", "settings.spanish"),
];
const THEMES: [(&str, &str); 3] = [
    ("system", "settings.theme_system"),
    ("light", "settings.theme_light"),
    ("dark", "settings.theme_dark"),
];
const SWITCH: [(&str, &str); 2] = [("on", "settings.on"), ("off", "settings.off")];

/// PLAN §2.6, in the order of the window. The texts of a setting are
/// `settings.<key>` and `settings.<key>_hint`.
pub fn rows() -> Vec<Row> {
    use SettingKey::{
        ClaudePath, ContextLanguage, FontFamily, FontSize, HibernateAfterHours, Language,
        MemoryValve, Scrollback, Theme, VersionCheckHours,
    };
    let row = |key: SettingKey, choices_of: &[(&'static str, &'static str)]| {
        let (label, hint) = texts_of(key);
        if choices_of.is_empty() {
            text(key, label, hint)
        } else {
            choices(key, label, hint, choices_of)
        }
    };
    vec![
        row(Language, &LANGUAGES),
        row(ContextLanguage, &CONTEXT_LANGUAGES),
        row(Theme, &THEMES),
        row(FontFamily, &[]),
        row(FontSize, &[]),
        row(Scrollback, &[]),
        row(HibernateAfterHours, &[]),
        row(MemoryValve, &SWITCH),
        row(ClaudePath, &[]),
        row(VersionCheckHours, &[]),
    ]
}

/// The label and the hint of a setting: `settings.<key>` and
/// `settings.<key>_hint` in the locales.
fn texts_of(key: SettingKey) -> (&'static str, &'static str) {
    match key {
        SettingKey::Language => ("settings.language", "settings.language_hint"),
        SettingKey::ContextLanguage => (
            "settings.context_language",
            "settings.context_language_hint",
        ),
        SettingKey::Theme => ("settings.theme", "settings.theme_hint"),
        SettingKey::FontFamily => ("settings.font_family", "settings.font_family_hint"),
        SettingKey::FontSize => ("settings.font_size", "settings.font_size_hint"),
        SettingKey::Scrollback => ("settings.scrollback", "settings.scrollback_hint"),
        SettingKey::HibernateAfterHours => (
            "settings.hibernate_after_hours",
            "settings.hibernate_after_hours_hint",
        ),
        SettingKey::MemoryValve => ("settings.memory_valve", "settings.memory_valve_hint"),
        SettingKey::ClaudePath => ("settings.claude_path", "settings.claude_path_hint"),
        SettingKey::VersionCheckHours => (
            "settings.version_check_hours",
            "settings.version_check_hours_hint",
        ),
        SettingKey::LastClaudePath => ("settings.claude_path", "settings.claude_path_hint"),
    }
}

fn language(language: InterfaceLanguage) -> &'static str {
    match language {
        InterfaceLanguage::English => "en",
        InterfaceLanguage::Spanish => "es",
    }
}

/// What a row shows as the current value: what would be stored for it.
pub fn value_of(preferences: &Preferences, key: SettingKey) -> String {
    match key {
        SettingKey::Language => language(preferences.language).to_string(),
        SettingKey::ContextLanguage => preferences
            .context_language
            .map(language)
            .unwrap_or_default()
            .to_string(),
        SettingKey::Theme => match preferences.theme {
            ThemeChoice::System => "system",
            ThemeChoice::Light => "light",
            ThemeChoice::Dark => "dark",
        }
        .to_string(),
        SettingKey::FontFamily => preferences.font_family.clone(),
        SettingKey::FontSize => preferences.font_size.to_string(),
        SettingKey::Scrollback => preferences.scrollback.to_string(),
        SettingKey::HibernateAfterHours => preferences.hibernate_after_hours.to_string(),
        SettingKey::MemoryValve => if preferences.memory_valve {
            "on"
        } else {
            "off"
        }
        .to_string(),
        SettingKey::ClaudePath => preferences.claude_path.clone().unwrap_or_default(),
        SettingKey::VersionCheckHours => preferences.version_check_hours.to_string(),
        SettingKey::LastClaudePath => String::new(),
    }
}

/// What is wrong with what the row shows, said as a launch would say it.
/// Only `claude_path` is kept as stored when it cannot be used: `kind_of`
/// says what is at its path.
pub fn warning_of(
    preferences: &Preferences,
    key: SettingKey,
    kind_of: impl FnOnce(&Path) -> PathKind,
) -> Option<Message> {
    if key != SettingKey::ClaudePath {
        return None;
    }
    let value = preferences.claude_path.as_deref()?;
    let problem = claude_path_problem(value, kind_of)?;
    Some(claude_path_message(value, problem))
}
