use lithemux_app::session_host::{HostConfig, SessionHost};
use lithemux_app::terminal_view::GridSize;
use lithemux_core::modules::session::SessionRef;

#[test]
fn the_host_opens_a_session_of_the_real_home_with_the_real_shell() {
    let (mut host, _changes) = SessionHost::start(HostConfig::from_environment().unwrap()).unwrap();
    let target = SessionRef::parse("demo/ejecutora").unwrap();
    assert!(host.open(&target, GridSize::INITIAL).is_ok());
}
