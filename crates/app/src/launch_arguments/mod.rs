//! What the app was asked to do on its command line.

mod launch_request;

#[cfg(test)]
mod test;

pub use launch_request::{LaunchRequest, NO_WINDOW_FLAG, OPEN_FLAG};
