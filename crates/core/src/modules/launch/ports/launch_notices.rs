/// Warnings the user should see before the session takes over the terminal.
pub trait LaunchNotices {
    fn warn(&self, message: &str);
}
