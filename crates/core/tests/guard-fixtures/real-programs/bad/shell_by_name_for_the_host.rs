use lithemux_app::session_host::HostConfig;

fn config(base: HostConfig) -> HostConfig {
    HostConfig {
        shell: "zsh".into(),
        ..base
    }
}
