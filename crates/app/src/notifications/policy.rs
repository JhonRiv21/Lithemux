//! Whether a finished turn rings (PLAN §3.1, as Jhon decided on 22-nov:
//! one sound when a task is done, and nothing else). No gpui, no processes.

use std::time::{Duration, Instant};

use lithemux_core::modules::category::Category;
use lithemux_core::modules::hook_event::HookEvent;
use lithemux_core::modules::session::State;

use crate::session_host::HookNews;

/// At most one sound in this while, however many sessions end together.
pub const QUIET_AFTER_A_SOUND: Duration = Duration::from_secs(2);

/// A session, as it stands right after a hook was heard.
#[derive(Debug, Clone, Copy)]
pub struct Situation {
    /// The state the hook's event moved it into; `None` if it stayed.
    pub entered: Option<State>,
    pub is_in_view: bool,
    /// The category's switch, «Sound when done».
    pub category_rings: bool,
    /// Its program runs in this app. One opened in a terminal of its own
    /// is that terminal's business.
    pub runs_here: bool,
}

/// Only a turn that ends (`Stop` that leaves the session idle), out of
/// view, in a category that wants it, of a session that runs here. Waiting
/// for the user does not ring: the count on the card says it.
pub fn rings(event: Option<HookEvent>, session: &Situation) -> bool {
    event == Some(HookEvent::Stop)
        && session.entered == Some(State::Idle)
        && session.runs_here
        && session.category_rings
        && !session.is_in_view
}

/// Where the session of a piece of news stands for the user.
#[derive(Debug, Clone, Copy)]
pub struct Seen<'a> {
    pub category: Option<&'a Category>,
    pub is_in_view: bool,
    pub runs_here: bool,
}

/// Whether a hook's news rings.
pub fn news_rings(news: &HookNews, seen: &Seen<'_>) -> bool {
    let situation = Situation {
        entered: news.entered,
        is_in_view: seen.is_in_view,
        category_rings: seen.category.is_some_and(|category| category.notify),
        runs_here: seen.runs_here,
    };
    rings(news.event, &situation)
}

/// Keeps the sounds apart.
#[derive(Debug, Default)]
pub struct SoundGate {
    last: Option<Instant>,
}

impl SoundGate {
    /// Whether a sound may be played now; if so, it counts as played.
    pub fn may_ring(&mut self, now: Instant) -> bool {
        let quiet = self
            .last
            .is_none_or(|last| now.duration_since(last) >= QUIET_AFTER_A_SOUND);
        if quiet {
            self.last = Some(now);
        }
        quiet
    }
}
