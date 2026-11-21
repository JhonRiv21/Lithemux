/// Where the hook leaves its problems. It can never fail or print: the hook
/// must stay invisible to Claude.
pub trait HookLog {
    fn append(&self, message: &str);
}
