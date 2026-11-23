//! PLAN §3.7 spike: a browser tab in a pane, behind the `webview-spike`
//! feature. Not persisted and not in the layout: one page per pane at
//! most, shown in place of the pane's terminal while its tab is chosen.
//! `browsers` keeps the pages and when they show; `pane_browser` paints
//! the tab and places the page where the pane's content is; `web_page`
//! is the WebKit page itself.
#![cfg(all(feature = "webview-spike", target_os = "macos"))]

pub mod browsers;
pub mod pane_browser;
pub mod web_page;

#[cfg(test)]
mod test;
