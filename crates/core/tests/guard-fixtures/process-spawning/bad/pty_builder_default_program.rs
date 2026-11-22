pub fn shell() -> portable_pty::CommandBuilder {
    portable_pty::CommandBuilder::new_default_prog()
}
