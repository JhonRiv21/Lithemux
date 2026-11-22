use portable_pty::CommandBuilder;

pub fn shell(argv: Vec<std::ffi::OsString>) -> CommandBuilder {
    CommandBuilder::from_argv(argv)
}
