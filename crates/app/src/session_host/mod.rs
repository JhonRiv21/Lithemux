//! The sessions the app has open, without any interface: `host` owns their
//! terminals and their state, `opening` starts them and `closing` ends them; `event_socket`
//! hears the hooks; `instance_lock` keeps the app to one; `mailbox` carries
//! what the background threads see; `leaving` keeps what is still on its
//! way out of the sessions that were closed.

mod closing;
mod config;
mod error;
mod event_socket;
mod failure;
mod host;
mod hosted;
mod instance_lock;
mod leaving;
mod lifecycle;
mod mailbox;
mod opened;
mod opening;
mod phase;
mod queries;

#[cfg(test)]
mod test;

pub use config::{HIBERNATE_GRACE, HostConfig};
pub use error::HostError;
pub use failure::Failure;
pub use host::SessionHost;
pub use opened::Opened;
pub use phase::Phase;
