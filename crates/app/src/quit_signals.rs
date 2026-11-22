//! SIGTERM, SIGINT and SIGHUP end the app the way cmd+q does: with its
//! sessions closed first. Left alone, any of them would kill the app on the
//! spot and leave behind every program that ignores the hangup.

use gpui::{App, Entity};
use signal_hook::consts::{SIGHUP, SIGINT, SIGTERM};
use signal_hook::iterator::Signals;

use crate::hosted_sessions::HostedSessions;

pub fn quit_on_signals(sessions: &Entity<HostedSessions>, cx: &mut App) {
    let Ok(mut signals) = Signals::new([SIGTERM, SIGINT, SIGHUP]) else {
        eprintln!("lithemux-app: signals will not close the sessions in order");
        return;
    };
    let (asked, asks) = async_channel::bounded::<()>(1);
    std::thread::spawn(move || {
        for _ in signals.forever() {
            // Full means the app is already on its way out.
            let _ = asked.try_send(());
        }
    });
    let sessions = sessions.downgrade();
    cx.spawn(async move |cx| {
        if asks.recv().await.is_ok() {
            // Closed here and not left to the quit handlers: without a
            // window the app has no delegate and they never run.
            let _ = sessions.update(cx, |sessions, _| sessions.host.shut_down());
            cx.update(|cx| cx.quit());
        }
    })
    .detach();
}
