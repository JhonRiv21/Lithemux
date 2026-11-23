//! The browser tab of a pane, on the workspace: its tab, its content and
//! what the user does with it.

use gpui::{AnyElement, Context, Task, Window, canvas, div, prelude::*, rgb};
use lithemux_webview_spike::page::Place;

use super::web_page::open_page;
use crate::i18n::Message;
use crate::theme;
use crate::workspace::Workspace;

impl Workspace {
    /// Where the pages say which pane was clicked: the sender each page
    /// keeps, and the task that hands every click to the workspace.
    pub(crate) fn listen_to_page_clicks(
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> (async_channel::Sender<u32>, Task<()>) {
        let (clicks, clicked) = async_channel::unbounded::<u32>();
        let listening = cx.spawn_in(window, async move |this, cx| {
            while let Ok(pane) = clicked.recv().await {
                let handed = this.update_in(cx, |this, window, cx| {
                    this.page_clicked(pane, window, cx);
                });
                if handed.is_err() {
                    return;
                }
            }
        });
        (clicks, listening)
    }

    /// Opens the pane's browser tab, or chooses it.
    pub(crate) fn open_browser(&mut self, pane: u32, window: &mut Window, cx: &mut Context<Self>) {
        let clicks = self.browser_clicks.clone();
        let over = self.on_show_in(pane);
        let opened = self.browsers.open(pane, over, || {
            open_page(window, move || {
                let _ = clicks.try_send(pane);
            })
        });
        if let Err(error) = opened {
            let problem = Message::with("error.internal", "detail", error.to_string());
            self.notice = Some(self.texts.say(&problem));
        }
        self.layout.focus_pane(pane);
        self.changed(window, cx);
    }

    fn on_show_in(&self, pane: u32) -> Option<String> {
        let pane = self.layout.pane(pane)?;
        pane.active_session().map(str::to_string)
    }

    /// After any change of the layout: the page of a pane that is gone goes
    /// with it, and a pane that shows another session than before shows
    /// it, not its page. Also on every frame, so that no way of changing
    /// the layout can leave a page behind.
    pub(crate) fn follow_the_layout(&mut self) {
        self.browsers.follow_the_layout(&self.layout);
    }

    pub(crate) fn close_browser(&mut self, pane: u32, window: &mut Window, cx: &mut Context<Self>) {
        self.browsers.close(pane);
        self.changed(window, cx);
    }

    /// A click in a page: its pane is the one in focus, as a click on a
    /// terminal makes its own.
    pub(crate) fn page_clicked(&mut self, pane: u32, window: &mut Window, cx: &mut Context<Self>) {
        self.browsers.clicked(pane);
        if self.layout.active_pane().id != pane {
            self.layout.focus_pane(pane);
            self.changed(window, cx);
        }
    }

    pub(crate) fn render_browser_tab(
        &self,
        pane: u32,
        cx: &mut Context<Self>,
    ) -> Option<AnyElement> {
        if !self.browsers.has(pane) {
            return None;
        }
        let is_shown = self.browsers.is_chosen(pane);
        let tab = div()
            .id(("browser-tab", pane as usize))
            .flex()
            .flex_none()
            .items_center()
            .gap_1()
            .h_full()
            .px_2()
            .border_r_1()
            .border_color(rgb(theme::colors().border))
            .text_sm()
            .text_color(rgb(if is_shown {
                theme::colors().text
            } else {
                theme::colors().text_muted
            }))
            .when(is_shown, |tab| tab.bg(rgb(theme::colors().window)))
            .on_click(cx.listener(move |this, _, window, cx| this.open_browser(pane, window, cx)))
            .child("◎")
            .child(self.texts.get("workspace.browser_tab"))
            .child(
                div()
                    .id(("browser-close", pane as usize))
                    .px_1()
                    .rounded_sm()
                    .text_color(rgb(theme::colors().text_faint))
                    .on_click(cx.listener(move |this, _, window, cx| {
                        this.close_browser(pane, window, cx);
                        cx.stop_propagation();
                    }))
                    .child("×"),
            );
        Some(tab.into_any_element())
    }

    /// Where the page goes: the pane's content, measured as gpui lays it
    /// out on every frame, so the page follows any change of size.
    pub(crate) fn render_browser_content(&self, pane: u32) -> Option<AnyElement> {
        if !self.browsers.is_chosen(pane) {
            return None;
        }
        let page = self.browsers.shown_page(pane);
        let place = canvas(
            |_, _, _| {},
            move |bounds, _, _, _| {
                if let Some(page) = &page {
                    let _ = page.place(Place {
                        x: f64::from(bounds.origin.x),
                        y: f64::from(bounds.origin.y),
                        width: f64::from(bounds.size.width),
                        height: f64::from(bounds.size.height),
                    });
                }
            },
        )
        .size_full();
        Some(
            div()
                .flex_1()
                .min_h_0()
                .bg(rgb(theme::colors().window))
                .child(place)
                .into_any_element(),
        )
    }

    /// For measuring without touching the keyboard or the mouse:
    /// `LITHEMUX_SPIKE_OPEN=1` opens a page in the focused pane at start,
    /// `LITHEMUX_SPIKE_CLOSE_AFTER=<s>` closes it and
    /// `LITHEMUX_SPIKE_QUIT_AFTER=<s>` quits the app after that many seconds,
    /// with a page or without one.
    pub(crate) fn measure_from_environment(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let seconds = |name: &str| {
            std::env::var(name)
                .ok()
                .and_then(|value| value.parse::<u64>().ok())
                .map(std::time::Duration::from_secs)
        };
        // The quit comes first: a measurement without a page must end too.
        if let Some(after) = seconds("LITHEMUX_SPIKE_QUIT_AFTER") {
            cx.spawn(async move |_, cx| {
                cx.background_executor().timer(after).await;
                cx.update(|cx| cx.quit());
            })
            .detach();
        }
        if std::env::var("LITHEMUX_SPIKE_OPEN").as_deref() != Ok("1") {
            return;
        }
        let pane = self.layout.active_pane().id;
        self.open_browser(pane, window, cx);
        if let Some(after) = seconds("LITHEMUX_SPIKE_CLOSE_AFTER") {
            cx.spawn_in(window, async move |this, cx| {
                cx.background_executor().timer(after).await;
                let _ = this.update_in(cx, |this, window, cx| this.close_browser(pane, window, cx));
            })
            .detach();
        }
    }
}
