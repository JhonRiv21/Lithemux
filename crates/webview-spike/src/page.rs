use objc2::MainThreadMarker;
use raw_window_handle::HasWindowHandle;
use wry::dpi::{LogicalPosition, LogicalSize};
use wry::{Rect, WebView, WebViewBuilder, WebViewBuilderExtMacos, WebViewExtMacOS};

use crate::configuration::configuration;

#[derive(Debug, thiserror::Error)]
pub enum PageError {
    #[error("a web page can only be made on the main thread")]
    NotMainThread,
    #[error(transparent)]
    WebView(#[from] wry::Error),
}

/// Tells the app of every click in the page, through WebKit's message
/// handler (no network).
const TELL_CLICKS: &str =
    "document.addEventListener('mousedown', () => window.ipc.postMessage('click'), true);";

/// Whether WebKit's fraudulent website warning (Safe Browsing) is on.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FraudWarning {
    On,
    Off,
}

/// Where the page goes, in logical pixels from the top left of the window's
/// content.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Place {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

/// A web page on top of the window's view. Dropping it takes it away.
pub struct Page {
    webview: WebView,
}

impl Page {
    /// Opens `url` hidden; it shows once it is placed and shown. A click in
    /// the page calls `clicked`: the window's own view does not see it.
    pub fn open(
        window: &impl HasWindowHandle,
        url: &str,
        fraud_warning: FraudWarning,
        clicked: impl Fn() + 'static,
    ) -> Result<Page, PageError> {
        let mtm = MainThreadMarker::new().ok_or(PageError::NotMainThread)?;
        let webview = WebViewBuilder::new()
            .with_webview_configuration(configuration(mtm, fraud_warning == FraudWarning::On))
            .with_initialization_script(TELL_CLICKS)
            .with_ipc_handler(move |_| clicked())
            .with_url(url)
            .with_visible(false)
            .with_focused(false)
            .with_accept_first_mouse(true)
            .build_as_child(window)?;
        Ok(Page { webview })
    }

    /// `wry` takes the window for granted when placing or focusing (it
    /// unwraps it): a view that is not in a window is left alone, rather
    /// than panicking in the middle of a frame.
    fn is_in_a_window(&self) -> bool {
        self.webview.webview().window().is_some()
    }

    pub fn place(&self, place: Place) -> Result<(), PageError> {
        if !self.is_in_a_window() {
            return Ok(());
        }
        self.webview.set_bounds(Rect {
            position: LogicalPosition::new(place.x, place.y).into(),
            size: LogicalSize::new(place.width, place.height).into(),
        })?;
        Ok(())
    }

    pub fn show(&self, visible: bool) -> Result<(), PageError> {
        self.webview.set_visible(visible)?;
        Ok(())
    }

    /// The keyboard goes to the page.
    pub fn focus(&self) -> Result<(), PageError> {
        if !self.is_in_a_window() {
            return Ok(());
        }
        self.webview.focus()?;
        Ok(())
    }

    /// The keyboard goes back to the window's own view.
    pub fn give_back_focus(&self) -> Result<(), PageError> {
        self.webview.focus_parent()?;
        Ok(())
    }

    pub fn url(&self) -> Option<String> {
        self.webview.url().ok()
    }
}
