//! A band dragged across a list where several rows can be picked: it picks every row it crosses.

use super::*;

pub(super) struct Band {
    id: String,
    from: (f32, f32),
    to: (f32, f32),
    moved: bool,
    kept: Vec<usize>,
}

impl<M: Clone> Ui<M> {
    pub(super) fn band_press(&mut self, id: &str, x: f32, y: f32) -> bool {
        let Some(What::List { many: Some(m), .. }) = self.node(id).map(|n| &n.what) else { return false };
        if self.row_at(id, y).is_some() && !self.mods.ctrl {
            return false;
        }
        let kept = if self.mods.ctrl { m.picked.clone() } else { Vec::new() };
        self.band = Some(Band { id: id.to_string(), from: (x, y), to: (x, y), moved: false, kept });
        true
    }

    fn crossed(&self, id: &str, a: f32, b: f32) -> Vec<usize> {
        let (Some(s), Some(What::List { rows, .. })) = (self.spot(id), self.node(id).map(|n| &n.what)) else { return Vec::new() };
        let row = self.row_of(id).max(1.0);
        let (top, foot) = ((a.min(b) - s.rect.y) / row, (a.max(b) - s.rect.y) / row);
        let first = top.floor().max(0.0) as usize;
        let last = (foot.ceil() as usize).min(rows.len());
        (first..last).collect()
    }

    pub(super) fn band_to(&mut self, x: f32, y: f32, reply: &mut Reply<M>) -> bool {
        let slop = self.px(RIGHT_SLOP);
        let Some(b) = &mut self.band else { return false };
        b.to = (x, y);
        if !b.moved && (x - b.from.0).abs() <= slop && (y - b.from.1).abs() <= slop {
            return true;
        }
        b.moved = true;
        let (id, from, mut picked) = (b.id.clone(), b.from.1, b.kept.clone());
        for r in self.crossed(&id, from, y) {
            if !picked.contains(&r) {
                picked.push(r);
            }
        }
        picked.sort_unstable();
        if let Some(What::List { many: Some(m), .. }) = self.node(&id).map(|n| &n.what) {
            if m.picked != picked {
                reply.messages.push((m.pick)(picked));
            }
        }
        self.tip = None;
        reply.redraw = true;
        true
    }

    pub(super) fn band_let_go(&mut self, reply: &mut Reply<M>) -> bool {
        if !self.band.take().is_some_and(|b| b.moved) {
            return false;
        }
        self.pressed = None;
        reply.redraw = true;
        true
    }

    pub(super) fn paint_band(&self, p: &mut Painter, spot: &Spot) {
        let Some(b) = self.band.as_ref().filter(|b| b.moved && b.id == spot.id) else { return };
        let r = spot.rect;
        let (x0, x1) = (b.from.0.min(b.to.0).max(r.x), b.from.0.max(b.to.0).min(r.x + r.w));
        let (y0, y1) = (b.from.1.min(b.to.1).max(r.y), b.from.1.max(b.to.1).min(r.y + r.h));
        if x1 <= x0 || y1 <= y0 {
            return;
        }
        let area = Rect { x: x0, y: y0, w: x1 - x0, h: y1 - y0 };
        fill_on(p, area, 0.0, theme::alpha(self.theme.accent, theme::sheet().selection));
        edge_on(p, area, 0.0, self.theme.accent, self.scale);
    }
}
