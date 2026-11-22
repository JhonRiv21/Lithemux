//! The sessions the app has open, without any interface: `host` owns their
//! terminals and their state, `opening` starts them and `closing` ends them; `event_socket`
//! hears the hooks; `instance_lock` keeps the app to one; `mailbox` carries
//! what the background threads see.

mod closing;
mod config;
mod error;
mod event_socket;
mod host;
mod hosted;
mod instance_lock;
mod mailbox;
mod opening;
mod phase;

#[cfg(test)]
mod test;

pub use config::HostConfig;
pub use error::HostError;
pub use host::{Opened, SessionHost};
pub use phase::Phase;
