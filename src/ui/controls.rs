//! Painting buttons, choices, lists and the focus ring.

use super::*;

impl<M: Clone> Ui<M> {
    pub(super) fn paint_round_button(&self, canvas: &mut PixmapMut, node: &Node<M>, spot: &Spot, t: &Theme, lit: bool) {
        let r = spot.rect;
        if let What::Button { icon, press, on, face, size: across, working, tip, .. } = &node.what {
            let armed = self.is_armed(&spot.id);
            let confirm = !armed && node.filled && press.is_some();
            let pale = if t.light { t.picked } else { t.on_picked };
            let on = armed || on.unwrap_or(false) || confirm;
            let icon = icon.as_str();
            let face = face.as_ref().filter(|_| !armed);
            let across = across.unwrap_or(theme::icon_button());
            let size = self.px(across);
            let named = self.labels == Labels::Always && !tip.is_empty();
            let b = self.disc(r, across, named);
            if named {
                let centred = Style { align: crate::text::Align::Centre, ..tip_style(self.scale) };
                let l = self.texts.get(&self.fonts, tip, &centred, Some(self.px(w::NAME_WIDTH)));
                let lw = l.widest();
                let left = l.left();
                let left = if left.is_finite() { left } else { 0.0 };
                self.fonts.draw(canvas, &l, (r.x + (r.w - lw) / 2.0 - left).round(), b.y + size + self.px(w::NAME_GAP), t.fg);
            }
            let used = press.is_some();
            let pointed = lit && used;
            let tinted = node.tint > 0.0 && !(armed || on || pointed);
            let resting = if node.tint > 0.0 { theme::mix(self.control_at(&spot.path), t.accent, node.tint) } else { t.hover };
            let square = if armed { Some(t.accent) } else if confirm { Some(pale) } else if on { Some(t.picked) } else if pointed { Some(t.hover) }
                else if tinted { Some(resting) } else { Some(t.well) };
            if let Some(square) = square {
                let corner = self.corner(if node.tint > 0.0 { theme::Part::RingButton } else { theme::Part::IconButton });
                fill(canvas, b, corner, square);
            }
            if let Some((share, tone)) = working {
                let line = self.px(theme::sheet().working_line);
                let (cx, cy) = (b.x + size / 2.0, b.y + size / 2.0);
                let radius = size / 2.0 - line / 2.0;
                let stroke = Stroke { width: line, line_cap: tiny_skia::LineCap::Round, ..Stroke::default() };
                if let Some(track) = PathBuilder::from_circle(cx, cy, radius) {
                    canvas.stroke_path(&track, &paint(theme::alpha(t.fg, 0.10)), &stroke, Transform::identity(), None);
                }
                if let Some(arc) = arc(cx, cy, radius, *share) {
                    canvas.stroke_path(&arc, &paint(state(t, *tone, theme::calm())), &stroke, Transform::identity(), None);
                }
            }
            let ink = if armed { t.on_accent } else if confirm { theme::on(pale) } else if tinted { theme::on(resting) } else if on { t.on_picked } else if pointed { t.fg } else { t.muted };
            let i = self.px((across * theme::icon() / theme::icon_button()).round());
            let (ix, iy) = ((b.x + (size - i) / 2.0) as i32, (b.y + (size - i) / 2.0) as i32);
            match face {
                Some(f) => f(canvas, Rect { x: ix as f32, y: iy as f32, w: i, h: i }, ink,
                             &Pen { theme: t, fonts: &self.fonts, icons: &self.icons, scale: self.scale }),
                None => { self.icons.draw(canvas, icon, ix, iy, i as u32, 1, ink); }
            }
            let kept = self.glow_at(&spot.id, self.frame.get());
            if kept > 0.0 {
                self.paint_lit(canvas, b, self.corner(theme::Part::IconButton), kept);
            }
        }
    }

    pub(super) fn paint_choice(&self, canvas: &mut PixmapMut, spot: &Spot, options: &[String], chosen: usize, arrows: Option<&[w::Sub]>, lit: bool) {
        let t = &self.theme;
        let r = spot.rect;
        let pad = self.px(theme::sheet().tabs_padding);
        let radius = self.corner(theme::Part::Segmented);
        let boxed = arrows.is_none();
        let depth = self.card_depth(&spot.path);
        if boxed {
            fill(canvas, r, radius, t.card(depth + 1));
        }
        let chosen_back = if boxed { t.card(depth) } else { t.card(depth.max(1)) };
        let inner = r.inset(pad);
        let widths = w::pills_in(w::pill_widths(&self.fonts, &self.texts, options, arrows, self.scale), inner.w);
        let along = self.along.as_ref().filter(|(id, _)| *id == spot.id && self.keyboard_seen && self.focus.as_deref() == Some(spot.id.as_str())).map(|(_, i)| *i);
        let laid = self.spot(&spot.id).map_or(r, |s| s.rect);
        let hovered = self.pointer.filter(|_| lit).and_then(|(x, _)| w::pill_at(&widths, x - laid.x - pad));
        let style = Words::Plain.style(self.scale);
        let tab = (radius - pad).max(0.0);
        let mut x = inner.x;
        for (i, (option, width)) in options.iter().zip(&widths).enumerate() {
            let b = Rect { x, y: inner.y, w: *width, h: inner.h };
            if i == chosen {
                fill(canvas, b, tab, chosen_back);
            } else if hovered == Some(i) {
                fill(canvas, b, tab, theme::alpha(t.fg, 0.06));
            }
            let arrow = arrows.is_some_and(|a| w::has_arrow(a, i));
            let mark = if arrow { self.px(w::ARROW + w::ARROW_GAP).round() } else { 0.0 };
            let edge = self.px(theme::sheet().entry_padding);
            let room = (b.w - 2.0 * edge - mark).max(0.0);
            let words = self.texts.get(&self.fonts, option, &style, None);
            let words = if words.width > room { Rc::new(words.cut(room, &self.texts.get(&self.fonts, "\u{2026}", &style, None))) } else { words };
            self.note_cut(&spot.id, Some(i), option, &words);
            let ink = if i == chosen { t.fg } else { t.muted };
            self.fonts.draw(canvas, &words, (b.x + (b.w - mark - words.widest()) / 2.0).round(),
                            b.y + ((b.h - words.height) / 2.0).round(), ink);
            if arrow {
                let sep = self.texts.get(&self.fonts, "\u{203a}", &style, None);
                let slot = self.px(w::ARROW).round();
                self.fonts.draw(canvas, &sep, (b.x + b.w - edge - slot + (slot - sep.widest()) / 2.0).round(),
                                b.y + ((b.h - sep.height) / 2.0).round(), t.muted);
            }
            if along == Some(i) {
                self.hover_edge(canvas, b, tab);
            }
            x += width;
        }
    }

    pub(super) fn pill(&self, id: &str, x: f32) -> Option<usize> {
        let (spot, What::Choice { options, arrows, .. }) = (self.spot(id)?, &self.node(id)?.what) else { return None };
        let pad = self.px(theme::sheet().tabs_padding);
        let subs = arrows.as_ref().map(|a| a.subs.as_slice());
        w::pill_at(&w::pills_in(w::pill_widths(&self.fonts, &self.texts, options, subs, self.scale), spot.rect.w - 2.0 * pad), x - spot.rect.x - pad)
    }

    pub(super) fn paint_action(&self, canvas: &mut PixmapMut, r: Rect, icon: &str, words: &str, used: bool, lit: bool, for_good: bool) -> Rc<Layout> {
        let t = &self.theme;
        let (b, l, at) = self.action_place(r, words, icon.is_empty());
        let back = if !used {
            t.control
        } else if for_good {
            if lit && t.hover.a < 1.0 { theme::over(t.hover, t.danger) } else { t.danger }
        } else if lit {
            t.accent_hover
        } else {
            t.accent
        };
        fill(canvas, b, self.corner(theme::Part::Button), back);
        let ink = if !used { t.muted } else if for_good { theme::on(back) } else { t.on_accent };
        if !icon.is_empty() {
            let i = self.px(theme::icon());
            let x = at.x - i - self.px(theme::sheet().action_gap);
            self.icons.draw(canvas, icon, x as i32, (b.y + ((b.h - i) / 2.0).round()) as i32, i as u32, 1, ink);
        }
        self.fonts.draw(canvas, &l, at.x, at.y, ink);
        l
    }

    pub(super) fn paint_quiet_action(&self, canvas: &mut PixmapMut, r: Rect, icon: &str, words: &str, used: bool, lit: bool, path: &[usize]) -> Rc<Layout> {
        let t = &self.theme;
        let style = Words::Plain.style(self.scale);
        let padx = self.px(w::QUIET_PADDING);
        let l = self.texts.get(&self.fonts, words, &style, None);
        let h = self.px(theme::sheet().button).min(r.h);
        let b = Rect { y: r.y + ((r.h - h) / 2.0).round(), h, ..r };
        let back = t.card(self.card_depth(path).max(1));
        let back = if lit && used { theme::mix(back, t.fg, 0.06) } else { back };
        fill(canvas, b, self.corner(theme::Part::Button), back);
        let ink = if used { t.fg } else { t.muted };
        let mut x = b.x + padx;
        if !icon.is_empty() {
            let i = self.px(theme::icon());
            self.icons.draw(canvas, icon, x as i32, (b.y + ((b.h - i) / 2.0).round()) as i32, i as u32, 1, ink);
            x += i + self.px(theme::sheet().action_gap);
        }
        let room = (b.x + b.w - padx - x).max(1.0);
        let l = if l.width > room { Rc::new(l.cut(room, &self.texts.get(&self.fonts, "\u{2026}", &style, None))) } else { l.clone() };
        self.fonts.draw(canvas, &l, x.round(), b.y + ((b.h - l.height) / 2.0).round(), ink);
        l
    }

    pub(super) fn action_place(&self, r: Rect, words: &str, bare: bool) -> (Rect, Rc<Layout>, Rect) {
        let spared = if bare { w::action_icon_room(self.scale) } else { 0.0 };
        let l = w::action_words(&self.fonts, &self.texts, words, r.w + spared, self.scale);
        let tall = if l.lines.len() > 1 { l.height.ceil() + 2.0 * w::action_room(self.scale) } else { 0.0 };
        let h = self.px(theme::sheet().action_height).max(tall).min(r.h);
        let b = Rect { y: r.y + ((r.h - h) / 2.0).round(), h, ..r };
        let wide = l.widest().ceil();
        let before = if bare { 0.0 } else { self.px(theme::icon()) + self.px(theme::sheet().action_gap) };
        let x = b.x + self.px(theme::sheet().action_padding).max(((b.w - before - wide) / 2.0).floor());
        (b, l.clone(), Rect { x: (x + before).round(), y: b.y + ((h - l.height) / 2.0).round(), w: wide, h: l.height })
    }

    #[cfg(any(test, feature = "testing"))]
    pub fn worded_buttons(&self) -> Vec<(String, Rect, Rect)> {
        self.spots.iter().filter_map(|s| match &self.tree.at(&s.path)?.what {
            What::Button { words: Some(words), icon, on, .. } if !w::is_word_button(icon, *on) => {
                let (pill, _, at) = self.action_place(s.rect, words, icon.is_empty());
                Some((words.clone(), pill, at))
            }
            _ => None,
        }).collect()
    }

    pub(super) fn paint_word_button(&self, canvas: &mut PixmapMut, r: Rect, words: &str, used: bool, back: Option<Colour>) -> Layout {
        let t = &self.theme;
        let h = self.px(theme::sheet().word_button.1).min(r.h);
        let b = Rect { y: r.y + ((r.h - h) / 2.0).round(), h, ..r };
        fill(canvas, b, self.corner(theme::Part::Button), back.unwrap_or(t.picked));
        let ink = if back.is_none() { t.on_picked } else if used { t.fg } else { t.muted };
        let style = w::word_button_style(self.scale);
        let room = (b.w - 2.0 * self.px(theme::sheet().word_button_padding)).max(1.0);
        let l = self.texts.get(&self.fonts, words, &style, None).cut(room, &self.texts.get(&self.fonts, "\u{2026}", &style, None));
        self.fonts.draw(canvas, &l, (b.x + (b.w - l.width) / 2.0).round(), b.y + ((h - l.height) / 2.0).round(), ink);
        l
    }

    pub(super) fn row_set(&self, spot: &Spot, count: usize, size: (u32, u32)) -> Option<Rows> {
        let r = spot.rect;
        let row = self.row_of(&spot.id);
        let seen = spot.clip.map_or(r, |c| c.intersect(&r)).intersect(&Rect { x: 0.0, y: 0.0, w: size.0 as f32, h: size.1 as f32 });
        if row <= 0.0 || seen.h <= 0.0 {
            return None;
        }
        let first = ((seen.y - r.y) / row).floor().max(0.0) as usize;
        let last = (((seen.y + seen.h - r.y) / row).ceil() as usize).min(count);
        let hovered = self.hover_row().filter(|(id, _)| *id == spot.id && self.gap.as_ref().is_none_or(|(g, _)| *g != spot.id)).map(|(_, i)| i);
        let style = Words::Plain.style(self.scale);
        let dots = self.texts.get(&self.fonts, "\u{2026}", &style, None);
        Some(Rows { row, first, last, hovered, pad: self.px(theme::sheet().entry_padding), icon: self.px(theme::icon()), style, dots })
    }

    pub(super) fn paint_places(&self, p: &mut Painter, size: (u32, u32), spot: &Spot, rows: &[Item], chosen: Option<usize>) {
        let t = &self.theme;
        let r = spot.rect;
        let Some(set) = self.row_set(spot, rows.len(), size) else { return };
        let Rows { row, first, last, hovered, .. } = set;
        let card = t.card(self.card_depth(&spot.path).max(1));
        for (i, item) in rows.iter().enumerate().take(last).skip(first) {
            let b = Rect { x: r.x, y: r.y + i as f32 * row, w: r.w, h: row };
            let here = chosen == Some(i);
            let back = if here { Some(card) } else if hovered == Some(i) { Some(theme::alpha(t.fg, 0.06)) } else { None };
            if let Some(back) = back {
                fill_on(p, b, self.corner(theme::Part::Row), back);
            }
            if !here && hovered == Some(i) {
                self.hover_edge_on(p, b, self.corner(theme::Part::Row));
            }
            let ink = if here { t.fg } else { t.muted };
            let (x, end) = self.row_ends(p, spot, item, b, &set, set.icon, ink, back, t.muted);
            let words = self.texts.get(&self.fonts, &item.text, &set.style, None).cut((end - x).max(0.0), &set.dots);
            self.note_cut(&spot.id, Some(i), &item.text, &words);
            let top = b.y + ((row - words.height) / 2.0).round();
            self.paint_marks_on(p, &words, (x, top), &item.marks, ink);
            self.fonts.paint(p, &words, x, top, ink);
        }
    }

    pub(super) fn row_ends(&self, p: &mut Painter, spot: &Spot, item: &Item, b: Rect, set: &Rows, icon: f32, ink: Colour, back: Option<Colour>, quiet: Colour) -> (f32, f32) {
        let mut x = b.x + set.pad;
        if let Some(name) = &item.icon {
            let ink = self.row_icon_ink(item, ink, back, &spot.path);
            self.icons.draw_on(p, name, x as i32, (b.y + ((set.row - icon) / 2.0).round()) as i32, icon as u32, 1, ink);
            x += icon + self.px(theme::sheet().gap);
        }
        let mut end = b.x + b.w - set.pad;
        if let Some(d) = &item.detail {
            let words = self.texts.get(&self.fonts, d, &set.style, None);
            end -= words.width;
            self.fonts.paint(p, &words, end, b.y + ((set.row - words.height) / 2.0).round(), quiet);
            end -= self.px(theme::sheet().gap);
        }
        (x, end)
    }

    pub(super) fn paint_marks(&self, canvas: &mut PixmapMut, layout: &Layout, at: (f32, f32), marks: &[Range<usize>], ink: Colour) {
        let mut p = Painter::default();
        self.paint_marks_on(&mut p, layout, at, marks, ink);
        p.play(canvas);
    }

    pub(super) fn paint_marks_on(&self, p: &mut Painter, layout: &Layout, at: (f32, f32), marks: &[Range<usize>], ink: Colour) {
        if marks.is_empty() {
            return;
        }
        let bands: Vec<Rect> = layout.mark_bands(marks).into_iter().map(|(x, y, w, h)| Rect { x: at.0 + x, y: at.1 + y, w, h }).collect();
        if bands.is_empty() {
            return;
        }
        let under = self.lies_on();
        let t = &self.theme;
        let wash = t.wash_on(ink, theme::alpha(t.accent, theme::sheet().mark), under);
        for band in bands {
            fill_on(p, band, self.corner(theme::Part::Row).min(band.h / 4.0), wash);
        }
    }

    pub(super) fn paint_list(&self, p: &mut Painter, size: (u32, u32), spot: &Spot, rows: &[Item], chosen: Option<usize>, picked: Option<&[usize]>,
                             tree: Option<&[(usize, Option<bool>)]>) {
        let t = &self.theme;
        let filled = self.tree.at(&spot.path).is_some_and(|n| n.filled);
        let (lit_back, lit_ink) = (t.picked, t.on_picked);
        let r = spot.rect;
        let Some(set) = self.row_set(spot, rows.len(), size) else { return };
        let Rows { row, first, last, hovered, .. } = set;
        let lighter = t.card(self.card_depth(&spot.path).max(1));
        for (i, item) in rows.iter().enumerate().take(last).skip(first) {
            let b = Rect { x: r.x, y: r.y + i as f32 * row, w: r.w, h: row };
            let here = picked.map_or(chosen == Some(i), |p| p.contains(&i));
            let banded = i % 2 == 1 && !filled;
            if banded {
                fill_on(p, b, self.corner(theme::Part::Row), t.band);
            }
            let back = if here {
                Some(lit_back)
            } else if hovered == Some(i) {
                Some(if filled { theme::alpha(t.fg, 0.08) } else { lighter })
            } else {
                None
            };
            if let Some(back) = back {
                fill_on(p, b, self.corner(theme::Part::Row), back);
            }
            if !here && hovered == Some(i) {
                self.hover_edge_on(p, b, self.corner(theme::Part::Row));
            }
            let back = back.or(banded.then_some(t.band));
            let ink = if here { lit_ink } else { t.fg };
            let icon = if item.under.is_some() { self.px(theme::sheet().rich_icon) } else { set.icon };
            let b = match tree.and_then(|t| t.get(i)) {
                Some(&(depth, open)) => self.tree_row_room(p, b, &set, depth, open, ink),
                None => b,
            };
            let quiet = if here { lit_ink } else { t.muted };
            let (x, end) = self.row_ends(p, spot, item, b, &set, icon, ink, back, quiet);
            let room = (end - x).max(0.0);
            self.list_row_words(p, spot, (item, i), &set, (x, b.y), room, (ink, quiet));
        }
    }

    fn tree_row_room(&self, p: &mut Painter, b: Rect, set: &Rows, depth: usize, open: Option<bool>, ink: Colour) -> Rect {
        let (step, arrow) = (self.tree_step(depth), self.px(w::CHEVRON));
        if let Some(open) = open {
            let name = if open { "pan-down-symbolic" } else { "pan-end-symbolic" };
            self.icons.draw_on(p, name, (b.x + set.pad + step).round() as i32, (b.y + ((set.row - arrow) / 2.0).round()) as i32, arrow as u32, 1, ink);
        }
        let by = (step + arrow + self.px(theme::sheet().gap)).min(b.w);
        Rect { x: b.x + by, w: b.w - by, ..b }
    }

    fn list_row_words(&self, p: &mut Painter, spot: &Spot, (item, i): (&Item, usize), set: &Rows, (x, y): (f32, f32), room: f32, (ink, quiet): (Colour, Colour)) {
        let row = set.row;
        match &item.under {
            Some(under) => {
                let bold = Style { weight: theme::sheet().rich_weight, ..set.style.clone() };
                let name = self.texts.get(&self.fonts, &item.text, &bold, None).cut(room, &set.dots);
                let small = Words::Note.style(self.scale);
                let line = self.texts.get(&self.fonts, under, &small, None).cut(room, &set.dots);
                self.note_cut(&spot.id, Some(i), &item.text, &name);
                self.note_cut(&spot.id, Some(i), under, &line);
                let gap = self.px(theme::sheet().rich_gap);
                let top = y + ((row - name.height - gap - line.height) / 2.0).round();
                self.paint_marks_on(p, &name, (x, top), &item.marks, ink);
                self.paint_marks_on(p, &line, (x, top + name.height + gap), &item.under_marks, quiet);
                self.fonts.paint(p, &name, x, top, ink);
                self.fonts.paint(p, &line, x, top + name.height + gap, quiet);
            }
            None => {
                let words = self.texts.get(&self.fonts, &item.text, &set.style, None).cut(room, &set.dots);
                self.note_cut(&spot.id, Some(i), &item.text, &words);
                let top = y + ((row - words.height) / 2.0).round();
                self.paint_marks_on(p, &words, (x, top), &item.marks, ink);
                self.fonts.paint(p, &words, x, top, ink);
            }
        }
    }

    pub(super) fn paint_lit(&self, canvas: &mut PixmapMut, r: Rect, radius: f32, share: f32) {
        let width = self.px(theme::sheet().lit_width).max(1.0);
        let Some(path) = rounded(r.inset(-width / 2.0), radius + width / 2.0) else { return };
        let mut p = paint(theme::alpha(self.theme.accent, share));
        p.anti_alias = true;
        canvas.stroke_path(&path, &p, &Stroke { width, ..Stroke::default() }, Transform::identity(), None);
    }

    pub(super) fn paint_focus(&self, canvas: &mut PixmapMut, spot: &Spot) {
        let Some(node) = self.tree.at(&spot.path) else { return };
        let r = spot.rect;
        let on_eye = self.on_eye(&spot.id);
        let shape = match &node.what {
            What::Button { words: Some(_), .. } => r,
            What::Button { size, tip, .. } => self.disc(r, size.unwrap_or(theme::icon_button()), self.labels == Labels::Always && !tip.is_empty()),
            What::Switch { .. } => {
                let (w, h) = (self.px(theme::sheet().switch.0), self.px(theme::sheet().switch.1));
                Rect { x: r.x, y: r.y + ((r.h - h) / 2.0).round(), w, h }
            }
            What::Tick { .. } => {
                let s = self.px(theme::sheet().tick);
                Rect { x: r.x, y: r.y + ((r.h - s) / 2.0).round(), w: s, h: s }
            }
            What::Slider { value, min, max, .. } => {
                let k = self.px(theme::sheet().knob);
                let part = if max > min { ((value - min) / (max - min)).clamp(0.0, 1.0) } else { 0.0 };
                Rect { x: r.x + part * (r.w - k), y: r.y + ((r.h - k) / 2.0).round(), w: k, h: k }
            }
            What::List { chosen: Some(i), row: h, .. } => {
                let row = self.px(*h);
                Rect { y: r.y + *i as f32 * row, h: row, ..r }
            }
            What::Entry { .. } if on_eye => self.eye_rect(r),
            What::Rich { .. } => self.rich_ring(spot).unwrap_or(r),
            _ => r,
        };
        let width = self.px(theme::sheet().focus_width).max(1.0);
        let inside = matches!(node.what, What::List { chosen: Some(_), .. });
        let out = if inside { -width / 2.0 } else { self.px(theme::sheet().focus_offset) + width / 2.0 };
        let ring = shape.inset(-out);
        let radius = match node.what {
            What::Card(..) if node.flat => self.corner(theme::Part::Row),
            What::Card(..) if node.sunk || self.card_depth(&spot.path) > 0 => self.corner(theme::Part::Box),
            What::Card(..) => self.corner(theme::Part::Card),
            What::Button { words: None, .. } => self.corner(theme::Part::IconButton),
            What::Choice { .. } | What::Dropdown { .. } => self.corner(theme::Part::Box),
            What::Entry { .. } if !on_eye => self.corner(theme::Part::Box),
            What::List { chosen: Some(_), .. } => self.corner(theme::Part::Row),
            What::List { .. } => self.corner(theme::Part::Box),
            What::Drawing { .. } if shape.h > 2.0 * self.px(theme::icon_button()) => self.corner(theme::Part::Box),
            _ => self.corner(theme::Part::Button),
        } + out;
        if let Some(path) = rounded(ring, radius) {
            let mut p = paint(self.theme.lit_edge);
            p.anti_alias = true;
            canvas.stroke_path(&path, &p, &Stroke { width, ..Stroke::default() }, Transform::identity(), None);
        }
    }

    pub(super) fn hover_edge(&self, canvas: &mut PixmapMut, r: Rect, radius: f32) {
        let mut p = Painter::default();
        self.hover_edge_on(&mut p, r, radius);
        p.play(canvas);
    }

    pub(super) fn hover_edge_on(&self, p: &mut Painter, r: Rect, radius: f32) {
        let width = self.px(theme::sheet().hover_edge).max(1.0);
        if let Some(path) = rounded(r.inset(width / 2.0), (radius - width / 2.0).max(0.0)) {
            p.stroke_path(path, Stroke { width, ..Stroke::default() }, self.theme.lit_edge);
        }
    }

    pub(super) fn current_edge(&self, canvas: &mut PixmapMut, r: Rect, radius: f32) {
        let width = (theme::sheet().current_edge * self.scale).round().max(1.0);
        if let Some(path) = rounded(r.inset(width / 2.0), (radius - width / 2.0).max(0.0)) {
            canvas.stroke_path(&path, &paint(self.theme.current_edge), &Stroke { width, ..Stroke::default() }, Transform::identity(), None);
        }
    }

    pub(super) fn disc(&self, r: Rect, across: f32, named: bool) -> Rect {
        let s = self.px(across);
        let y = if named { r.y } else { r.y + ((r.h - s) / 2.0).round() };
        Rect { x: r.x + ((r.w - s) / 2.0).round(), y, w: s, h: s }
    }
}
