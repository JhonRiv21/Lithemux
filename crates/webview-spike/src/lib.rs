//! PLAN §3.7 spike: a web page in a pane of lithemux, as a `WKWebView`
//! that is a child of the window's view. Made with `wry`; the only thing
//! `wry` does not offer, the WebKit configuration, is in `configuration`.
//! Behind the app's `webview-spike` feature; never in a release build.

#![cfg(target_os = "macos")]

mod configuration;
pub mod page;
