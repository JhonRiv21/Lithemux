//! Rust source as a flat list of tokens. The guards read code this way
//! instead of as text: spacing, comments, line breaks and string contents
//! cannot hide or fake a name, and the inside of a macro call is visible too.

use std::str::FromStr;

use proc_macro2::{Delimiter, Spacing, TokenStream, TokenTree};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Kind {
    Ident(String),
    /// A string literal, without its quotes.
    Text(String),
    /// `::`
    PathSeparator,
    Open(Delimiter),
    Close(Delimiter),
    Other,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Token {
    pub kind: Kind,
    pub line: usize,
}

impl Token {
    pub fn ident(&self) -> Option<&str> {
        match &self.kind {
            Kind::Ident(name) => Some(name),
            _ => None,
        }
    }

    pub fn text(&self) -> Option<&str> {
        match &self.kind {
            Kind::Text(text) => Some(text),
            _ => None,
        }
    }
}

/// `None` when the text is not Rust the lexer accepts.
pub fn tokens_of(source: &str) -> Option<Vec<Token>> {
    let stream = TokenStream::from_str(source).ok()?;
    let mut tokens = Vec::new();
    flatten(stream, &mut tokens);
    Some(tokens)
}

fn flatten(stream: TokenStream, out: &mut Vec<Token>) {
    let mut pending_colon = false;
    for tree in stream {
        let line = tree.span().start().line;
        let mut push = |kind| out.push(Token { kind, line });
        let was_pending = std::mem::take(&mut pending_colon);
        match tree {
            // `r#name` is the same name.
            TokenTree::Ident(ident) => {
                push(Kind::Ident(
                    ident.to_string().trim_start_matches("r#").to_string(),
                ));
            }
            TokenTree::Literal(literal) => {
                let written = literal.to_string();
                match string_contents(&written) {
                    Some(text) => push(Kind::Text(text.to_string())),
                    None => push(Kind::Other),
                }
            }
            TokenTree::Punct(punct) if punct.as_char() == ':' => {
                if was_pending {
                    push(Kind::PathSeparator);
                } else if punct.spacing() == Spacing::Joint {
                    pending_colon = true;
                } else {
                    push(Kind::Other);
                }
            }
            TokenTree::Punct(_) => push(Kind::Other),
            TokenTree::Group(group) => {
                let delimiter = group.delimiter();
                let close_line = group.span_close().start().line;
                push(Kind::Open(delimiter));
                flatten(group.stream(), out);
                out.push(Token {
                    kind: Kind::Close(delimiter),
                    line: close_line,
                });
            }
        }
    }
}

/// The inside of `"..."`, `r"..."`, `r#"..."#` and their byte and C forms.
fn string_contents(written: &str) -> Option<&str> {
    let start = written.find('"')?;
    let end = written.rfind('"')?;
    let prefix = &written[..start];
    let is_string = prefix.chars().all(|c| matches!(c, 'r' | 'b' | 'c' | '#'));
    (is_string && end > start).then(|| &written[start + 1..end])
}

/// Every `a::b::c` in the tokens, as its segments and the line it starts on.
/// A lone identifier is a path of one segment.
pub fn paths_in(tokens: &[Token]) -> Vec<(Vec<&str>, usize)> {
    let mut paths = Vec::new();
    let mut index = 0;
    while index < tokens.len() {
        let Some(first) = tokens[index].ident() else {
            index += 1;
            continue;
        };
        let mut segments = vec![first];
        let line = tokens[index].line;
        index += 1;
        while let (Some(separator), Some(next)) = (tokens.get(index), tokens.get(index + 1))
            && separator.kind == Kind::PathSeparator
            && let Some(segment) = next.ident()
        {
            segments.push(segment);
            index += 2;
        }
        paths.push((segments, line));
    }
    paths
}
