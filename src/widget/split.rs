//! A split pane: two areas with a divider between them that is dragged.

use super::{node, Lay, Length, Node, Rect, Spot, What};
use crate::theme;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Start {
    Part(f32),
    Points(f32),
}

pub const SPLIT_LEAST: f32 = 120.0;
pub const SPLIT_STEP: f32 = 16.0;

pub fn split<M>(key: impl Into<String>, first: Node<M>, second: Node<M>) -> Node<M> {
    split_of(key, first, second, false)
}

pub fn split_down<M>(key: impl Into<String>, first: Node<M>, second: Node<M>) -> Node<M> {
    split_of(key, first, second, true)
}

fn split_of<M>(key: impl Into<String>, first: Node<M>, second: Node<M>, down: bool) -> Node<M> {
    let items = vec![first, second, node(What::Divider { down }, Length::Fill)];
    Node { height: Length::Fill, key: Some(key.into()), ..node(What::Split { items, down, start: Start::Part(0.5), least: (SPLIT_LEAST, SPLIT_LEAST) }, Length::Fill) }
}

impl<M> Node<M> {
    pub fn start(mut self, at: Start) -> Self {
        if let What::Split { start, .. } = &mut self.what {
            *start = at;
        }
        self
    }

    pub fn least(mut self, first: f32, second: f32) -> Self {
        if let What::Split { least, .. } = &mut self.what {
            *least = (first.max(0.0), second.max(0.0));
        }
        self
    }
}

pub(crate) fn first_side(kept: Option<f32>, start: Start, least: (f32, f32), room: f32, scale: f32) -> f32 {
    let want = match (kept, start) {
        (Some(p), _) | (None, Start::Points(p)) => p * scale,
        (None, Start::Part(f)) => room * f.clamp(0.0, 1.0),
    };
    want.min(room - least.1 * scale).max(least.0 * scale).min(room).max(0.0).round()
}

impl Lay<'_> {
    pub(super) fn place_split<M>(&self, n: &Node<M>, rect: Rect, id: &str, child: &mut dyn FnMut(usize, &Node<M>, Rect, &mut Vec<Spot>), spots: &mut Vec<Spot>) {
        let What::Split { items, down, start, least } = &n.what else { return };
        let line = self.px(theme::sheet().divider);
        let grab = self.px(theme::TARGET_LEAST).max(line);
        let room = (if *down { rect.h } else { rect.w } - line).max(0.0);
        let a = first_side(self.splits.and_then(|s| s.get(id)).copied(), *start, *least, room, self.scale);
        let b = room - a;
        let edge = a + ((line - grab) / 2.0).round();
        let (one, two, bar) = if *down {
            (Rect { h: a, ..rect }, Rect { y: rect.y + a + line, h: b, ..rect }, Rect { y: rect.y + edge, h: grab, ..rect })
        } else {
            (Rect { w: a, ..rect }, Rect { x: rect.x + a + line, w: b, ..rect }, Rect { x: rect.x + edge, w: grab, ..rect })
        };
        for (i, r) in [one, two, bar].into_iter().enumerate() {
            child(i, &items[i], r, spots);
        }
    }

    pub(super) fn measure_split<M>(&self, n: &Node<M>, max_w: f32) -> (f32, f32) {
        let What::Split { items, down, .. } = &n.what else { return (0.0, 0.0) };
        if n.height == Length::Fill && !self.whole.get() {
            return (max_w, 0.0);
        }
        let (a, b) = (self.measure(&items[0], max_w).1, self.measure(&items[1], max_w).1);
        (max_w, if *down { a + b + self.px(theme::sheet().divider) } else { a.max(b) })
    }
}
