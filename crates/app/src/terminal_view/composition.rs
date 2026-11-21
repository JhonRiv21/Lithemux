//! Text the input method is composing (dead keys, the press-and-hold accent
//! menu, IMEs) and what reaches the PTY when it commits.

use std::ops::Range;

const DEL: u8 = 0x7f;

#[derive(Debug, Default)]
pub struct Composition {
    /// Last text sent to the PTY: the accent menu replaces it after the fact.
    committed: String,
    marked: String,
}

impl Composition {
    pub fn marked(&self) -> &str {
        &self.marked
    }

    /// What the input method sees as "the document".
    pub fn document(&self) -> String {
        format!("{}{}", self.committed, self.marked)
    }

    pub fn marked_range_utf16(&self) -> Option<Range<usize>> {
        if self.marked.is_empty() {
            return None;
        }
        let start = utf16_len(&self.committed);
        Some(start..start + utf16_len(&self.marked))
    }

    pub fn cursor_utf16(&self) -> usize {
        utf16_len(&self.committed) + utf16_len(&self.marked)
    }

    pub fn mark(&mut self, text: &str) {
        self.marked = text.to_string();
    }

    pub fn unmark(&mut self) {
        self.marked.clear();
    }

    /// A key that is not text went to the program: nothing typed before it can
    /// be replaced any more.
    pub fn reset(&mut self) {
        self.committed.clear();
        self.marked.clear();
    }

    /// Returns the bytes for the PTY.
    pub fn commit(&mut self, range_utf16: Option<Range<usize>>, text: &str) -> Vec<u8> {
        let mut bytes = Vec::new();
        if self.marked.is_empty()
            && let Some(range) = range_utf16
        {
            let replaced = self.replaced_tail_chars(&range);
            bytes.extend(std::iter::repeat_n(DEL, replaced));
        }
        self.marked.clear();
        self.committed = text.to_string();
        bytes.extend_from_slice(text.as_bytes());
        bytes
    }

    // Only a range that ends where the committed text ends can be undone
    // with backspaces.
    fn replaced_tail_chars(&self, range: &Range<usize>) -> usize {
        if range.is_empty() || range.end != utf16_len(&self.committed) {
            return 0;
        }
        let mut offset = 0;
        self.committed
            .chars()
            .filter(|c| {
                let inside = offset >= range.start;
                offset += c.len_utf16();
                inside
            })
            .count()
    }
}

fn utf16_len(text: &str) -> usize {
    text.encode_utf16().count()
}
