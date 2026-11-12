//! Painting boxes, cards, labels and drawings.

use std::borrow::Cow;

use tiny_skia::IntRect;

use super::*;
use crate::paint::Painter;

impl<M: Clone> Ui<M> {
    fn look(&self, node: &Node<M>) -> Cow<'_, Theme> {
        if node.off { Cow::Owned(self.theme.faded()) } else { Cow::Borrowed(&self.theme) }
    }

    pub(super) fn paint_entry<'s>(&'s self, p: &mut Painter<'s>, size: (u32, u32), spot: &Spot, text: &str, tip: &str, eye: bool, off: bool, lit: bool) {
        let t = &self.theme;
        let r = spot.rect;
        let focused = self.focus.as_deref() == Some(&spot.id) && !self.on_eye(&spot.id);
        let secret = eye && !self.eye_open(&spot.id);
        let radius = self.corner(theme::Part::Box);
        let glow = self.glow_at(&spot.id, self.frame.get());
        let rest = if lit { theme::mix(t.field_edge, t.fg, 0.3) } else { t.field_edge };
        let rim = if focused { t.accent } else { theme::mix(rest, t.accent, glow) };
        self.sunk_on(p, r, radius, rim, glow);
        let pad = self.entry_pad(&spot.id);
        let room = if eye { self.eye_room(r) } else { 0.0 };
        if eye {
            let spot = spot.clone();
            self.on_pixels(p, size, move |c| self.paint_eye(c, &spot, off));
        }
        let inner = Rect { x: r.x + pad, y: r.y, w: r.w - 2.0 * pad - room, h: r.h };
        if inner.w < 1.0 {
            return;
        }
        let Some(at) = IntRect::from_xywh(inner.x as i32, inner.y as i32, inner.w as u32, inner.h.max(1.0) as u32) else { return };
        p.push_clip(at, 0.0);
        let outside = self.pixels_at.take();
        let size = (at.width(), at.height());
        let state = self.edits.get(&spot.id);
        let scroll = state.map_or(0.0, |e| e.scroll);
        let mut at = state.map_or(text.len(), |e| e.caret);
        let preedit = self.preedit_in(&spot.id);
        let pre = preedit.map(|(p, c)| (at..at + p.len(), c.map(|(a, b)| (at + a, at + b))));
        let joined;
        let text = match (preedit, &pre) {
            (Some((p, _)), Some((word, c))) => {
                joined = with_preedit(text, at, p);
                at = c.map_or(word.end, |c| c.1);
                joined.as_str()
            }
            _ => text,
        };
        let caret_at = |layout: &Layout, at: usize| layout.caret(if secret { edit::to_dots(text, at) } else { at });
        let words = if text.is_empty() { 0.0 } else { self.entry_layout(&spot.id, text, secret).width };
        let scroll = scroll - self.entry_lead(&spot.id, words, inner.w);
        let layout = self.paint_entry_words(p, size, spot, (text, tip), secret, &pre, state, scroll, inner.w);
        let hidden = pre.as_ref().is_some_and(|(_, c)| c.is_none());
        if focused && self.keys_here && !hidden {
            let (x, top, h) = if text.is_empty() { layout.caret(0) } else { caret_at(&layout, at) };
            let w = self.px(theme::sheet().caret).max(1.0);
            let ty = self.entry_down(&spot.id, r.h, layout.height);
            let bar = Rect { x: (x - scroll - w / 2.0).round().clamp(0.0, (inner.w - w).max(0.0)), y: ty + top, w, h };
            p.round(bar, [0.0; 4], t.fg);
        }
        self.pixels_at.set(outside);
        p.pop();
    }

    fn paint_entry_words<'s>(&'s self, p: &mut Painter<'s>, size: (u32, u32), spot: &Spot, (text, tip): (&str, &str), secret: bool,
                             pre: &Option<(Range<usize>, Option<(usize, usize)>)>, state: Option<&Edit>, scroll: f32, room: f32) -> Rc<Layout> {
        let t = &self.theme;
        let style = self.entry_style(&spot.id);
        let caret_at = |layout: &Layout, at: usize| layout.caret(if secret { edit::to_dots(text, at) } else { at });
        if text.is_empty() {
            let place = self.entry_layout(&spot.id, tip, false);
            let (x, y, muted) = (self.entry_lead(&spot.id, place.width, room), self.entry_down(&spot.id, spot.rect.h, place.height), t.muted);
            self.on_pixels(p, size, move |c| self.fonts.draw(c, &place, x, y, muted));
            self.texts.get(&self.fonts, " ", &style, None)
        } else {
            let layout = self.entry_layout(&spot.id, text, secret);
            let ty = self.entry_down(&spot.id, spot.rect.h, layout.height);
            let sel = match pre {
                Some((_, c)) => c.map(|(a, b)| a.min(b)..a.max(b)),
                None => state.map(Edit::selection),
            };
            if let Some(sel) = sel.filter(|s| !s.is_empty()) {
                let shown = if secret { edit::to_dots(text, sel.start)..edit::to_dots(text, sel.end) } else { sel };
                let under = theme::over(t.field, theme::over(t.panel, t.bg));
                for (x, top, w, h) in layout.mark_bands(&[shown]) {
                    p.round(Rect { x: x - scroll, y: ty + top, w, h }, [0.0; 4], t.wash(theme::alpha(t.accent, theme::sheet().selection), &[under]));
                }
            }
            let (shown, fg) = (layout.clone(), t.fg);
            self.on_pixels(p, size, move |c| self.fonts.draw(c, &shown, -scroll, ty, fg));
            if let (false, None, Some(What::Entry { text: given, misspelt, .. })) = (secret, pre, self.node(&spot.id).map(|n| &n.what)) {
                let thick = self.px(theme::sheet().caret).max(1.0);
                for r in misspelt.iter().filter(|_| given == text) {
                    let ink = t.spelling;
                    if self.entry_lines(&spot.id) > 1 {
                        for (x, top, w, h) in layout.mark_bands(std::slice::from_ref(r)) {
                            self.on_pixels(p, size, move |c| crate::spelling::squiggle(c, x, x + w, ty + top + h - thick, thick, ink));
                        }
                        continue;
                    }
                    let (a, top, h) = caret_at(&layout, r.start);
                    let b = caret_at(&layout, r.end).0;
                    self.on_pixels(p, size, move |c| crate::spelling::squiggle(c, a.min(b) - scroll, a.max(b) - scroll, ty + top + h - thick, thick, ink));
                }
            }
            if let Some((word, _)) = &pre {
                let (a, top, h) = caret_at(&layout, word.start);
                let b = caret_at(&layout, word.end).0;
                let thick = self.px(theme::sheet().caret).max(1.0);
                p.round(Rect { x: a.min(b) - scroll, y: ty + top + h - thick, w: (b - a).abs(), h: thick }, [0.0; 4], t.fg);
            }
            layout
        }
    }

    pub(super) fn paint_one<'s>(&'s self, p: &mut Painter<'s>, size: (u32, u32), node: &'s Node<M>, spot: &Spot) {
        let t = self.look(node);
        let r = spot.rect;
        let lit = !node.off && (self.hover.as_deref() == Some(&spot.id) || self.pressed.as_deref() == Some(&spot.id));
        let at = spot.clone();
        match &node.what {
            What::Card(..) => self.paint_card(p, size, node, spot, t.clone(), lit),
            What::Label { .. } => self.paint_label(p, size, node, spot, &t),
            What::Rich { .. } => self.on_pixels(p, size, move |c| self.paint_rich(c, node, &at, &self.look(node))),
            What::Ring { .. } => self.paint_ring(p, node, spot, &t),
            What::Spinner { .. } => self.paint_spinner(p, node, spot, &t),
            What::Drawing { .. } => self.paint_drawing(p, size, node, spot, &t),
            What::Button { icon, press, on, words: Some(words), .. } if w::is_word_button(icon, *on) => {
                let back = if on.unwrap_or(false) || (lit && press.is_some()) { None } else { Some(self.control_at(&spot.path)) };
                let used = press.is_some() && !node.off;
                self.on_pixels(p, size, move |c| {
                    let shown = self.paint_word_button(c, r, words, used, back);
                    self.note_cut(&at.id, None, words, &shown);
                });
            }
            What::Button { icon, press, words: Some(words), .. } if node.quiet => {
                let used = press.is_some() && !node.off;
                self.on_pixels(p, size, move |c| {
                    let shown = self.paint_quiet_action(c, r, icon, words, used, lit, &at.path);
                    self.note_cut(&at.id, None, words, &shown);
                });
            }
            What::Button { icon, press, words: Some(words), for_good, .. } => {
                let used = press.is_some() && !node.off;
                self.on_pixels(p, size, move |c| {
                    let shown = self.paint_action(c, r, icon, words, used, lit, *for_good);
                    self.note_cut(&at.id, None, words, &shown);
                });
            }
            What::Button { .. } => self.on_pixels(p, size, move |c| self.paint_round_button(c, node, &at, &self.look(node), lit)),
            What::Switch { on, .. } => self.paint_switch(p, size, spot, *on, t.clone(), lit),
            What::Tick { on, .. } => self.paint_tick(p, size, spot, *on, &t, lit),
            What::Divider { down } => self.on_pixels(p, size, move |c| self.paint_divider(c, &at, *down, &self.look(node), lit)),
            What::Slider { value, min, max, .. } => self.paint_slider(p, size, spot, (*value, *min, *max), t.clone(), lit),
            What::Entry { text, tip, secret, .. } => self.paint_entry(p, size, spot, text, tip, *secret, node.off, lit),
            What::List { rows, chosen, here: true, .. } => {
                self.paint_places(p, size, &at, rows, *chosen);
                self.paint_gap(p, &at, rows.len());
            }
            What::List { rows, chosen, many, tree, .. } => {
                self.paint_list(p, size, &at, rows, *chosen, many.as_ref().map(|m| m.picked.as_slice()), tree.as_ref().map(|t| t.rows.as_slice()));
                self.paint_gap(p, &at, rows.len());
                self.paint_band(p, &at);
            }
            What::Choice { options, chosen, arrows, .. } => {
                self.on_pixels(p, size, move |c| self.paint_choice(c, &at, options, *chosen, arrows.as_ref().map(|a| a.subs.as_slice()), lit));
            }
            What::Dropdown { options, chosen, .. } => {
                let shown = chosen.and_then(|i| options.get(i)).map_or(w::nothing_chosen(), String::as_str);
                self.on_pixels(p, size, move |c| self.paint_dropdown(c, &at, shown, lit));
            }
            _ => {}
        }
    }

    fn paint_tick<'s>(&'s self, p: &mut Painter<'s>, size: (u32, u32), spot: &Spot, on: bool, t: &Theme, lit: bool) {
        let r = spot.rect;
        let side = self.px(theme::sheet().tick);
        let s = Rect { x: r.x, y: r.y + ((r.h - side) / 2.0).round(), w: side, h: side };
        let corner = self.corner(theme::Part::Tick);
        let back = if on { t.soft } else { self.control_at(&spot.path) };
        p.round(s, [corner; 4], back);
        if lit {
            self.on_pixels(p, size, move |c| self.hover_edge(c, s, corner));
        }
        if on {
            let k = side / w::TICK_SIZE;
            let at = |x: f32, y: f32| (s.x + x * k, s.y + y * k);
            let mut pb = tiny_skia::PathBuilder::new();
            let (a, b, c) = (at(4.5, 9.5), at(7.8, 12.8), at(13.5, 5.5));
            pb.move_to(a.0, a.1);
            pb.line_to(b.0, b.1);
            pb.line_to(c.0, c.1);
            if let Some(path) = pb.finish() {
                let line = tiny_skia::Stroke { width: (2.0 * k).max(1.0), line_cap: tiny_skia::LineCap::Round, line_join: tiny_skia::LineJoin::Round, ..tiny_skia::Stroke::default() };
                p.stroke_path(path, line, t.words_on(&[back]));
            }
        }
    }

    fn paint_switch<'s>(&'s self, p: &mut Painter<'s>, size: (u32, u32), spot: &Spot, on: bool, t: Cow<'s, Theme>, lit: bool) {
        let r = spot.rect;
        let (w, h) = (self.px(theme::sheet().switch.0), self.px(theme::sheet().switch.1));
        let s = Rect { x: r.x, y: r.y + ((r.h - h) / 2.0).round(), w, h };
        let corner = self.corner(theme::Part::Switch);
        p.round(s, [corner; 4], if on { t.soft } else { self.control_at(&spot.path) });
        if lit {
            self.on_pixels(p, size, move |c| self.hover_edge(c, s, corner));
        }
        let k = self.px(theme::sheet().knob);
        let inset = ((h - k) / 2.0).round();
        let x = if on { s.x + w - inset - k } else { s.x + inset };
        let at = Rect { x, y: s.y + inset, w: k, h: k };
        if on {
            self.on_pixels(p, size, move |c| knob(c, at, &t, self.scale));
        } else {
            p.round(at, [self.corner(theme::Part::Knob); 4], t.knob_off);
        }
    }

    fn paint_slider<'s>(&'s self, p: &mut Painter<'s>, size: (u32, u32), spot: &Spot, (value, min, max): (f32, f32, f32), t: Cow<'s, Theme>, lit: bool) {
        let r = spot.rect;
        let k = self.px(theme::sheet().knob);
        let track = self.px(theme::sheet().track);
        let mid = r.y + (r.h / 2.0).round();
        let line = Rect { x: r.x + k / 2.0 - track / 2.0, y: mid - track / 2.0, w: r.w - k + track, h: track };
        let part = if max > min { ((value - min) / (max - min)).clamp(0.0, 1.0) } else { 0.0 };
        let x = r.x + part * (r.w - k);
        let mut piece = |from: f32, to: f32, colour| {
            if to > from {
                p.round(Rect { x: from, w: to - from, ..line }, [self.corner(theme::Part::Slider); 4], colour);
            }
        };
        let end = line.x + line.w;
        if self.focus.as_deref() == Some(&spot.id) && self.keys_here {
            let clear = self.px(theme::sheet().focus_offset) * 2.0 + self.px(theme::sheet().focus_width).max(1.0);
            piece(line.x, x - clear, self.control_at(&spot.path));
            piece(x + k + clear, end, self.control_at(&spot.path));
            piece(line.x, x - clear, t.soft);
        } else {
            piece(line.x, end, self.control_at(&spot.path));
            piece(line.x, x + k / 2.0 + track / 2.0, t.soft);
        }
        let ring = Rect { x, y: mid - k / 2.0, w: k, h: k };
        self.on_pixels(p, size, move |c| knob(c, ring, &t, self.scale));
        if lit {
            let corner = self.corner(theme::Part::Knob);
            self.on_pixels(p, size, move |c| self.hover_edge(c, ring, corner));
        }
    }

    pub(super) fn paint_card<'s>(&'s self, p: &mut Painter<'s>, size: (u32, u32), node: &Node<M>, spot: &Spot, t: Cow<'s, Theme>, lit: bool) {
        let r = spot.rect;
        match &node.what {
            What::Card(..) if node.band => {
                p.round(r, [self.corner(theme::Part::Row); 4], t.band);
            }
            What::Card(..) if node.floats => {
                let (radius, t) = (self.corner(theme::Part::Menu), t);
                self.on_pixels(p, size, move |c| self.paint_flat(c, r, radius, &t.float, t.raised, t.card_edge));
            }
            What::Card(_, press) if node.chip => {
                let radius = self.corner(theme::Part::Chip);
                let back = if lit && press.is_some() { t.picked } else { self.control_at(&spot.path) };
                p.round(r, [radius; 4], back);
            }
            What::Card(_, press) if node.sunk => {
                let radius = self.corner(theme::Part::Box);
                let rest = if lit && press.is_some() { t.accent } else { t.field_edge };
                self.sunk_on(p, r, radius, rest, 0.0);
            }
            What::Card(_, press) if node.flat => {
                if lit && press.is_some() {
                    let radius = self.corner(theme::Part::Row);
                    p.round(r, [radius; 4], t.hover);
                    self.on_pixels(p, size, move |c| self.hover_edge(c, r, radius));
                }
            }
            What::Card(..) if node.side => {
                if let Some((which, out)) = node.area {
                    self.paint_area(p, r, which, out, &t);
                    return;
                }
                let line = self.scale.round().max(1.0);
                p.round(Rect { x: r.x + r.w - line, w: line, ..r }, [0.0; 4], t.side_line);
            }
            What::Card(_, press) => self.paint_page_card(p, size, node, spot, t, lit && press.is_some()),
            _ => {}
        }
    }

    fn paint_area(&self, p: &mut Painter, r: Rect, which: Area, out: [f32; 4], t: &Theme) {
            let [up, far, down, near] = out.map(|o| self.px(o));
            let all = Rect { x: r.x - near, y: r.y - up, w: r.w + near + far, h: r.h + up + down };
            let (surface, radius) = match which {
                Area::Top => (t.top_bar, 0.0),
                Area::Side | Area::Between => (t.side_bar, 0.0),
                Area::Foot => (t.foot, self.corner(theme::Part::Box)),
                Area::Controls => (t.top_bar, self.corner(theme::Part::Box)),
            };
            if surface.a > 0.0 {
                p.round(all, [radius; 4], surface);
            }
            let thick = (theme::sheet().area_line * self.scale).round().max(1.0);
            if t.area_line.a > 0.0 {
                match which {
                    Area::Top => p.round(Rect { y: all.y + all.h - thick, h: thick, ..all }, [0.0; 4], t.area_line),
                    Area::Foot => p.round(Rect { h: thick, ..all }, [0.0; 4], t.area_line),
                    Area::Side => p.round(Rect { x: all.x + all.w - thick, w: thick, ..all }, [0.0; 4], t.area_line),
                    Area::Controls | Area::Between => {}
                }
            }
    }

    fn paint_page_card<'s>(&'s self, p: &mut Painter<'s>, size: (u32, u32), node: &Node<M>, spot: &Spot, t: Cow<'s, Theme>, pressable_lit: bool) {
        let r = spot.rect;
        let depth = self.card_depth(&spot.path);
        let (radius, border) = match depth {
            0 => (self.corner(theme::Part::Card), t.edge),
            _ => (self.corner(theme::Part::Box), t.card_edge),
        };
        let border = theme::alpha(border, 0.0);
        let colour = t.card(depth);
        let colour = if pressable_lit { theme::mix(colour, t.fg, 0.06) } else { colour };
        self.on_pixels(p, size, move |c| self.paint_flat(c, r, radius, if depth == 0 { &t.panel_falls } else { &t.raised_falls }, colour, border));
        if node.current {
            self.on_pixels(p, size, move |c| self.current_edge(c, r, radius));
        }
        if pressable_lit {
            self.on_pixels(p, size, move |c| self.hover_edge(c, r, radius));
        }
        let shine = node.lit.max(self.glow_at(&spot.id, self.frame.get()));
        if shine > 0.0 {
            self.on_pixels(p, size, move |c| self.paint_lit(c, r, radius, shine));
        }
    }

    pub(super) fn paint_label<'s>(&'s self, p: &mut Painter<'s>, size: (u32, u32), node: &'s Node<M>, spot: &Spot, t: &Theme) {
        let r = spot.rect;
        if let What::Label { text, words, tone, marks } = &node.what {
            let style = words.style(self.scale);
            let colour = toned(t, *tone, if matches!(words, Words::Note | Words::Column | Words::Side | Words::Headline) { t.muted } else { t.fg });
            let full = self.texts.get(&self.fonts, text, &style, if node.cut == 1 { None } else { Some(r.w.max(1.0)) });
            if let Some((_, e)) = self.picked.as_ref().filter(|(id, _)| *id == spot.id) {
                let bands: Vec<Rect> = picked_bands(&full, e.selection()).into_iter().filter(|b| b.y < r.h)
                    .map(|band| Rect { x: r.x + band.x, y: r.y + band.y, w: band.w.min(r.w - band.x), h: band.h }).collect();
                let under = self.lies_on();
                let wash = t.wash_on(colour, theme::alpha(t.accent, theme::sheet().selection), under);
                for band in bands {
                    p.round(band, [0.0; 4], wash);
                }
            }
            let layout = match node.cut {
                0 => full,
                1 => Rc::new(full.cut(r.w, &self.texts.get(&self.fonts, "\u{2026}", &style, None))),
                lines => Rc::new(full.cut_after(lines as usize, r.w, &self.texts.get(&self.fonts, "\u{2026}", &style, None))),
            };
            self.note_cut(&spot.id, None, text, &layout);
            self.on_pixels(p, size, move |c| {
                self.paint_marks(c, &layout, (r.x, r.y), marks, colour);
                self.fonts.draw(c, &layout, r.x, r.y, colour);
            });
        }
    }

    pub(super) fn paint_ring(&self, p: &mut Painter, node: &Node<M>, spot: &Spot, t: &Theme) {
        let r = spot.rect;
        if let What::Ring { share, size, tone } = &node.what {
            let size = self.px(*size);
            let line = (size / 8.4).round().max(self.px(6.0));
            let (cx, cy) = (r.x + r.w / 2.0, r.y + r.h / 2.0);
            let radius = size / 2.0 - line / 2.0 - 1.0;
            let stroke = Stroke { width: line, line_cap: tiny_skia::LineCap::Round, ..Stroke::default() };
            if let Some(track) = PathBuilder::from_circle(cx, cy, radius) {
                p.stroke_path(track, stroke.clone(), theme::alpha(t.fg, 0.10));
            }
            let done = *share >= 1.0 && *tone == Tone::Plain;
            if let Some(arc) = arc(cx, cy, radius, *share) {
                let c = if done { t.fg } else { state(t, *tone, theme::calm()) };
                p.stroke_path(arc, stroke, c);
            }
        }
    }

    pub(super) fn paint_spinner(&self, p: &mut Painter, node: &Node<M>, spot: &Spot, t: &Theme) {
        let r = spot.rect;
        if let What::Spinner { size } = &node.what {
            let size = self.px(*size);
            let line = self.px(theme::sheet().spinner_line).max(1.0);
            let (ox, oy) = (r.x.floor(), r.y.floor());
            let at = IntRect::from_xywh(ox as i32, oy as i32, ((r.x + r.w).ceil() - ox).max(1.0) as u32, ((r.y + r.h).ceil() - oy).max(1.0) as u32);
            let Some(at) = at else { return };
            p.push_clip(at, 0.0);
            let (cx, cy) = (r.x - ox + r.w / 2.0, r.y - oy + r.h / 2.0);
            let radius = size / 2.0 - line / 2.0;
            let stroke = Stroke { width: line, ..Stroke::default() };
            if let Some(track) = PathBuilder::from_circle(cx, cy, radius) {
                p.stroke_path(track, stroke.clone(), t.control_edge);
            }
            let turn = if self.moving { self.born.elapsed().as_secs_f32() / SPIN.as_secs_f32() } else { 0.0 };
            let from = -std::f32::consts::FRAC_PI_2 + turn.fract() * std::f32::consts::TAU;
            let mut pb = PathBuilder::new();
            arc_to(&mut pb, (cx, cy), radius, from, from + std::f32::consts::FRAC_PI_2, true);
            if let Some(arc) = pb.finish() {
                p.stroke_path(arc, stroke, t.accent);
            }
            p.pop();
        }
    }

    pub(super) fn paint_drawing<'s>(&'s self, p: &mut Painter<'s>, size: (u32, u32), node: &'s Node<M>, spot: &Spot, t: &Theme) {
        let r = spot.rect;
        match &node.what {
            What::Drawing { seen_only: true, .. } if spot.clip.is_some_and(|c| c.intersect(&r).h <= 0.0 || c.intersect(&r).w <= 0.0) => {}
            What::Drawing { paint: draw, .. } => {
                if node.floats {
                    let falls: Vec<theme::Fall> = t.float.iter().map(|f| self.scaled(f)).collect();
                    p.shadow(r, self.corner(theme::Part::Menu), &falls, t.shadow);
                }
                let (w, h) = (r.w.round() as u32, r.h.round() as u32);
                let id = spot.id.clone();
                let paint = move |c: &mut PixmapMut| {
                    if !node.own_labels {
                        self.fonts.listen();
                    }
                    draw(c, &Pen { theme: &self.look(node), fonts: &self.fonts, icons: &self.icons, scale: self.scale });
                    if !node.own_labels {
                        let heard = self.fonts.heard().into_iter().map(|s| (Rect { x: s.at.0, y: s.at.1, w: s.at.2, h: s.at.3 }, s.whole)).collect();
                        self.said.borrow_mut().insert(id, heard);
                    }
                };
                if r.x == 0.0 && r.y == 0.0 && (w, h) == size {
                    self.on_pixels(p, size, paint);
                } else if let Some(at) = IntRect::from_xywh(r.x.round() as i32, r.y.round() as i32, w.max(1), h.max(1)) {
                    p.push_clip(at, 0.0);
                    let room = self.pixels_at.replace(None);
                    self.on_pixels(p, (at.width(), at.height()), paint);
                    self.pixels_at.set(room);
                    p.pop();
                }
            }
            _ => {}
        }
    }
}
