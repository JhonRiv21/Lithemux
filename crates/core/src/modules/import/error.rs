use std::path::PathBuf;

use crate::modules::category::CategoryError;
use crate::modules::session::SessionError;
use crate::platform::store::error::StoreError;

#[derive(Debug, thiserror::Error)]
pub enum ImportError {
    #[error("cmux has no saved sessions at {0}")]
    NoCmuxFile(PathBuf),

    #[error("could not read {path}: {detail} (cmux may have been writing it; try again)")]
    Unreadable { path: PathBuf, detail: String },

    #[error("the sessions file of cmux has no version")]
    MissingVersion,

    #[error("{path} is not in the format of cmux's sessions file: {detail}")]
    Format { path: PathBuf, detail: String },

    #[error("{path} has {megabytes} MB, more than the {limit} MB a sessions file of cmux can have")]
    TooBig {
        path: PathBuf,
        megabytes: u64,
        limit: u64,
    },

    #[error("the sessions file of cmux is version {0}, and only version 1 is known")]
    UnknownVersion(i64),

    #[error("the import finished without storing anything")]
    NothingStored,

    #[error("no session numbered {0} in the proposal")]
    UnknownNumber(usize),

    #[error(
        "{} without a category: choose one with --into <category>, or leave {} out with --skip",
        numbers(.0),
        if .0.len() == 1 { "it" } else { "them" }
    )]
    WithoutCategory(Vec<usize>),

    #[error(transparent)]
    Category(#[from] CategoryError),

    #[error(transparent)]
    Session(#[from] SessionError),

    #[error(transparent)]
    Store(#[from] StoreError),
}

fn numbers(list: &[usize]) -> String {
    let joined: Vec<String> = list.iter().map(usize::to_string).collect();
    let noun = if list.len() == 1 {
        "session"
    } else {
        "sessions"
    };
    format!("{noun} {}", joined.join(", "))
}
