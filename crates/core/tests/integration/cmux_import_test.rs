//! PLAN §3.4: importing cmux's sessions, from synthetic files of cmux's
//! format (never the user's real one) into a real SQLite database.

use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;

use lithemux_core::modules::category::{
    BoardStore, Category, FsBoardStore, NewCategory, Slug, SqliteCategoryRepository,
    create_category,
};
use lithemux_core::modules::import::{
    Choices, CmuxSessionFile, CmuxSnapshot, Destination, ImportError, ImportPorts, ImportStore,
    SqliteImportStore, apply_import, choose, propose_import,
};
use lithemux_core::modules::launch::ClaudeProjectsTranscriptLocator;
use lithemux_core::modules::session::{SessionRepository, SqliteSessionRepository, State};
use lithemux_core::platform::store::connection::Db;

use crate::temp_home::TempHome;

const HOME: &str = "/Users/someone";
const PLANNER: &str = "11111111-1111-4111-8111-111111111111";
const IN_HOME: &str = "22222222-2222-4222-8222-222222222222";
const BUILD: &str = "55555555-5555-4555-8555-555555555555";

fn fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/cmux")
        .join(name)
}

struct World {
    temp: TempHome,
    db: Db,
    cmux: CmuxSessionFile,
    transcripts: ClaudeProjectsTranscriptLocator,
    alpha: Category,
}

impl World {
    /// lithemux with the category «alpha», and Claude Code keeping the
    /// conversations of PLANNER and BUILD but not of IN_HOME.
    fn new(cmux_file: &Path) -> Self {
        let temp = TempHome::new();
        let db = temp.db();
        let config = temp.dir.path().join("claude-config");
        for id in [PLANNER, BUILD] {
            let project = config.join("projects/-somewhere");
            fs::create_dir_all(&project).unwrap();
            fs::write(project.join(format!("{id}.jsonl")), "{}\n").unwrap();
        }
        let alpha = create_category(
            &SqliteCategoryRepository::new(&db),
            &FsBoardStore::new(&temp.home),
            NewCategory {
                slug: Slug::parse("alpha").unwrap(),
                name: "Alpha".to_string(),
                root_dir: format!("{HOME}/work/alpha"),
                color: None,
            },
            1,
        )
        .unwrap();
        World {
            cmux: CmuxSessionFile::at(cmux_file),
            transcripts: ClaudeProjectsTranscriptLocator::in_config_dir(config),
            temp,
            db,
            alpha,
        }
    }

    fn with(&self, store: &dyn ImportStore, run: impl FnOnce(&ImportPorts<'_>)) {
        let ports = ImportPorts {
            cmux: &self.cmux,
            categories: &SqliteCategoryRepository::new(&self.db),
            sessions: &SqliteSessionRepository::new(&self.db),
            transcripts: &self.transcripts,
            boards: &FsBoardStore::new(&self.temp.home),
            store,
        };
        run(&ports);
    }

    fn run(&self, run: impl FnOnce(&ImportPorts<'_>)) {
        self.with(&SqliteImportStore::new(&self.db), run);
    }
}

// PLAN §3.4: la propuesta, por cada panel de Claude
#[test]
fn the_proposal_has_each_claude_panel_and_counts_the_rest() {
    let world = World::new(&fixture("sessions-v1.json"));
    world.run(|ports| {
        let proposal = propose_import(ports, Some(HOME)).unwrap();

        let rows: Vec<_> = proposal
            .sessions
            .iter()
            .map(|s| (s.number, s.name.as_str(), s.has_conversation, s.selected))
            .collect();
        assert_eq!(
            rows,
            [
                (1, "Planner", true, true),
                (2, "Review the API", false, false),
                (5, "Fix the build", true, true),
            ]
        );
        assert_eq!(proposal.not_claude, 1);
        assert_eq!(proposal.without_conversation_id, 1);
        assert_eq!(proposal.sessions[1].claude_session_id, IN_HOME);
        assert_eq!(proposal.sessions[1].destination, Destination::Unassigned);
        assert_eq!(proposal.new_categories.len(), 1);
        assert_eq!(proposal.new_categories[0].slug, "beta");
    });
}

// PLAN §3.4: aplicar crea categorías y sesiones hibernadas con su
// claude_session_id, sin arrancar nada; reimportar no duplica
#[test]
fn applying_makes_hibernated_sessions_and_importing_again_adds_nothing() {
    let world = World::new(&fixture("sessions-v1.json"));
    world.run(|ports| {
        let proposal = propose_import(ports, Some(HOME)).unwrap();
        let chosen = choose(&proposal, &Choices::default()).unwrap();
        let imported = apply_import(ports, &chosen, 50).unwrap();
        assert_eq!((imported.categories, imported.sessions), (1, 2));

        let sessions = SqliteSessionRepository::new(&world.db).list_all().unwrap();
        let planner = sessions.iter().find(|s| s.slug == "planner").unwrap();
        assert_eq!(planner.category_id, world.alpha.id);
        assert_eq!(planner.claude_session_id.as_deref(), Some(PLANNER));
        assert_eq!(planner.state, State::Hibernated);
        assert_eq!(planner.title.as_deref(), Some("Planner"));
        assert_eq!(
            planner.cwd.as_deref(),
            Some("/Users/someone/work/alpha/api")
        );
        assert!(planner.is_pinned());
        let build = sessions.iter().find(|s| s.slug == "fix-the-build").unwrap();
        assert_eq!(build.state, State::Hibernated);
        assert_eq!(build.cwd, None);

        let again = propose_import(ports, Some(HOME)).unwrap();
        let left: Vec<_> = again.sessions.iter().map(|s| s.number).collect();
        assert_eq!(left, [2]);
        assert_eq!(again.already_imported, 2);
    });
}

/// Does all the work of the import inside the real transaction, and then
/// fails before it commits.
struct FailsBeforeCommitting<'a>(&'a Db);

impl ImportStore for FailsBeforeCommitting<'_> {
    fn atomically(
        &self,
        work: &mut dyn FnMut() -> Result<(), ImportError>,
    ) -> Result<(), ImportError> {
        SqliteImportStore::new(self.0).atomically(&mut || {
            work()?;
            Err(ImportError::UnknownNumber(0))
        })
    }
}

// PLAN §3.4: aplicar en una transacción
#[test]
fn an_import_that_fails_halfway_stores_nothing() {
    let world = World::new(&fixture("sessions-v1.json"));
    world.with(&FailsBeforeCommitting(&world.db), |ports| {
        let proposal = propose_import(ports, Some(HOME)).unwrap();
        let chosen = choose(&proposal, &Choices::default()).unwrap();

        assert!(apply_import(ports, &chosen, 50).is_err());
    });

    let categories = SqliteCategoryRepository::new(&world.db);
    let slugs: Vec<_> = lithemux_core::modules::category::list_categories(&categories)
        .unwrap()
        .into_iter()
        .map(|c| c.slug)
        .collect();
    assert_eq!(slugs, ["alpha"]);
    assert!(
        SqliteSessionRepository::new(&world.db)
            .list_all()
            .unwrap()
            .is_empty()
    );
    // Nor a board for a category that was never stored.
    let boards = FsBoardStore::new(&world.temp.home);
    assert!(!boards.path("beta").exists());
    assert!(boards.path("alpha").exists());
}

// ronda 3B bug 5: two imports at once, each on its own connection, add
// each conversation once
#[test]
fn two_imports_at_the_same_time_add_each_conversation_once() {
    let world = World::new(&fixture("sessions-v1.json"));
    let mut chosen = Vec::new();
    world.run(|ports| {
        let proposal = propose_import(ports, Some(HOME)).unwrap();
        chosen = choose(&proposal, &Choices::default()).unwrap();
    });
    let start = std::sync::Barrier::new(2);
    let results: Vec<Result<usize, String>> = std::thread::scope(|scope| {
        let runs: Vec<_> = (0..2)
            .map(|_| {
                scope.spawn(|| {
                    let db = world.temp.db();
                    let ports = ImportPorts {
                        cmux: &world.cmux,
                        categories: &SqliteCategoryRepository::new(&db),
                        sessions: &SqliteSessionRepository::new(&db),
                        transcripts: &world.transcripts,
                        boards: &FsBoardStore::new(&world.temp.home),
                        store: &SqliteImportStore::new(&db),
                    };
                    start.wait();
                    apply_import(&ports, &chosen, 50)
                        .map(|imported| imported.sessions)
                        .map_err(|error| error.to_string())
                })
            })
            .collect();
        runs.into_iter().map(|run| run.join().unwrap()).collect()
    });

    let added: usize = results.iter().filter_map(|r| r.as_ref().ok()).sum();
    assert_eq!(added, 2, "{results:?}");
    let sessions = SqliteSessionRepository::new(&world.db).list_all().unwrap();
    assert_eq!(sessions.len(), 2);
    let categories = SqliteCategoryRepository::new(&world.db);
    let all = lithemux_core::modules::category::list_categories(&categories).unwrap();
    assert_eq!(
        all.len(),
        2,
        "{:?}",
        all.iter().map(|c| &c.slug).collect::<Vec<_>>()
    );
}

// PLAN §3.4: solo lectura sobre cmux
#[test]
fn importing_leaves_the_file_of_cmux_exactly_as_it_was() {
    let dir = tempfile::tempdir().unwrap();
    let copy = dir.path().join("session-com.cmuxterm.app.json");
    fs::copy(fixture("sessions-v1.json"), &copy).unwrap();
    let before = (
        fs::read(&copy).unwrap(),
        fs::metadata(&copy).unwrap().modified().unwrap(),
    );
    let world = World::new(&copy);

    world.run(|ports| {
        let proposal = propose_import(ports, Some(HOME)).unwrap();
        let chosen = choose(&proposal, &Choices::default()).unwrap();
        apply_import(ports, &chosen, 50).unwrap();
    });

    let after = (
        fs::read(&copy).unwrap(),
        fs::metadata(&copy).unwrap().modified().unwrap(),
    );
    assert_eq!(before, after);
    let names: Vec<_> = fs::read_dir(dir.path())
        .unwrap()
        .flatten()
        .map(|e| e.file_name())
        .collect();
    assert_eq!(names.len(), 1, "{names:?}");
}

// PLAN §3.4: si la version no es 1, se rechaza con un mensaje claro
#[test]
fn a_file_of_an_unknown_version_is_refused() {
    let read = CmuxSessionFile::at(fixture("sessions-v2.json")).panels();

    let error = read.unwrap_err();
    assert!(matches!(error, ImportError::UnknownVersion(2)));
    assert_eq!(
        error.to_string(),
        "the sessions file of cmux is version 2, and only version 1 is known"
    );
}

// PLAN §3.4: a medio escribir, se reintenta una vez y, si sigue roto, se dice
#[test]
fn a_file_that_stays_half_written_is_refused_after_one_more_try() {
    let read = CmuxSessionFile::at(fixture("sessions-half-written.json")).panels();

    let error = read.unwrap_err();
    assert!(matches!(error, ImportError::Unreadable { .. }), "{error}");
    assert!(error.to_string().contains("try again"), "{error}");
}

// PLAN §3.4: cmux rewrites the file now and then; the second read finds it
// whole
#[test]
fn a_file_caught_half_written_is_read_again_a_moment_later() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("session-com.cmuxterm.app.json");
    fs::copy(fixture("sessions-half-written.json"), &path).unwrap();
    let writer = {
        let path = path.clone();
        std::thread::spawn(move || {
            std::thread::sleep(Duration::from_millis(40));
            fs::copy(fixture("sessions-v1.json"), &path).unwrap();
        })
    };

    let panels = CmuxSessionFile::at(&path).panels().unwrap();

    writer.join().unwrap();
    assert_eq!(panels.len(), 5);
}

// PLAN §3.4: no cmux file, no import, and it says where it looked
#[test]
fn without_a_file_of_cmux_it_says_where_it_looked() {
    let dir = tempfile::tempdir().unwrap();

    let error = CmuxSessionFile::of_user(dir.path()).panels().unwrap_err();

    assert!(matches!(error, ImportError::NoCmuxFile(_)));
    assert!(
        error
            .to_string()
            .ends_with("Library/Application Support/cmux/session-com.cmuxterm.app.json"),
        "{error}"
    );
}

// ronda 3B bug 3: a proposal made before a category with its folder
// existed does not make that category a second time
#[test]
fn a_proposal_made_before_its_category_existed_uses_that_category() {
    let world = World::new(&fixture("sessions-v1.json"));
    world.run(|ports| {
        let proposal = propose_import(ports, Some(HOME)).unwrap();
        let chosen = choose(&proposal, &Choices::default()).unwrap();
        let beta = create_category(
            &SqliteCategoryRepository::new(&world.db),
            &FsBoardStore::new(&world.temp.home),
            NewCategory {
                slug: Slug::parse("beta-made-meanwhile").unwrap(),
                name: "Beta".to_string(),
                root_dir: format!("{HOME}/work/beta"),
                color: None,
            },
            2,
        )
        .unwrap();

        let imported = apply_import(ports, &chosen, 50).unwrap();

        assert_eq!(imported.categories, 0);
        let sessions = SqliteSessionRepository::new(&world.db).list_all().unwrap();
        let build = sessions.iter().find(|s| s.slug == "fix-the-build").unwrap();
        assert_eq!(build.category_id, beta.id);
    });
}

fn written(dir: &Path, text: &str) -> PathBuf {
    let path = dir.join("session-com.cmuxterm.app.json");
    fs::write(&path, text).unwrap();
    path
}

// ronda 3B punto 8: sin `version` se dice que falta, no «version 0»
#[test]
fn a_file_without_a_version_says_the_version_is_missing() {
    let dir = tempfile::tempdir().unwrap();
    let path = written(dir.path(), r#"{ "windows": [] }"#);

    let error = CmuxSessionFile::at(&path).panels().unwrap_err();

    assert!(matches!(error, ImportError::MissingVersion), "{error}");
    assert_eq!(
        error.to_string(),
        "the sessions file of cmux has no version"
    );
}

// ronda 3B punto 8: a version that is not a number is a format problem,
// not a file caught half written
#[test]
fn a_version_that_is_not_a_number_is_a_format_problem_and_not_retried() {
    let dir = tempfile::tempdir().unwrap();
    let path = written(dir.path(), r#"{ "version": "1", "windows": [] }"#);

    let started = std::time::Instant::now();
    let error = CmuxSessionFile::at(&path).panels().unwrap_err();

    assert!(matches!(error, ImportError::Format { .. }), "{error}");
    assert!(!error.to_string().contains("try again"), "{error}");
    assert!(started.elapsed() < Duration::from_millis(200));
}

// ronda 3B punto 8: `null` where a list goes is an empty list
#[test]
fn a_null_list_is_read_as_empty() {
    let dir = tempfile::tempdir().unwrap();
    let path = written(
        dir.path(),
        r#"{ "version": 1, "windows": [
             { "tabManager": { "workspaces": [ { "panels": null }, { "processTitle": "x" } ] } },
             { "tabManager": { "workspaces": null } },
             { "tabManager": null } ] }"#,
    );

    assert_eq!(CmuxSessionFile::at(&path).panels().unwrap(), Vec::new());
    let none = written(dir.path(), r#"{ "version": 1, "windows": null }"#);
    assert_eq!(CmuxSessionFile::at(&none).panels().unwrap(), Vec::new());
}

// ronda 3B punto 8: a file far bigger than cmux writes is not read
#[test]
fn a_file_too_big_to_be_cmux_s_is_not_read() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("session-com.cmuxterm.app.json");
    let file = fs::File::create(&path).unwrap();
    file.set_len(21 * 1024 * 1024).unwrap();

    let error = CmuxSessionFile::at(&path).panels().unwrap_err();

    assert!(matches!(error, ImportError::TooBig { .. }), "{error}");
    assert!(error.to_string().contains("20 MB"), "{error}");
}
