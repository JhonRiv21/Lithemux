/// A text of the interface before it is put in a language: its key in
/// `locales/*.toml` and the values that go in its `{places}`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Message {
    pub key: &'static str,
    pub values: Vec<(&'static str, String)>,
}

impl Message {
    pub fn plain(key: &'static str) -> Self {
        Message {
            key,
            values: Vec::new(),
        }
    }

    pub fn with(key: &'static str, name: &'static str, value: impl Into<String>) -> Self {
        Message {
            key,
            values: vec![(name, value.into())],
        }
    }

    pub fn and(mut self, name: &'static str, value: impl Into<String>) -> Self {
        self.values.push((name, value.into()));
        self
    }
}
