use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::process::{Command, Stdio};
use std::time::Instant;

use lithemux_core::modules::category::list_categories;
use lithemux_core::modules::launch::{
    HookBinary, LoginShellProbe, SessionLocks, SiblingHookBinary, inspect_claude,
};
use lithemux_core::modules::session::list_sessions;
use lithemux_core::platform::paths::home::{Home, SESSION_ENV};
use lithemux_core::platform::process::shell::user_shell;

use crate::output::doctor::{Check, Level};
use crate::wiring::Wiring;

pub fn run() -> anyhow::Result<u8> {
    let mut checks = vec![data_dir(&Home::resolve()?)];
    match Wiring::open() {
        Ok(wiring) => {
            checks.push(database(&wiring));
            checks.extend(claude(&wiring));
            checks.push(open_sessions(&wiring));
        }
        Err(error) => checks.push(Check::new("database", Level::Fail, format!("{error:#}"))),
    }
    checks.extend(hook());
    crate::output::doctor::report(&checks);
    let failed = checks.iter().any(|check| check.level == Level::Fail);
    Ok(u8::from(failed))
}

fn data_dir(home: &Home) -> Check {
    let probe = home.path().join(".doctor-probe");
    let writable = std::fs::create_dir_all(home.path())
        .and_then(|()| std::fs::write(&probe, b""))
        .and_then(|()| std::fs::remove_file(&probe));
    match writable {
        Ok(()) => Check::new(
            "data directory",
            Level::Ok,
            format!("{} (writable)", home.path().display()),
        ),
        Err(error) => Check::new(
            "data directory",
            Level::Fail,
            format!("{}: {error}", home.path().display()),
        ),
    }
}

fn database(wiring: &Wiring) -> Check {
    match wiring.db.schema_version() {
        Ok(version) => Check::new(
            "database",
            Level::Ok,
            format!("{} (schema v{version})", wiring.home.db_path().display()),
        ),
        Err(error) => Check::new("database", Level::Fail, error.to_string()),
    }
}

fn claude(wiring: &Wiring) -> Vec<Check> {
    let inspection = inspect_claude(&wiring.settings(), &LoginShellProbe::new(user_shell()));
    let mut checks = vec![match &inspection.chosen {
        Ok(path) => Check::new("claude", Level::Ok, path.display().to_string()),
        Err(error) => Check::new("claude", Level::Fail, error.to_string()),
    }];
    if let Some(warning) = inspection.warning {
        checks.push(Check::new("claude lookup", Level::Warn, warning));
    }
    for install in &inspection.installations {
        let note = match (install.chosen, install.usable) {
            (true, _) => "  <- used by lithemux",
            (false, false) => "  (ignored: wrapper or not executable)",
            (false, true) => "",
        };
        let detail = format!(
            "{}  {}{note}",
            install.version.as_deref().unwrap_or("version unknown"),
            install.path.display()
        );
        checks.push(Check::new("claude install", Level::Ok, detail));
    }
    if inspection.version_mismatch {
        checks.push(Check::new(
            "claude installs",
            Level::Warn,
            "several installs on different versions; set claude_path if the wrong one is used"
                .to_string(),
        ));
    }
    checks
}

fn open_sessions(wiring: &Wiring) -> Check {
    let (sessions, locks) = (wiring.sessions(), wiring.locks());
    let mut open = Vec::new();
    for category in list_categories(&wiring.categories()).unwrap_or_default() {
        for session in list_sessions(&sessions, &category).unwrap_or_default() {
            if let Some(pid) = locks.holder(&session.id) {
                open.push(format!("{} (pid {pid})", session.messaging_name(&category)));
            }
        }
    }
    let detail = if open.is_empty() {
        "none".to_string()
    } else {
        open.join(", ")
    };
    Check::new("open sessions", Level::Ok, detail)
}

fn hook() -> Vec<Check> {
    let path = match SiblingHookBinary.path() {
        Ok(path) => path,
        Err(error) => return vec![Check::new("hook binary", Level::Fail, error.to_string())],
    };
    let executable = std::fs::metadata(&path)
        .is_ok_and(|meta| meta.is_file() && meta.permissions().mode() & 0o111 != 0);
    if !executable {
        return vec![Check::new(
            "hook binary",
            Level::Fail,
            format!("{} is missing or not executable", path.display()),
        )];
    }
    vec![
        Check::new("hook binary", Level::Ok, path.display().to_string()),
        hook_dry_run(&path),
    ]
}

// Outside a lithemux session the hook has to be a no-op: exit 0, no output.
fn hook_dry_run(path: &Path) -> Check {
    let started = Instant::now();
    let output = Command::new(path)
        .arg("stop")
        .env_remove(SESSION_ENV)
        .stdin(Stdio::null())
        .output();
    let elapsed = started.elapsed().as_secs_f64() * 1000.0;
    match output {
        Ok(out) if out.status.success() && out.stdout.is_empty() && out.stderr.is_empty() => {
            Check::new(
                "hook dry run",
                Level::Ok,
                format!("exit 0, no output, {elapsed:.1} ms"),
            )
        }
        Ok(out) => Check::new(
            "hook dry run",
            Level::Fail,
            format!(
                "{}, {} bytes on stdout, {} bytes on stderr",
                out.status,
                out.stdout.len(),
                out.stderr.len()
            ),
        ),
        Err(error) => Check::new("hook dry run", Level::Fail, error.to_string()),
    }
}
