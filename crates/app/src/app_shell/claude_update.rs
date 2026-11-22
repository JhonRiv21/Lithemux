//! PLAN §3.2: the version of Claude Code. Sessions keep the version they
//! started with; when the `claude` a launch would use now is newer, the
//! status bar says so, and the idle ones can be resumed on it at once.

use std::time::{Duration, Instant};

use gpui::{AppContext, Context, Task};
use lithemux_core::modules::launch::Update;

use super::live::Live;

/// Not at the very start: the sessions on show are being resumed then, and
/// they ask the shell for `claude` too.
const FIRST_CHECK_AFTER: Duration = Duration::from_secs(15);
/// How often it looks at whether the hours of the setting have passed.
const LOOK_EVERY: Duration = Duration::from_secs(10 * 60);
const SECONDS_PER_HOUR: u64 = 3600;

impl Live {
    /// Asks for the installed version on start and then every
    /// `version_check_hours`, always away from the interface thread.
    pub(super) fn keep_checking_the_version(cx: &mut Context<Self>) -> Task<()> {
        cx.spawn(async move |live, cx| {
            cx.background_executor().timer(FIRST_CHECK_AFTER).await;
            loop {
                let Ok(ask) = live.update(cx, |live, _| live.host.version_asker()) else {
                    return;
                };
                let installed = cx.background_spawn(async move { ask() }).await;
                let noted = live.update(cx, |live, cx| {
                    live.installed_claude = installed;
                    live.check_the_update();
                    cx.notify();
                });
                if noted.is_err() {
                    return;
                }
                let asked_at = Instant::now();
                loop {
                    cx.background_executor().timer(LOOK_EVERY).await;
                    let Ok(hours) =
                        live.update(cx, |live, _| live.preferences().version_check_hours)
                    else {
                        return;
                    };
                    let every = Duration::from_secs(hours.unsigned_abs() * SECONDS_PER_HOUR);
                    if asked_at.elapsed() >= every {
                        break;
                    }
                }
            }
        })
    }

    /// Whether some session here runs an older Claude Code than the one a
    /// launch would use now, and which of them can be resumed on it. Kept
    /// up to date by the refresh cycle, not read on every paint.
    pub fn claude_update(&self) -> &Update {
        &self.update
    }

    /// Reads it again; returns whether it changed.
    pub(super) fn check_the_update(&mut self) -> bool {
        let update = self
            .installed_claude
            .as_deref()
            .map(|installed| self.host.claude_update(installed))
            .unwrap_or_default();
        let changed = update != self.update;
        self.update = update;
        changed
    }

    /// Resumes the idle sessions on the new version: each is hibernated and
    /// opened again, with its conversation. The others are left alone.
    pub fn resume_idle_on_the_new_version(&mut self, cx: &mut Context<Self>) {
        if let Some(installed) = self.installed_claude.clone() {
            self.host.resume_idle_on(&installed);
        }
        self.check_the_update();
        self.reload(cx);
        cx.notify();
    }
}
