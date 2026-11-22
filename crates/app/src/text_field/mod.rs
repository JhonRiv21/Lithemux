//! A one-line text field. `buffer` is the text and its cursor; `view` is
//! the gpui side.

mod buffer;
mod view;

#[cfg(test)]
mod test;

pub use view::{TextField, TextFieldEvent};
