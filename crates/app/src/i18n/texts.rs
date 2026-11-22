use std::collections::HashMap;

use super::language::Language;
use super::message::Message;

const ENGLISH: &str = include_str!("../../../../locales/en.toml");
const SPANISH: &str = include_str!("../../../../locales/es.toml");

/// The texts of one language, by key (`section.name`).
#[derive(Debug, Clone)]
pub struct Texts {
    language: Language,
    chosen: HashMap<String, String>,
    english: HashMap<String, String>,
}

impl Texts {
    pub fn of(language: Language) -> Self {
        let texts = match language {
            Language::English => Self::from_files(ENGLISH, ""),
            Language::Spanish => Self::from_files(ENGLISH, SPANISH),
        };
        Texts { language, ..texts }
    }

    /// `english` is the reference; what `chosen` lacks is read from it.
    pub fn from_files(english: &str, chosen: &str) -> Self {
        Texts {
            language: Language::English,
            chosen: entries(chosen),
            english: entries(english),
        }
    }

    pub fn language(&self) -> Language {
        self.language
    }

    /// A message in this language.
    pub fn say(&self, message: &Message) -> String {
        let values: Vec<(&str, &str)> = message
            .values
            .iter()
            .map(|(name, value)| (*name, value.as_str()))
            .collect();
        self.with(message.key, &values)
    }

    /// The text of `key`: in the chosen language, else in English, else the
    /// key itself, so that a missing text shows where it is missing.
    pub fn get(&self, key: &str) -> String {
        self.chosen
            .get(key)
            .or_else(|| self.english.get(key))
            .cloned()
            .unwrap_or_else(|| key.to_string())
    }

    /// Like `get`, with each `{name}` replaced by its value.
    pub fn with(&self, key: &str, values: &[(&str, &str)]) -> String {
        values.iter().fold(self.get(key), |text, (name, value)| {
            text.replace(&format!("{{{name}}}"), value)
        })
    }

    pub fn keys(&self) -> impl Iterator<Item = &str> {
        self.english.keys().map(String::as_str)
    }

    pub fn is_translated(&self, key: &str) -> bool {
        self.chosen.contains_key(key)
    }
}

/// `[section]` tables of `name = "text"`, flattened to `section.name`.
/// A file that cannot be read gives no entries: the fallback takes over.
fn entries(file: &str) -> HashMap<String, String> {
    let Ok(table) = file.parse::<toml::Table>() else {
        return HashMap::new();
    };
    let mut found = HashMap::new();
    for (section, values) in &table {
        let Some(values) = values.as_table() else {
            continue;
        };
        for (name, text) in values {
            if let Some(text) = text.as_str() {
                found.insert(format!("{section}.{name}"), text.to_string());
            }
        }
    }
    found
}
