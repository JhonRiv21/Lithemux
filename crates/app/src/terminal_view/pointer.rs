//! Mouse and wheel on the view: selection, focus on click and scrolling.

use gpui::{
    Context, MouseButton, MouseDownEvent, MouseMoveEvent, MouseUpEvent, Pixels, Point, ScrollEvent,
    Window,
};

use super::keys::{KeyModifiers, KeyPress};
use super::session::{GridPoint, SelectionKind};
use super::view::TerminalView;

impl TerminalView {
    fn cell_at(&self, position: Point<Pixels>) -> Option<GridPoint> {
        let placement = self.placement?;
        let x = f32::from(position.x - placement.bounds.origin.x).max(0.0);
        let y = f32::from(position.y - placement.bounds.origin.y).max(0.0);
        let cell_width = f32::from(placement.cell.width);
        let column = x / cell_width;
        Some(GridPoint {
            line: (y / f32::from(placement.cell.height)) as usize,
            column: column as usize,
            right_half: column.fract() > 0.5,
        })
    }

    pub(super) fn on_mouse_down(
        &mut self,
        event: &MouseDownEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        window.focus(&self.focus_handle, cx);
        let Some(at) = self.cell_at(event.position) else {
            return;
        };
        let kind = match event.click_count {
            0 | 1 => SelectionKind::Cells,
            2 => SelectionKind::Words,
            _ => SelectionKind::Lines,
        };
        self.session.start_selection(kind, at);
        self.selecting = true;
        cx.notify();
    }

    pub(super) fn on_mouse_move(
        &mut self,
        event: &MouseMoveEvent,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if !self.selecting || event.pressed_button != Some(MouseButton::Left) {
            return;
        }
        if let Some(to) = self.cell_at(event.position) {
            self.session.extend_selection(to);
            cx.notify();
        }
    }

    pub(super) fn on_mouse_up(&mut self, _: &MouseUpEvent, _: &mut Window, cx: &mut Context<Self>) {
        self.selecting = false;
        // A click without a drag leaves an empty selection behind.
        if self.session.selected_text().is_none() {
            self.session.clear_selection();
        }
        cx.notify();
    }

    pub(super) fn on_scroll(
        &mut self,
        event: &ScrollEvent,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let Some(placement) = self.placement else {
            return;
        };
        let cell_height = f32::from(placement.cell.height);
        let moved = f32::from(event.delta.pixel_delta(placement.cell.height).y);
        let total = self.wheel_remainder + moved / cell_height;
        let lines = total.trunc();
        self.wheel_remainder = total - lines;
        let lines = lines as i32;
        if lines == 0 {
            return;
        }
        if self.session.modes().wheel_as_arrows {
            let press = KeyPress {
                key: if lines > 0 { "up" } else { "down" },
                modifiers: KeyModifiers::default(),
            };
            for _ in 0..lines.unsigned_abs() {
                self.session.press(press);
            }
        } else {
            self.session.scroll(lines);
        }
        cx.notify();
    }
}
