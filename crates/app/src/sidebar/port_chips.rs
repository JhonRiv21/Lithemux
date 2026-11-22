//! Which ports a card names. A dev server tree can listen on a dozen; the
//! card has room for a few.

pub const MAX_PORTS_SHOWN: usize = 3;

/// The ports to name and how many more there are.
pub fn ports_shown(ports: &[u16]) -> (&[u16], usize) {
    let shown = &ports[..ports.len().min(MAX_PORTS_SHOWN)];
    (shown, ports.len() - shown.len())
}
