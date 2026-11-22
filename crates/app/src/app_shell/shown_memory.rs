//! The memory shown for each session. macOS compresses and decompresses a
//! program's pages all the time, so its resident memory jumps (95 MB, then
//! 175 MB, then 95 MB within seconds). What is shown is the highest figure
//! of the last minute: it moves only with a new peak or when the old one
//! is a minute old, so the window is painted again only then.

use std::collections::{HashMap, VecDeque};
use std::time::{Duration, Instant};

pub const WINDOW: Duration = Duration::from_secs(60);
const MEGABYTE: u64 = 1024 * 1024;

#[derive(Default)]
pub struct ShownMemory {
    samples: HashMap<String, VecDeque<(Instant, u64)>>,
}

impl ShownMemory {
    /// Takes one measure and returns, for each session in it, the highest
    /// of its last minute, rounded up to the megabyte. A session that was
    /// not measured this time is forgotten.
    pub fn record(
        &mut self,
        measured: &HashMap<String, u64>,
        now: Instant,
    ) -> HashMap<String, u64> {
        self.samples.retain(|id, _| measured.contains_key(id));
        measured
            .iter()
            .map(|(id, bytes)| {
                let samples = self.samples.entry(id.clone()).or_default();
                samples.push_back((now, *bytes));
                while samples
                    .front()
                    .is_some_and(|(at, _)| now.duration_since(*at) >= WINDOW)
                {
                    samples.pop_front();
                }
                let peak = samples
                    .iter()
                    .map(|(_, bytes)| *bytes)
                    .max()
                    .unwrap_or(*bytes);
                (id.clone(), peak.div_ceil(MEGABYTE) * MEGABYTE)
            })
            .collect()
    }
}
