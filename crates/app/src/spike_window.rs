//! Phase 1 spike: one window with two terminals side by side. Throwaway;
//! only `terminal_view` is meant to stay.

use std::env;
use std::path::PathBuf;
use std::time::{Duration, Instant};

use gpui::{
    App, Bounds, Context, Entity, Focusable, IntoElement, KeyBinding, Render, TitlebarOptions,
    Window, WindowBounds, WindowOptions, actions, div, prelude::*, px, rgb, size,
};
use lithemux_app::terminal_view::{GridSize, ShellCommand, TerminalSession, TerminalView};

const PANES: usize = 2;
const FALLBACK_SHELL: &str = "/bin/zsh";

actions!(spike, [FocusNextPane, FocusPreviousPane, Quit]);

struct SpikeWindow {
    panes: Vec<Entity<TerminalView>>,
}

type StartedSession = (TerminalSession, async_channel::Receiver<()>);

impl SpikeWindow {
    fn new(sessions: Vec<StartedSession>, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let panes = sessions
            .into_iter()
            .map(|(session, wakes)| cx.new(|cx| TerminalView::new(session, wakes, window, cx)))
            .collect();
        Self { panes }
    }

    fn focus_pane(&self, step: usize, window: &mut Window, cx: &mut Context<Self>) {
        let current = self
            .panes
            .iter()
            .position(|pane| pane.focus_handle(cx).contains_focused(window, cx))
            .unwrap_or(0);
        if let Some(pane) = self.panes.get((current + step) % self.panes.len()) {
            window.focus(&pane.focus_handle(cx), cx);
        }
    }
}

/// `lithemux-app <program> [args]` runs that in every pane instead of the shell.
fn pane_command() -> ShellCommand {
    let mut args = env::args().skip(1);
    let Some(program) = args.next() else {
        return user_shell();
    };
    let mut command = ShellCommand::new(program);
    command.args = args.collect();
    command.cwd = env::current_dir().ok();
    command
}

const STALL_TICK: Duration = Duration::from_millis(16);
const STALL_REPORT_EVERY: Duration = Duration::from_secs(1);

/// Measurement aid: with LITHEMUX_SPIKE_STALLS set, prints every second the
/// longest time the main thread went without running a 16 ms timer.
fn report_main_thread_stalls(cx: &mut App) {
    if env::var_os("LITHEMUX_SPIKE_STALLS").is_none() {
        return;
    }
    cx.spawn(async move |cx| {
        let mut last_tick = Instant::now();
        let mut last_report = last_tick;
        let mut worst = Duration::ZERO;
        loop {
            cx.background_executor().timer(STALL_TICK).await;
            let now = Instant::now();
            worst = worst.max(now.duration_since(last_tick).saturating_sub(STALL_TICK));
            last_tick = now;
            if now.duration_since(last_report) >= STALL_REPORT_EVERY {
                eprintln!("main-thread worst stall: {} ms", worst.as_millis());
                worst = Duration::ZERO;
                last_report = now;
            }
        }
    })
    .detach();
}

fn user_shell() -> ShellCommand {
    let program = env::var_os("SHELL").map_or_else(|| PathBuf::from(FALLBACK_SHELL), PathBuf::from);
    let mut command = ShellCommand::new(program);
    command.args.push("-l".to_string());
    command.cwd = env::current_dir().ok();
    command
}

impl Render for SpikeWindow {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let panes = self.panes.len();
        div()
            .flex()
            .size_full()
            .bg(rgb(0x161618))
            .on_action(cx.listener(move |this, _: &FocusNextPane, window, cx| {
                this.focus_pane(1, window, cx)
            }))
            .on_action(cx.listener(move |this, _: &FocusPreviousPane, window, cx| {
                this.focus_pane(panes - 1, window, cx)
            }))
            .children(self.panes.iter().map(|pane| {
                let focused = pane.focus_handle(cx).contains_focused(window, cx);
                div()
                    .flex_1()
                    .min_w_0()
                    .h_full()
                    .p_1()
                    .border_1()
                    .border_color(if focused {
                        rgb(0x4d78cc)
                    } else {
                        rgb(0x2a2a2e)
                    })
                    .child(pane.clone())
            }))
    }
}

pub fn run() {
    gpui_platform::application().run(|cx: &mut App| {
        cx.bind_keys([
            KeyBinding::new("cmd-alt-right", FocusNextPane, None),
            KeyBinding::new("cmd-alt-left", FocusPreviousPane, None),
            KeyBinding::new("cmd-q", Quit, None),
        ]);
        cx.on_action(|_: &Quit, cx| cx.quit());
        cx.on_window_closed(|cx, _| cx.quit()).detach();

        let options = WindowOptions {
            window_bounds: Some(WindowBounds::Windowed(Bounds::centered(
                None,
                size(px(1280.0), px(720.0)),
                cx,
            ))),
            titlebar: Some(TitlebarOptions {
                title: Some("lithemux".into()),
                ..TitlebarOptions::default()
            }),
            ..WindowOptions::default()
        };
        report_main_thread_stalls(cx);
        if cx.text_system().all_font_names().is_empty() {
            eprintln!("lithemux: this build has no fonts, so no text will be drawn");
        }
        let command = pane_command();
        let sessions: Result<Vec<_>, _> = (0..PANES)
            .map(|_| TerminalSession::spawn(&command, GridSize::INITIAL))
            .collect();
        let opened = sessions.map_err(anyhow::Error::from).and_then(|sessions| {
            cx.open_window(options, |window, cx| {
                let root = cx.new(|cx| SpikeWindow::new(sessions, window, cx));
                if let Some(first) = root.read(cx).panes.first() {
                    window.focus(&first.focus_handle(cx), cx);
                }
                root
            })
        });
        if let Err(error) = opened {
            eprintln!("lithemux-app: {error}");
            cx.quit();
            return;
        }
        cx.activate(true);
    });
}
