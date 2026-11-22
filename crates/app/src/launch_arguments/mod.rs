//! What the app was asked to do on its command line.

mod open_request;

#[cfg(test)]
mod test;

pub use open_request::{NO_WINDOW_FLAG, OPEN_FLAG, OpenRequest};
