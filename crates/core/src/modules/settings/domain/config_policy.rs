use crate::modules::settings::domain::setting_key::SettingKey;

/// The settings the command line knows: the same ones the settings window
/// changes (PLAN §2.6), and `last_claude_path`, which the launcher writes
/// and can only be read or unset from here.
pub const CONFIGURABLE_KEYS: [SettingKey; 10] = [
    SettingKey::ClaudePath,
    SettingKey::ContextLanguage,
    SettingKey::Language,
    SettingKey::HibernateAfterHours,
    SettingKey::MemoryValve,
    SettingKey::Scrollback,
    SettingKey::FontFamily,
    SettingKey::FontSize,
    SettingKey::Theme,
    SettingKey::LastClaudePath,
];

pub const HIBERNATE_HOURS: std::ops::RangeInclusive<i64> = 1..=8760;
pub const SCROLLBACK_LINES: std::ops::RangeInclusive<i64> = 100..=100_000;
pub const FONT_SIZES: std::ops::RangeInclusive<i64> = 8..=32;
const MAX_FONT_FAMILY_CHARS: usize = 80;

pub fn configurable_key(name: &str) -> Option<SettingKey> {
    CONFIGURABLE_KEYS
        .into_iter()
        .find(|key| key.as_str() == name)
}

/// Returns the value as it should be stored, or what is wrong with it.
pub fn checked_value(key: SettingKey, value: &str) -> Result<String, &'static str> {
    let value = value.trim();
    match key {
        SettingKey::ClaudePath if value.starts_with('/') => Ok(value.to_string()),
        SettingKey::ClaudePath => Err("expected an absolute path"),
        SettingKey::ContextLanguage | SettingKey::Language => {
            one_of(value, &["es", "en"]).ok_or("expected `es` or `en`")
        }
        SettingKey::HibernateAfterHours => {
            whole_number_in(value, &HIBERNATE_HOURS).ok_or("expected hours, from 1 to 8760")
        }
        SettingKey::MemoryValve => one_of(value, &["on", "off"]).ok_or("expected `on` or `off`"),
        SettingKey::Scrollback => {
            whole_number_in(value, &SCROLLBACK_LINES).ok_or("expected lines, from 100 to 100000")
        }
        SettingKey::FontSize => {
            whole_number_in(value, &FONT_SIZES).ok_or("expected a size, from 8 to 32")
        }
        SettingKey::FontFamily => font_family(value).ok_or("expected the name of a font"),
        SettingKey::Theme => one_of(value, &["light", "dark", "system"])
            .ok_or("expected `light`, `dark` or `system`"),
        SettingKey::LastClaudePath => Err("lithemux writes this one; it can only be read or unset"),
    }
}

fn one_of(value: &str, allowed: &[&str]) -> Option<String> {
    let value = value.to_ascii_lowercase();
    allowed.contains(&value.as_str()).then_some(value)
}

fn whole_number_in(value: &str, range: &std::ops::RangeInclusive<i64>) -> Option<String> {
    let number: i64 = value.parse().ok()?;
    range.contains(&number).then(|| number.to_string())
}

fn font_family(value: &str) -> Option<String> {
    let is_a_name = !value.is_empty()
        && value.chars().count() <= MAX_FONT_FAMILY_CHARS
        && value.chars().all(|c| !c.is_control());
    is_a_name.then(|| value.to_string())
}
