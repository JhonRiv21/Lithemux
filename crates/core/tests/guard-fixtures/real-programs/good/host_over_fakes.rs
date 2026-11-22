use lithemux_app::session_host::{HostConfig, SessionHost};

fn fake(name: &str) -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fakes")
        .join(name)
}

fn start(base: HostConfig) -> SessionHost {
    let config = HostConfig {
        shell: fake("fake-shell"),
        hook_binary: fake("fake-hook"),
        environment: vec![("PATH".into(), "/usr/bin:/bin".into())],
        ..base
    };
    SessionHost::start(config).unwrap().0
}
