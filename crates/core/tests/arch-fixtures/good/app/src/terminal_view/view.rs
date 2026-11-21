use super::keys::key_bytes;

pub struct TerminalView;

impl TerminalView {
    pub fn enter(&self) -> Option<Vec<u8>> {
        key_bytes("enter")
    }
}
