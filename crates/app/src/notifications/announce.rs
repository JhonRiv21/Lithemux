//! Turns what the hooks said into the sound of a finished turn.

use std::path::Path;
use std::thread;
use std::time::Instant;

use gpui::Context;
use lithemux_core::platform::process::done_sound::{DONE_SOUND, play_sound};

use super::policy::{Seen, news_rings};
use crate::app_shell::live::Live;
use crate::session_host::Phase;

impl Live {
    /// The sessions the user is looking at: each tab on show, while the
    /// window is active. The workspace keeps this up to date.
    pub fn set_in_view(&mut self, sessions: Vec<String>) {
        self.in_view = sessions;
    }

    /// Reads what the hooks said and rings once if some turn deserves it.
    /// Runs after the board was read again, so each session is as the hook
    /// left it.
    pub(crate) fn announce_the_news(&mut self, _cx: &mut Context<Self>) {
        let news = self.host.take_news();
        let any_rings = news.iter().any(|news| {
            let Some(session) = self.session(&news.session_id) else {
                return false;
            };
            let seen = Seen {
                category: self
                    .categories()
                    .iter()
                    .find(|category| category.id == session.category_id),
                is_in_view: self.in_view.contains(&news.session_id),
                runs_here: self.host.phase(&news.session_id) == Some(&Phase::Running),
            };
            news_rings(news, &seen)
        });
        if any_rings && self.sound_gate.may_ring(Instant::now()) {
            // `afplay` takes about a second; never on the interface thread.
            thread::spawn(|| play_sound(Path::new(DONE_SOUND)));
        }
    }
}
