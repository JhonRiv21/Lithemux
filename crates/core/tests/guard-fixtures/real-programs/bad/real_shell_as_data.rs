#[test]
fn a_shell_session_is_a_login_shell() {
    let plan = shell_plan(&category(), &session(), Path::new("/bin/zsh"), None);
    assert_eq!(plan.args, vec!["-l".to_string()]);
}
