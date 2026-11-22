//! Paints the split tree: each split is a row or a column of two sides
//! with a divider that can be dragged.

use gpui::{
    AnyElement, Context, Div, DragMoveEvent, Stateful, Window, div, prelude::*, px, relative, rgb,
};
use lithemux_core::modules::workspace::{Axis, Node};

use super::tab_drag::DividerDrag;
use super::view::Workspace;
use crate::theme;

const DIVIDER: f32 = 5.0;

/// The bar between the two sides of a split; dragging it resizes them.
fn divider_of(split: u32) -> Stateful<Div> {
    div()
        .id(("divider", split as usize))
        .flex_none()
        .bg(rgb(theme::BORDER))
        .hover(|style| style.bg(rgb(theme::ACCENT)))
        .on_drag(DividerDrag { split }, |drag, _, _, cx| cx.new(|_| *drag))
}

impl Workspace {
    /// The divider follows the pointer: its place within the split is the
    /// share of the first side.
    fn drag_divider(
        &mut self,
        split: u32,
        axis: Axis,
        event: &DragMoveEvent<DividerDrag>,
        cx: &mut Context<Self>,
    ) {
        let (position, bounds) = (event.event.position, event.bounds);
        let (at, start, length) = match axis {
            Axis::Horizontal => (position.x, bounds.origin.x, bounds.size.width),
            Axis::Vertical => (position.y, bounds.origin.y, bounds.size.height),
        };
        if length <= px(0.0) {
            return;
        }
        let share = ((at - start) / length).clamp(0.0, 1.0) * 1000.0;
        self.layout.resize(split, share.round() as u16);
        self.save_soon(cx);
        cx.notify();
    }

    pub(super) fn render_node(
        &self,
        node: &Node,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        match node {
            Node::Pane(pane) => self.render_pane(*pane, window, cx),
            Node::Split {
                id,
                axis,
                share,
                first,
                second,
            } => self.render_split((*id, *axis, *share), (first, second), window, cx),
        }
    }

    /// Two sides in a row or a column, with the divider between them.
    fn render_split(
        &self,
        (id, axis, share): (u32, Axis, u16),
        (first, second): (&Node, &Node),
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let first_share = relative(f32::from(share) / 1000.0);
        let first_side = div().flex_none().overflow_hidden();
        let divider = divider_of(id);
        let container = div()
            .id(("split", id as usize))
            .flex()
            .size_full()
            .on_drag_move(
                cx.listener(move |this, event: &DragMoveEvent<DividerDrag>, _, cx| {
                    if event.drag(cx).split == id {
                        this.drag_divider(id, axis, event, cx);
                    }
                }),
            );
        let (container, first_side, divider) = match axis {
            Axis::Horizontal => (
                container.flex_row(),
                first_side.h_full().w(first_share),
                divider.h_full().w(px(DIVIDER)).cursor_col_resize(),
            ),
            Axis::Vertical => (
                container.flex_col(),
                first_side.w_full().h(first_share),
                divider.w_full().h(px(DIVIDER)).cursor_row_resize(),
            ),
        };
        container
            .child(first_side.child(self.render_node(first, window, cx)))
            .child(divider)
            .child(
                div()
                    .flex_1()
                    .min_w_0()
                    .min_h_0()
                    .overflow_hidden()
                    .child(self.render_node(second, window, cx)),
            )
            .into_any_element()
    }
}
