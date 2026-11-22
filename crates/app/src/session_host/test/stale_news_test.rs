use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use lithemux_core::modules::category::{
    FsBoardStore, NewCategory, Slug, SqliteCategoryRepository, create_category,
};
use lithemux_core::modules::session::{
    Kind, NewSession, Profile, SessionRef, SqliteSessionRepository, create_session,
};
use lithemux_core::modules::settings::{SettingKey, SqliteSettingsRepository, write_setting};
use lithemux_core::platform::paths::home::Home;
use lithemux_core::platform::store::connection::Db;

use crate::session_host::{HostConfig, Opened, Phase, SessionHost};
use crate::terminal_view::GridSize;

fn core_fake(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../core/tests/fakes")
        .join(name)
}

/// A host with one session of the fake claude, which stays up for a while.
fn host_with_a_running_session(dir: &Path) -> (SessionHost, String) {
    let home = Home::new(dir.join("home"));
    fs::create_dir_all(dir.join("work")).unwrap();
    let db = Db::open(&home).unwrap();
    let categories = SqliteCategoryRepository::new(&db);
    let new_category = NewCategory {
        slug: Slug::parse("demo").unwrap(),
        name: "Demo".to_string(),
        root_dir: dir.join("work").display().to_string(),
        color: None,
    };
    create_category(&categories, &FsBoardStore::new(&home), new_category, 0).unwrap();
    let target = SessionRef::parse("demo/ejecutora").unwrap();
    let new_session = NewSession {
        target: target.clone(),
        role: "role".to_string(),
        kind: Kind::Child,
        profile: Profile::Claude,
        cwd: None,
        model: None,
    };
    create_session(
        &categories,
        &SqliteSessionRepository::new(&db),
        new_session,
        0,
    )
    .unwrap();
    let fake_claude = core_fake("fake-claude").display().to_string();
    write_setting(
        &SqliteSettingsRepository::new(&db),
        SettingKey::ClaudePath,
        &fake_claude,
    )
    .unwrap();
    fs::write(home.path().join("fake-claude.sleep"), "30").unwrap();

    let config = HostConfig {
        lithemux_home: Some(home.path().to_path_buf()),
        home,
        hook_binary: core_fake("fake-hook"),
        shell: core_fake("fake-shell"),
        claude_config_dir: Some(dir.join("claude-config")),
        environment: vec![("PATH".into(), "/usr/bin:/bin".into())],
    };
    let (mut host, _changes) = SessionHost::start(config).unwrap();
    let Opened::Opening { session_id } = host.open(&target, GridSize::INITIAL).unwrap() else {
        panic!("the session was not asked for");
    };
    let deadline = Instant::now() + Duration::from_secs(10);
    while host.phase(&session_id) != Some(&Phase::Running) {
        assert!(Instant::now() < deadline, "the session never started");
        host.process_pending();
    }
    (host, session_id)
}

// Audit of the §2.1 (M14): news that an earlier run of the session is gone,
// arriving late, does not end the run that is on screen
#[test]
fn late_news_of_an_earlier_run_does_not_end_the_current_one() {
    let dir = tempfile::tempdir().unwrap();
    let (mut host, session_id) = host_with_a_running_session(&dir.path().canonicalize().unwrap());
    let current = host.hosted[0].run;

    host.program_ended(&session_id, current - 1);

    assert!(host.is_running(&session_id));

    // And the news of this very run does end it.
    host.program_ended(&session_id, current);
    assert_eq!(host.phase(&session_id), Some(&Phase::Ended));
}
