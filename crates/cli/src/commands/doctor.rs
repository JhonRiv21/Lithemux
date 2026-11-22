use lithemux_core::modules::category::list_categories;
use lithemux_core::modules::launch::{
    HookDryRun, LoginShellProbe, ProcessHookProbe, SessionLocks, SiblingHookBinary, inspect_claude,
    inspect_hook,
};
use lithemux_core::modules::session::list_sessions;
use lithemux_core::platform::paths::home::Home;
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
    match home.check_writable() {
        Ok(()) => Check::new(
            "data directory",
            Level::Ok,
            format!("{} (writable)", home.path().display()),
        ),
        Err(error) => Check::new("data directory", Level::Fail, error.to_string()),
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
    if let Some(remembered) = &inspection.remembered {
        checks.push(Check::new(
            "claude remembered",
            Level::Ok,
            format!(
                "{}  (used if the shell does not answer; `lithemux config unset last_claude_path` forgets it)",
                remembered.display()
            ),
        ));
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
    let probe = LoginShellProbe::new(user_shell());
    let inspection = inspect_hook(&SiblingHookBinary, &probe, &ProcessHookProbe);
    let mut checks = vec![match &inspection.path {
        Ok(path) => Check::new("hook binary", Level::Ok, path.display().to_string()),
        Err(error) => Check::new("hook binary", Level::Fail, error.to_string()),
    }];
    checks.extend(inspection.dry_run.as_ref().map(hook_dry_run));
    checks
}

// Outside a lithemux session the hook has to be a no-op: exit 0, no output.
fn hook_dry_run(run: &HookDryRun) -> Check {
    let elapsed = run.elapsed.as_secs_f64() * 1000.0;
    let detail = match &run.outcome {
        Ok(_) if run.was_silent_success() => format!("exit 0, no output, {elapsed:.1} ms"),
        Ok(output) => format!(
            "{}, {} bytes on stdout, {} bytes on stderr",
            output.status, output.stdout_bytes, output.stderr_bytes
        ),
        Err(error) => error.clone(),
    };
    let level = if run.was_silent_success() {
        Level::Ok
    } else {
        Level::Fail
    };
    Check::new("hook dry run", level, detail)
}
