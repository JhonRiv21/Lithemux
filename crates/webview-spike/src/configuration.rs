//! The single place of this crate allowed to use `unsafe` (D-64, spike
//! only): every method of `objc2-web-kit` 0.3 is an `unsafe fn`.
#![allow(unsafe_code)]

use objc2::MainThreadMarker;
use objc2::rc::Retained;
use objc2_web_kit::{WKWebViewConfiguration, WKWebsiteDataStore};

/// What the page keeps lives in memory and goes with it; the fraudulent
/// website warning asks the network on its own, so it can be turned off.
pub fn configuration(
    mtm: MainThreadMarker,
    fraud_warning: bool,
) -> Retained<WKWebViewConfiguration> {
    // SAFETY: `mtm` proves this runs on the main thread, where WebKit must be
    // used; the configuration, its store and its preferences are created or
    // returned here and kept alive by their `Retained` while they are used.
    unsafe {
        let configuration = WKWebViewConfiguration::new(mtm);
        configuration.setWebsiteDataStore(&WKWebsiteDataStore::nonPersistentDataStore(mtm));
        configuration
            .preferences()
            .setFraudulentWebsiteWarningEnabled(fraud_warning);
        configuration
    }
}
