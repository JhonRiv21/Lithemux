//! Sleep and waking of the sessions, as the workspace drives them: it is
//! the one that knows what is on show. The decisions are pure functions
//! (`core::modules::session`, `start_policy`); here they get the time, the
//! settings and the memory pressure of the machine.

use std::time::Duration;

use gpui::{AppContext, Context, Task};
use lithemux_core::modules::session::Pressure;
use lithemux_core::platform::clock::system_clock::now_unix;
use lithemux_core::platform::process::memory_pressure::{MemoryPressure, memory_pressure};

use super::start_policy::{relaunches_at_startup, wakes_when_seen};
use super::view::Workspace;

/// How often the sessions are looked at. The hours of hibernation do not
/// need more; the memory valve, which acts once a minute at most, does.
const LOOK_EVERY: Duration = Duration::from_secs(30);

impl Workspace {
    fn sessions_on_show(&self) -> Vec<String> {
        let on_show = self.layout.visible_sessions();
        on_show.into_iter().map(str::to_string).collect()
    }

    /// Runs for as long as the workspace lives.
    pub(super) fn keep_the_lifecycle(cx: &mut Context<Self>) -> Task<()> {
        cx.spawn(async move |this, cx| {
            loop {
                cx.background_executor().timer(LOOK_EVERY).await;
                let Ok(valve_is_on) = this.update(cx, |this, cx| {
                    this.sleep_what_is_due(cx);
                    this.live.read(cx).preferences().memory_valve
                }) else {
                    return;
                };
                if !valve_is_on {
                    continue;
                }
                // Asked away from the interface thread: it runs `sysctl`.
                let level = cx.background_spawn(async { memory_pressure() }).await;
                let pressure = match level {
                    Some(MemoryPressure::Critical) => Pressure::Critical,
                    Some(MemoryPressure::Warning) => Pressure::Warning,
                    Some(MemoryPressure::Normal) | None => continue,
                };
                let _ = this.update(cx, |this, cx| this.relieve_memory(pressure, cx));
            }
        })
    }

    /// PLAN §2.4: at rest, out of sight, not kept alive, quiet for too long.
    fn sleep_what_is_due(&mut self, cx: &mut Context<Self>) {
        let on_show = self.sessions_on_show();
        self.live.update(cx, |live, cx| {
            let hours = live.preferences().hibernate_after_hours;
            let slept = live.host.hibernate_what_is_due(&on_show, now_unix(), hours);
            if !slept.is_empty() {
                live.reload(cx);
                cx.notify();
            }
        });
    }

    /// PLAN §2.4, the memory valve: one session, the quietest, a minute.
    fn relieve_memory(&mut self, pressure: Pressure, cx: &mut Context<Self>) {
        let on_show = self.sessions_on_show();
        self.live.update(cx, |live, cx| {
            let taken = live.host.relieve_memory(&on_show, now_unix(), pressure);
            if taken.is_some() {
                live.reload(cx);
                cx.notify();
            }
        });
    }

    /// A hibernated session whose tab came on show is resumed. One that is
    /// still leaving (within the grace of its hibernation) is asked for
    /// once: the host opens it when its last run is gone.
    pub(super) fn wake_what_is_seen(&mut self, cx: &mut Context<Self>) {
        let on_show = self.sessions_on_show();
        let asleep: Vec<String> = {
            let live = self.live.read(cx);
            let wakes = |id: &String| {
                let Some(session) = live.session(id) else {
                    return false;
                };
                let phase = live.host.phase(id);
                wakes_when_seen(phase, session.state, session.profile)
                    && !live.host.reopens_when_closed(id)
            };
            on_show.into_iter().filter(wakes).collect()
        };
        if asleep.is_empty() {
            return;
        }
        self.live.update(cx, |live, cx| {
            for id in &asleep {
                live.open(id, cx);
            }
        });
    }

    /// PLAN §2.5: when the app starts, the shell sessions that have a tab
    /// are started again in their directory.
    pub(super) fn relaunch_the_shells(&mut self, cx: &mut Context<Self>) {
        let panes = self.layout.panes_in_order();
        let tabs: Vec<String> = panes.iter().flat_map(|pane| pane.tabs.clone()).collect();
        self.live.update(cx, |live, cx| {
            let shells: Vec<String> = tabs
                .into_iter()
                .filter(|id| {
                    live.session(id).is_some_and(|session| {
                        relaunches_at_startup(live.host.phase(id), session.profile)
                    })
                })
                .collect();
            for id in &shells {
                live.open(id, cx);
            }
        });
    }
}
