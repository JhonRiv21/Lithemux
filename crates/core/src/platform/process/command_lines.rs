//! The command lines of the processes that run, read from the kernel's
//! tables, to tell who runs what. Only those with one of the asked-for
//! arguments are kept: the rest is nobody's business here.

use sysinfo::{ProcessRefreshKind, ProcessesToUpdate, System, UpdateKind};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandLine {
    pub pid: u32,
    pub args: Vec<String>,
}

/// `arg` is one of `flags`, alone or as `flag=value`.
fn is_one_of(arg: &str, flags: &[&str]) -> bool {
    flags.iter().any(|flag| {
        arg == *flag
            || arg
                .strip_prefix(flag)
                .is_some_and(|rest| rest.starts_with('='))
    })
}

/// What the kernel's tables need to give the command lines too.
pub(super) fn with_command_lines(kind: ProcessRefreshKind) -> ProcessRefreshKind {
    kind.with_cmd(UpdateKind::OnlyIfNotSet)
}

/// The processes of `system` that have one of `flags` in their command
/// line.
pub(super) fn kept_from(system: &System, flags: &[&str]) -> Vec<CommandLine> {
    system
        .processes()
        .iter()
        .filter_map(|(pid, process)| {
            let args: Vec<String> = process
                .cmd()
                .iter()
                .map(|arg| arg.to_string_lossy().into_owned())
                .collect();
            args.iter()
                .any(|arg| is_one_of(arg, flags))
                .then_some(CommandLine {
                    pid: pid.as_u32(),
                    args,
                })
        })
        .collect()
}

/// Every process that runs now with one of `flags`. Lists all of them, so
/// it costs some milliseconds: for a moment like starting a session, not
/// for every refresh.
pub fn command_lines_with(flags: &[&str]) -> Vec<CommandLine> {
    let Ok(mut system) = System::new() else {
        return Vec::new();
    };
    system.refresh_processes_specifics(
        ProcessesToUpdate::All,
        true,
        with_command_lines(ProcessRefreshKind::nothing()),
    );
    kept_from(&system, flags)
}
