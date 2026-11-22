//! One line of text being edited, and where the cursor is. No gpui.

use std::ops::Range;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Buffer {
    text: String,
    /// A byte index on a character boundary.
    cursor: usize,
    /// What an input method is composing, as a byte range of `text`.
    marked: Option<Range<usize>>,
}

impl Buffer {
    pub fn with_text(text: &str) -> Self {
        let text: String = text.chars().filter(|c| !c.is_control()).collect();
        Buffer {
            cursor: text.len(),
            text,
            marked: None,
        }
    }

    pub fn text(&self) -> &str {
        &self.text
    }

    /// The text before and after the cursor.
    pub fn around_cursor(&self) -> (&str, &str) {
        self.text.split_at(self.cursor)
    }

    /// Replaces `range` (or what is being composed, or nothing at the
    /// cursor) with `typed`, which stays on one line.
    pub fn replace(&mut self, range: Option<Range<usize>>, typed: &str) {
        let typed: String = typed.chars().filter(|c| !c.is_control()).collect();
        let range = self.resolved(range);
        self.text.replace_range(range.clone(), &typed);
        self.cursor = range.start + typed.len();
        self.marked = None;
    }

    /// Like `replace`, and the new text is what is being composed.
    pub fn compose(&mut self, range: Option<Range<usize>>, composing: &str) {
        let start = self.resolved(range.clone()).start;
        self.replace(range, composing);
        self.marked = (self.cursor > start).then_some(start..self.cursor);
    }

    pub fn stop_composing(&mut self) {
        self.marked = None;
    }

    pub fn delete_before(&mut self) {
        if let Some(previous) = self.text[..self.cursor].chars().next_back() {
            let start = self.cursor - previous.len_utf8();
            self.replace(Some(start..self.cursor), "");
        }
    }

    pub fn delete_after(&mut self) {
        if let Some(next) = self.text[self.cursor..].chars().next() {
            self.replace(Some(self.cursor..self.cursor + next.len_utf8()), "");
        }
    }

    pub fn left(&mut self) {
        if let Some(previous) = self.text[..self.cursor].chars().next_back() {
            self.cursor -= previous.len_utf8();
        }
    }

    pub fn right(&mut self) {
        if let Some(next) = self.text[self.cursor..].chars().next() {
            self.cursor += next.len_utf8();
        }
    }

    pub fn start(&mut self) {
        self.cursor = 0;
    }

    pub fn end(&mut self) {
        self.cursor = self.text.len();
    }

    fn resolved(&self, range: Option<Range<usize>>) -> Range<usize> {
        let range = range
            .or_else(|| self.marked.clone())
            .unwrap_or(self.cursor..self.cursor);
        let end = self.boundary(range.end);
        self.boundary(range.start).min(end)..end
    }

    fn boundary(&self, index: usize) -> usize {
        let mut index = index.min(self.text.len());
        while !self.text.is_char_boundary(index) {
            index -= 1;
        }
        index
    }

    // The system speaks in UTF-16 units.

    pub fn utf16_len(&self, bytes: usize) -> usize {
        self.text[..self.boundary(bytes)].encode_utf16().count()
    }

    pub fn bytes_of_utf16(&self, units: usize) -> usize {
        let mut seen = 0;
        for (index, character) in self.text.char_indices() {
            if seen >= units {
                return index;
            }
            seen += character.len_utf16();
        }
        self.text.len()
    }

    pub fn range_from_utf16(&self, range: &Range<usize>) -> Range<usize> {
        self.bytes_of_utf16(range.start)..self.bytes_of_utf16(range.end)
    }

    pub fn cursor_utf16(&self) -> usize {
        self.utf16_len(self.cursor)
    }

    pub fn marked_utf16(&self) -> Option<Range<usize>> {
        let marked = self.marked.as_ref()?;
        Some(self.utf16_len(marked.start)..self.utf16_len(marked.end))
    }
}
