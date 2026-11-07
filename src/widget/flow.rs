//! A flow box's layout: items at their own size, left to right, on to the next line when a line is full.

use super::{Lay, Node, Rect};

impl Lay<'_> {
    pub(super) fn flowed<M>(&self, items: &[Node<M>], gap: f32, width: f32) -> (Vec<Rect>, f32) {
        let gap = self.px(gap);
        let (mut x, mut y, mut line) = (0.0, 0.0, 0.0f32);
        let mut rects = Vec::with_capacity(items.len());
        for item in items {
            let (w, h) = self.measure(item, width);
            if x > 0.0 && x + w > width {
                (x, y, line) = (0.0, y + line + gap, 0.0);
            }
            rects.push(Rect { x, y, w, h });
            x += w + gap;
            line = line.max(h);
        }
        (rects, y + line)
    }
}
