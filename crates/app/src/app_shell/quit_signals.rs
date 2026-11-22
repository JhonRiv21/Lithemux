//! SIGTERM, SIGINT and SIGHUP end the app the way cmd+q does: with its
//! sessions closed first. Left alone, any of them would kill the app on the
//! spot and leave behind every program that ignores the hangup.

use gpui::App;
use signal_hook::consts::{SIGHUP, SIGINT, SIGTERM};
use signal_hook::iterator::Signals;

/// `shut_down` closes the sessions, and waits for them, before the app
/// quits.
pub fn quit_on_signals(cx: &mut App, shut_down: impl FnOnce(&mut App) + 'static) {
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
    cx.spawn(async move |cx| {
        if asks.recv().await.is_ok() {
            // Closed here and not left to the quit handlers: without a
            // window the app has no delegate and they never run.
            cx.update(|cx| {
                shut_down(cx);
                cx.quit();
            });
        }
    })
    .detach();
}
