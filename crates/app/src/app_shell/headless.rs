//! The session host without any interface: `--no-window`. It exists for
//! the tests of how the app exits, which cannot click on a sidebar.

use gpui::{App, AppContext, Context, Entity};
use lithemux_core::modules::session::SessionRef;

use super::quit_signals::quit_on_signals;
use crate::session_host::SessionHost;
use crate::terminal_view::GridSize;

struct Headless {
    host: SessionHost,
}

impl Headless {
    fn new(
        mut host: SessionHost,
        changes: async_channel::Receiver<()>,
        targets: &[String],
        cx: &mut Context<Self>,
    ) -> Self {
        for target in targets {
            let opened = SessionRef::parse(target)
                .map_err(|error| error.to_string())
                .and_then(|session| {
                    host.open(&session, GridSize::INITIAL)
                        .map_err(|error| error.to_string())
                });
            if let Err(problem) = opened {
                eprintln!("lithemux-app: {target}: {problem}");
            }
        }
        cx.spawn(async move |headless, cx| {
            while changes.recv().await.is_ok() {
                let attended = headless.update(cx, |headless, _| {
                    headless.host.process_pending();
                    // Nothing here notifies: the news of the hooks go unread.
                    headless.host.take_news();
                    for warning in headless.host.take_warnings() {
                        eprintln!("lithemux-app: {warning}");
                    }
                });
                if attended.is_err() {
                    return;
                }
            }
        })
        .detach();
        Headless { host }
    }
}

pub fn start(
    host: SessionHost,
    changes: async_channel::Receiver<()>,
    targets: &[String],
    cx: &mut App,
) {
    let headless: Entity<Headless> = cx.new(|cx| Headless::new(host, changes, targets, cx));
    let weak = headless.downgrade();
    quit_on_signals(cx, move |cx| {
        let _ = weak.update(cx, |headless, _| headless.host.shut_down());
    });
    // Nothing else holds it: the app keeps the sessions for its life.
    std::mem::forget(headless);
}
