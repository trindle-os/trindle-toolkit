//! What moves by itself as time passes: glows, the bell's flash, labels and when next to wake.

use super::*;

impl<M: Clone> Ui<M> {
    pub(super) fn start_glows(&mut self, now: Instant) {
        fn walk<M>(n: &Node<M>, anchor: Anchor, path: &mut Vec<usize>, out: &mut Vec<(String, u32)>) {
            if n.glow > 0 {
                out.push((id_of(n.key.as_ref(), anchor, path), n.glow));
            }
            let anchor = n.key.as_deref().map(|k| (k, path.len())).or(anchor);
            for (i, item) in n.items().iter().enumerate() {
                path.push(i);
                walk(item, anchor, path, out);
                path.pop();
            }
        }
        let mut found = Vec::new();
        walk(&self.tree, None, &mut Vec::new(), &mut found);
        let mut kept = HashMap::new();
        for (id, turn) in found {
            let was = self.glows.get(&id).copied();
            let state = match was {
                Some((t, started)) if t == turn => (t, started),
                _ => (turn, Some(now)),
            };
            kept.insert(id, state);
        }
        self.glows = kept;
    }

    pub(super) fn glow_at(&self, id: &str, now: Instant) -> f32 {
        let Some((_, Some(started))) = self.glows.get(id) else { return 0.0 };
        let k = now.saturating_duration_since(*started).as_secs_f32() / theme::sheet().glow.as_secs_f32();
        match k {
            k if k >= 1.0 => 0.0,
            _ if !self.moving => 1.0,
            k if k < 0.25 => k / 0.25,
            k if k < 0.6 => 1.0,
            k => (1.0 - k) / 0.4,
        }
    }

    pub(super) fn bell_at(&self, now: Instant) -> f32 {
        let Some(started) = self.bell else { return 0.0 };
        let k = now.saturating_duration_since(started).as_secs_f32() / theme::sheet().bell.as_secs_f32();
        match k {
            k if k >= 1.0 => 0.0,
            _ if !self.moving => 1.0,
            k if k < 0.2 => k / 0.2,
            k => (1.0 - k) / 0.8,
        }
    }

    pub(super) fn paint_bell(&self, canvas: &mut PixmapMut) {
        let mut p = Painter::default();
        self.bell_on(&mut p, (canvas.width() as f32, canvas.height() as f32));
        p.play(canvas);
    }

    pub(super) fn bell_on(&self, p: &mut Painter, room: (f32, f32)) {
        let by = self.bell_at(self.frame.get());
        if by <= 0.0 {
            return;
        }
        let width = self.px(theme::sheet().glow_width);
        let edge = Rect { x: 0.0, y: 0.0, w: room.0, h: room.1 }.inset(width / 2.0);
        if let Some(path) = rounded(edge, 0.0) {
            p.stroke_path(path, Stroke { width, ..Stroke::default() }, theme::alpha(self.theme.accent, by));
        }
    }

    pub(super) fn spinning(&self) -> bool {
        self.moving && self.spots.iter().any(|s| matches!(self.tree.at(&s.path).map(|n| &n.what), Some(What::Spinner { .. })))
    }
}
