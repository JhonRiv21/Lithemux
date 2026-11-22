//! The gpui entity: owns a session, routes input to it and paints it.

use std::rc::Rc;

use gpui::{
    App, Bounds, ClipboardItem, Context, ElementInputHandler, FocusHandle, Focusable, IntoElement,
    KeyDownEvent, MouseButton, Pixels, Render, Size, Subscription, Task, Window, canvas, div,
    prelude::*,
};

use super::composition::Composition;
use super::grid::CursorSnapshot;
use super::ime_log::{kind_of_key, trace};
use super::keys::{KeyModifiers, KeyPress};
use super::paint::{FontMetrics, Frame, measure, paint_frame};
use super::pty::GridSize;
use super::session::TerminalSession;

/// Where the grid was painted last, to map pointer positions to cells.
#[derive(Clone, Copy)]
pub(super) struct Placement {
    pub bounds: Bounds<Pixels>,
    pub cell: Size<Pixels>,
    pub cursor: Option<CursorSnapshot>,
}

pub struct TerminalView {
    pub(super) session: Rc<TerminalSession>,
    pub(super) focus_handle: FocusHandle,
    pub(super) composition: Composition,
    pub(super) placement: Option<Placement>,
    pub(super) selecting: bool,
    pub(super) wheel_remainder: f32,
    _repaint_on_output: Task<()>,
    _focus_subscriptions: [Subscription; 2],
}

impl TerminalView {
    /// `wakes` is the receiver `TerminalSession::spawn` returned with `session`.
    /// The view shows the session and types into it; whoever else holds it
    /// keeps it alive when the view goes away.
    pub fn new(
        session: Rc<TerminalSession>,
        wakes: async_channel::Receiver<()>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let focus_handle = cx.focus_handle();
        let repaint_on_output = cx.spawn(async move |view, cx| {
            while wakes.recv().await.is_ok() {
                if view.update(cx, |_, cx| cx.notify()).is_err() {
                    return;
                }
            }
        });
        let focus_subscriptions = [
            cx.on_focus(&focus_handle, window, |view, _, cx| {
                view.report_focus(true, cx)
            }),
            cx.on_blur(&focus_handle, window, |view, _, cx| {
                view.report_focus(false, cx)
            }),
        ];
        Self {
            session,
            focus_handle,
            composition: Composition::default(),
            placement: None,
            selecting: false,
            wheel_remainder: 0.0,
            _repaint_on_output: repaint_on_output,
            _focus_subscriptions: focus_subscriptions,
        }
    }

    pub fn has_exited(&self) -> bool {
        self.session.has_exited()
    }

    /// Typed input: the view follows the cursor again.
    pub(super) fn send(&mut self, bytes: Vec<u8>, cx: &mut Context<Self>) {
        if bytes.is_empty() {
            return;
        }
        self.session.write(bytes);
        self.follow_input(cx);
    }

    fn follow_input(&mut self, cx: &mut Context<Self>) {
        self.session.scroll_to_bottom();
        self.session.clear_selection();
        cx.notify();
    }

    fn report_focus(&mut self, focused: bool, cx: &mut Context<Self>) {
        trace(format_args!("focus {focused}"));
        self.session.report_focus(focused);
        cx.notify();
    }

    fn prepare_frame(
        &mut self,
        bounds: Bounds<Pixels>,
        metrics: &FontMetrics,
        focused: bool,
    ) -> Frame {
        let columns = (bounds.size.width / metrics.cell.width).floor();
        let lines = (bounds.size.height / metrics.cell.height).floor();
        self.session.resize(GridSize {
            columns: columns.clamp(2.0, 1000.0) as u16,
            lines: lines.clamp(1.0, 1000.0) as u16,
            cell_width: f32::from(metrics.cell.width).round() as u16,
            cell_height: f32::from(metrics.cell.height).round() as u16,
        });
        let snapshot = self.session.snapshot();
        self.placement = Some(Placement {
            bounds,
            cell: metrics.cell,
            cursor: snapshot.cursor,
        });
        Frame {
            snapshot,
            marked: self.composition.marked().to_string(),
            focused,
        }
    }

    fn on_key_down(&mut self, event: &KeyDownEvent, _: &mut Window, cx: &mut Context<Self>) {
        let keystroke = &event.keystroke;
        let modifiers = KeyModifiers {
            shift: keystroke.modifiers.shift,
            control: keystroke.modifiers.control,
            alt: keystroke.modifiers.alt,
            platform: keystroke.modifiers.platform,
        };
        let only_platform = modifiers
            == KeyModifiers {
                platform: true,
                ..KeyModifiers::default()
            };
        match keystroke.key.as_str() {
            "v" if only_platform => self.paste(cx),
            "c" if only_platform => self.copy(cx),
            key => {
                let sent = self.session.press(KeyPress { key, modifiers });
                trace(format_args!(
                    "key_down {} {modifiers:?} held {} -> {}",
                    kind_of_key(
                        key,
                        keystroke.key_char.is_some(),
                        modifiers.control || modifiers.platform
                    ),
                    event.is_held,
                    if sent {
                        "sent as a key"
                    } else {
                        "left to the input method"
                    }
                ));
                if !sent {
                    return;
                }
                self.composition.reset();
                self.follow_input(cx);
            }
        }
        cx.stop_propagation();
    }

    fn paste(&mut self, cx: &mut Context<Self>) {
        let Some(text) = cx.read_from_clipboard().and_then(|item| item.text()) else {
            return;
        };
        self.session.paste(&text);
        self.composition.reset();
        self.follow_input(cx);
    }

    fn copy(&mut self, cx: &mut Context<Self>) {
        if let Some(text) = self.session.selected_text() {
            cx.write_to_clipboard(ClipboardItem::new_string(text));
        }
    }
}

impl Focusable for TerminalView {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for TerminalView {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let view = cx.entity();
        let focus_handle = self.focus_handle.clone();
        div()
            .size_full()
            .track_focus(&self.focus_handle)
            .on_key_down(cx.listener(Self::on_key_down))
            .on_mouse_down(MouseButton::Left, cx.listener(Self::on_mouse_down))
            .on_mouse_move(cx.listener(Self::on_mouse_move))
            .on_mouse_up(MouseButton::Left, cx.listener(Self::on_mouse_up))
            .on_scroll(cx.listener(Self::on_scroll))
            .child(
                canvas(
                    {
                        let view = view.clone();
                        let focus_handle = focus_handle.clone();
                        move |bounds, window, cx| {
                            let metrics = measure(window);
                            let focused = focus_handle.is_focused(window);
                            let frame = view.update(cx, |view, _| {
                                view.prepare_frame(bounds, &metrics, focused)
                            });
                            (metrics, frame)
                        }
                    },
                    move |bounds, (metrics, frame), window, cx| {
                        paint_frame(bounds, &metrics, &frame, window, cx);
                        let input = ElementInputHandler::new(bounds, view);
                        window.handle_input(&focus_handle, input, cx);
                    },
                )
                .size_full(),
            )
    }
}
