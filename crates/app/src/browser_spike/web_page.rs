//! The real page: a WebKit view of the spike crate, opened with what the
//! environment asks for.

use lithemux_webview_spike::page::{FraudWarning, Page, PageError};

use super::browsers::{Browsers, Surface};

/// What the spike opens: `LITHEMUX_SPIKE_URL`, or a blank page.
const URL_VARIABLE: &str = "LITHEMUX_SPIKE_URL";
/// `on` turns the fraudulent website warning on, to measure it.
const FRAUD_WARNING_VARIABLE: &str = "LITHEMUX_SPIKE_FRAUD_WARNING";

pub type WebBrowsers = Browsers<Page>;

/// A page on the window, hidden until it is placed and shown. A click in it
/// calls `clicked`.
pub fn open_page(window: &gpui::Window, clicked: impl Fn() + 'static) -> Result<Page, PageError> {
    let url = std::env::var(URL_VARIABLE).unwrap_or_else(|_| "about:blank".to_string());
    let warning = match std::env::var(FRAUD_WARNING_VARIABLE).as_deref() {
        Ok("on") => FraudWarning::On,
        _ => FraudWarning::Off,
    };
    Page::open(window, &url, warning, clicked)
}

/// What WebKit refuses is left as it was: there is no one to tell in the
/// middle of a frame, and the next change tries again.
impl Surface for Page {
    fn show(&self, visible: bool) {
        let _ = Page::show(self, visible);
    }

    fn focus(&self) {
        let _ = Page::focus(self);
    }

    fn give_back_focus(&self) {
        let _ = Page::give_back_focus(self);
    }
}
