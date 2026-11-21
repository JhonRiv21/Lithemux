use std::path::PathBuf;

const FALLBACK_SHELL: &str = "/bin/zsh";

pub fn user_shell() -> PathBuf {
    std::env::var_os("SHELL")
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(FALLBACK_SHELL))
}
