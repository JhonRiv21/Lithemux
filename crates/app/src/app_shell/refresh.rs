//! Every few seconds: the board again, for what changed outside the app,
//! and what the sessions that run are doing with ports and memory.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Instant;

use gpui::{AppContext, Context, Task};
use lithemux_core::platform::process::listening_ports::listening_ports;
use lithemux_core::platform::process::memory_use::MemoryMeter;

use super::live::{Live, REFRESH_EVERY};
use super::shown_memory::ShownMemory;

/// The memory measured for each pid, by the session the pid belongs to.
fn memory_by_session(running: &[(String, u32)], used: &HashMap<u32, u64>) -> HashMap<String, u64> {
    running
        .iter()
        .filter_map(|(id, pid)| Some((id.clone(), *used.get(pid)?)))
        .collect()
}

/// The ports found for each pid, by the session the pid belongs to.
fn ports_by_session(
    running: Vec<(String, u32)>,
    found: &HashMap<u32, Vec<u16>>,
) -> HashMap<String, Vec<u16>> {
    running
        .into_iter()
        .filter_map(|(id, pid)| Some((id, found.get(&pid)?.clone())))
        .filter(|(_, ports)| !ports.is_empty())
        .collect()
}

impl Live {
    /// Every while: the board again, for what changed outside the app, and
    /// the ports of what runs, looked for away from the interface thread.
    pub(super) fn keep_refreshing(cx: &mut Context<Self>) -> Task<()> {
        // Kept between cycles: it maps the process trees only now and then.
        let meter = Arc::new(Mutex::new(MemoryMeter::new()));
        let mut shown = ShownMemory::default();
        cx.spawn(async move |live, cx| {
            loop {
                cx.background_executor().timer(REFRESH_EVERY).await;
                let Ok(running) = live.update(cx, |live, cx| {
                    if live.reload(cx) | live.check_the_update() {
                        cx.notify();
                    }
                    live.host.running_programs()
                }) else {
                    return;
                };
                let pids: Vec<u32> = running.iter().map(|(_, pid)| *pid).collect();
                // Both read the kernel's tables; neither on this thread.
                let meter = Arc::clone(&meter);
                let (found, used) = cx
                    .background_spawn(async move {
                        let used = meter
                            .lock()
                            .map(|mut meter| meter.measure(&pids, Instant::now()))
                            .unwrap_or_default();
                        (listening_ports(&pids), used)
                    })
                    .await;
                let memory = shown.record(&memory_by_session(&running, &used), Instant::now());
                let ports = ports_by_session(running, &found);
                let updated = live.update(cx, |live, cx| {
                    if live.ports != ports || live.memory != memory {
                        live.ports = ports;
                        live.memory = memory;
                        cx.notify();
                    }
                });
                if updated.is_err() {
                    return;
                }
            }
        })
    }
}
