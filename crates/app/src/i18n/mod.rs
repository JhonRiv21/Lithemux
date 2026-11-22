//! The texts of the interface, in English or Spanish. They live in
//! `locales/*.toml`, never in the code; `texts` looks them up. A `message`
//! is a text before it has a language, and `problems` turns the errors of
//! the app into messages.

mod language;
mod message;
mod problems;
mod texts;

#[cfg(test)]
mod test;

pub use language::Language;
pub use message::Message;
pub use texts::Texts;
