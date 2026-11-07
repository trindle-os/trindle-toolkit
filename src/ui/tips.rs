//! Labels on pointing and the words a refusal says.

use super::*;

impl<M: Clone> Ui<M> {
    pub fn say_refused(&mut self, now: Instant) {
        self.say_refused_as(now, crate::window::only_your_own());
    }

    pub(super) fn say_refused_as(&mut self, now: Instant, words: &'static str) {
        self.refused_words = words;
        let at = self.pointer.and_then(|(x, y)| self.hit(x, y)).or_else(|| self.focus.clone());
        self.refused = Some((at, now));
    }

    #[cfg(any(test, feature = "testing"))]
    pub fn saying_refused(&self) -> bool {
        self.refused.is_some()
    }

    pub(super) fn paint_refused(&self, canvas: &mut PixmapMut) -> bool {
        let mut p = Painter::default();
        let said = self.refused_on(&mut p, (canvas.width() as f32, canvas.height() as f32));
        p.play(canvas);
        said
    }

    pub(super) fn refused_on<'s>(&'s self, p: &mut Painter<'s>, room: (f32, f32)) -> bool {
        let Some((at, _)) = &self.refused else { return false };
        let (cw, ch) = room;
        let layout = label_layout(&self.fonts, self.refused_words, self.scale, cw);
        let gap = (theme::sheet().label_gap * self.scale).round();
        let foot = Rect { x: 0.0, y: ch - gap, w: cw, h: 0.0 };
        let r = at.as_deref().and_then(|id| self.spot(id)).map_or(foot, |s| s.rect);
        tip_box_on(p, &self.theme, &self.fonts, self.scale, &layout, r, room);
        true
    }

    pub(super) fn paint_tip(&self, canvas: &mut PixmapMut) {
        let mut p = Painter::default();
        self.tip_on(&mut p, (canvas.width() as f32, canvas.height() as f32));
        p.play(canvas);
    }

    pub(super) fn tip_on<'s>(&'s self, p: &mut Painter<'s>, room: (f32, f32)) {
        if let Some((layout, named)) = self.tip_target(room.0, room.1) {
            tip_box_on(p, &self.theme, &self.fonts, self.scale, &layout, named, room);
        }
    }

    #[cfg(test)]
    pub(super) fn tip_drawn(&self, canvas: &mut PixmapMut) -> Option<Rect> {
        let (layout, named) = self.tip_target(canvas.width() as f32, canvas.height() as f32)?;
        Some(tip_box(canvas, &self.theme, &self.fonts, self.scale, &layout, named, canvas.height() as f32))
    }

    pub(super) fn tip_target(&self, cw: f32, ch: f32) -> Option<(Layout, Rect)> {
        let tip = self.tip.as_ref().filter(|t| matches!(t.state, TipState::Shown(_)))?;
        let text = self.label_of(&tip.id)?;
        let spot = self.spot(&tip.id)?;
        let part = self.part_at(&tip.id).filter(|p| self.cuts.borrow().contains_key(&(tip.id.clone(), Some(*p))) || self.said.borrow().contains_key(&tip.id) || self.tool_at(&tip.id) == Some(*p));
        let named = if self.eyes.over.as_deref() == Some(tip.id.as_str()) {
            self.eye_rect(spot.rect)
        } else {
            part.and_then(|p| self.part_rect(&tip.id, p)).unwrap_or(spot.rect)
        };
        if self.out_labels {
            let k = self.k();
            let at = Rect { x: named.x / k, y: named.y / k, w: named.w / k, h: named.h / k };
            let layout = label_layout(&self.fonts, &text, self.scale, f32::MAX);
            let k = theme::sheet();
            let words = layout.widest();
            let size = ((words.ceil() / self.scale + 2.0 * k.label_padding.0.round()).ceil(),
                        (layout.height.ceil() / self.scale + 2.0 * k.label_padding.1.round()).ceil());
            *self.sent_out.borrow_mut() = Some(Outside { words: text, at, size });
            return None;
        }
        let layout = label_layout(&self.fonts, &text, self.scale, cw);
        let k = theme::sheet();
        let (h, gap) = (layout.height.ceil() + 2.0 * (k.label_padding.1 * self.scale).round(), (k.label_gap * self.scale).round());
        let room = named.y + named.h + gap + h <= ch || (named.y - gap - h >= 0.0 && named.y - gap <= ch);
        let named = if room {
            named
        } else if self.spot(&tip.id).is_some_and(|s| self.tells_more(s)) {
            Rect { y: (ch - h).max(0.0) - gap, h: 0.0, ..named }
        } else if self.node(&tip.id).is_some_and(|n| n.tip().is_none()) && self.said_here(&tip.id).is_some() {
            return None;
        } else {
            named
        };
        Some((layout, named))
    }

    pub(super) fn label_of(&self, id: &str) -> Option<String> {
        if self.eyes.over.as_deref() == Some(id) {
            return self.eye_name(id).map(str::to_string);
        }
        let node = self.node(id)?;
        if let (Some(i), Some(crate::role::Role::Toolbar { tools, .. })) = (self.tool_at(id), node.says.as_deref().and_then(|s| s.role.as_ref())) {
            return tools.get(i).map(|t| t.0.clone());
        }
        if self.labels == Labels::Always && self.named(node) {
            return None;
        }
        let cut = self.whole_here(id).or_else(|| self.cut_here(id));
        let why = node.why.as_ref().filter(|_| node.off).map(|w| format!("\n{w}")).unwrap_or_default();
        if let Some(tip) = node.tip() {
            if self.is_armed(id) {
                return Some(crate::common::ui::click_again(tip));
            }
            let leads = node.leads.as_ref().map(|to| format!("\n{to}")).unwrap_or_default();
            return Some(match cut.filter(|whole| whole != tip) {
                Some(whole) => format!("{whole} - {tip}{leads}{why}"),
                None => format!("{tip}{leads}{why}"),
            });
        }
        match (cut, why.trim()) {
            (Some(c), w) if !w.is_empty() => Some(format!("{c}\n{w}")),
            (None, w) if !w.is_empty() => Some(w.to_string()),
            (c, _) => c,
        }
    }

    pub(super) fn named(&self, node: &Node<M>) -> bool {
        matches!(&node.what, What::Button { words: None, tip, .. } if !tip.is_empty())
    }

    pub(super) fn follow_armed(&mut self) {
        let armed = self.armed.as_ref().map(|(id, _)| id.clone());
        self.tree.follow_armed(&|k| armed.as_deref() == Some(k));
    }

    pub(super) fn is_armed(&self, id: &str) -> bool {
        self.armed.as_ref().is_some_and(|(a, _)| a == id)
    }

    pub(super) fn scaled(&self, f: &theme::Fall) -> theme::Fall {
        let s = self.scale;
        theme::Fall { y: f.y * s, blur: f.blur * s, spread: f.spread * s, alpha: f.alpha }
    }

    pub(super) fn paint_flat(&self, canvas: &mut PixmapMut, r: Rect, radius: f32, falls: &theme::Falls, colour: Colour, border: Colour) {
        let mut p = Painter::default();
        self.flat_on(&mut p, r, radius, falls, colour, border);
        crate::paint::cpu::play(p.ops, canvas, &self.shadows);
    }

    pub(super) fn flat_on(&self, p: &mut Painter, r: Rect, radius: f32, falls: &theme::Falls, colour: Colour, border: Colour) {
        if !falls.is_empty() {
            let falls: Vec<theme::Fall> = falls.iter().map(|f| self.scaled(f)).collect();
            p.shadow(r, radius, &falls, self.theme.shadow);
        }
        fill_on(p, r, radius, colour);
        edge_on(p, r, radius, border, self.scale);
    }

    pub(super) fn paint_sunk(&self, canvas: &mut PixmapMut, r: Rect, radius: f32, rim: Colour, glow: f32) {
        let mut p = Painter::default();
        self.sunk_on(&mut p, r, radius, rim, glow);
        p.play(canvas);
    }

    pub(super) fn sunk_on(&self, p: &mut Painter, r: Rect, radius: f32, rim: Colour, glow: f32) {
        let t = &self.theme;
        let s = self.scale;
        fill_on(p, r, radius, t.field);
        if let Some(f) = t.field_inset {
            p.shadow_inside(r, radius, theme::Fall { y: f.y * s, blur: f.blur * s, spread: 0.0, alpha: f.alpha }, t.shadow);
        }
        edge_on(p, r, radius, rim, s);
        if glow > 0.0 {
            let width = self.px(theme::sheet().glow_width).max(1.0);
            if let Some(path) = rounded(r.inset(-width / 2.0), radius + width / 2.0) {
                p.stroke_path(path, Stroke { width, ..Stroke::default() }, theme::alpha(t.accent, theme::sheet().glow_strength * glow));
            }
        }
    }
}
