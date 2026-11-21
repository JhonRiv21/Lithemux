//! Claude Code hook. It must never get in Claude's way: it always exits 0,
//! never writes to stderr and does nothing outside a lithemux session.

use std::io::{IsTerminal, Read, Write};

use lithemux_core::modules::category::{FsBoardStore, SqliteCategoryRepository};
use lithemux_core::modules::context::ContextPorts;
use lithemux_core::modules::hook_event::{
    FileHookLog, HookLog, HookPorts, SocketAppNotifier, handle_hook_event,
};
use lithemux_core::modules::session::SqliteSessionRepository;
use lithemux_core::modules::settings::SqliteSettingsRepository;
use lithemux_core::platform::clock::system_clock::now_unix;
use lithemux_core::platform::paths::home::{Home, SESSION_ENV};
use lithemux_core::platform::store::connection::Db;

fn main() {
    std::panic::set_hook(Box::new(|_| {}));
    let _ = std::panic::catch_unwind(run);
    std::process::exit(0);
}

fn run() {
    let Some(session_id) = std::env::var(SESSION_ENV).ok().filter(|id| !id.is_empty()) else {
        return;
    };
    let Some(subcommand) = std::env::args().nth(1) else {
        return;
    };
    let Ok(home) = Home::resolve() else {
        return;
    };
    let log = FileHookLog::new(&home);
    let db = match Db::open_existing(&home) {
        Ok(Some(db)) => db,
        Ok(None) => {
            log.append(&format!(
                "{subcommand} {session_id}: the database does not exist"
            ));
            return;
        }
        Err(error) => {
            log.append(&format!(
                "{subcommand} {session_id}: opening the database: {error}"
            ));
            return;
        }
    };
    let mut stdin = String::new();
    if !std::io::stdin().is_terminal() {
        let _ = std::io::stdin().read_to_string(&mut stdin);
    }
    let ports = HookPorts {
        context: ContextPorts {
            categories: &SqliteCategoryRepository::new(&db),
            sessions: &SqliteSessionRepository::new(&db),
            settings: &SqliteSettingsRepository::new(&db),
            boards: &FsBoardStore::new(&home),
        },
        log: &log,
        notifier: &SocketAppNotifier::new(&home),
    };
    if let Some(output) = handle_hook_event(&ports, &session_id, &subcommand, &stdin, now_unix()) {
        let _ = writeln!(std::io::stdout(), "{output}");
    }
}
