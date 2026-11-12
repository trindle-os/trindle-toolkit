//! Painting a frame.

use tiny_skia::IntRect;

use super::*;
use crate::paint::{cpu, Draws, Painter};
use crate::window::dmabuf::{Drawn, Frame};

impl<M: Clone> Ui<M> {
    pub fn paint(&mut self, canvas: &mut PixmapMut, scale: f32) {
        let began = self.ready((canvas.width(), canvas.height()), scale);
        self.dev.gpu.set(false);
        self.paint_widgets(canvas);
        self.lay_before_over(canvas);
        *self.sent_out.borrow_mut() = None;
        if !self.paint_refused(canvas) {
            self.paint_tip(canvas);
        }
        self.paint_drop(canvas);
        self.paint_copy_menu(canvas);
        self.paint_bell(canvas);
        if self.shown < 1.0 {
            let k = self.shown.clamp(0.0, 1.0);
            for b in canvas.data_mut() {
                *b = (f32::from(*b) * k).round() as u8;
            }
        }
        self.painted_in(began.elapsed(), 1.0);
        self.paint_dev(canvas);
    }

    pub(crate) fn paint_by_gpu(&mut self, frame: &Frame, drawing: &mut dyn Draws) -> Drawn {
        let began = self.ready(frame.size, frame.scale);
        self.dev.gpu.set(true);
        let inspected = self.inspected();
        let drawn = {
            let mut p = Painter::default();
            self.record(&mut p, inspected);
            drawing.draw(frame, p, &self.shadows)
        };
        let share = match &drawn {
            Drawn::Parts(parts) => parts.iter().map(|p| (p[2] * p[3]) as f32).sum::<f32>() / (frame.size.0 * frame.size.1).max(1) as f32,
            _ => 1.0,
        };
        self.painted_in(began.elapsed(), share);
        drawn
    }

    #[cfg(test)]
    pub(crate) fn recorded(&mut self, size: (u32, u32), scale: f32) -> Painter<'_> {
        self.ready(size, scale);
        self.dev.gpu.set(true);
        let inspected = self.inspected();
        let mut p = Painter::default();
        self.record(&mut p, inspected);
        p
    }

    pub(super) fn record<'s>(&'s self, p: &mut Painter<'s>, inspected: Option<(Rect, Vec<String>)>) {
        let size = self.size;
        let Some(whole) = IntRect::from_xywh(0, 0, size.0, size.1) else { return };
        let fade = self.shown < 1.0;
        if fade {
            p.push_layer(whole, self.shown.clamp(0.0, 1.0));
        }
        let blend = self.before.as_ref().zip(self.blend).filter(|((_, b), _)| (b.width(), b.height()) == size);
        if let Some(((id, before), (_, share))) = blend {
            if self.see_through {
                p.pixels(whole, move |c| c.data_mut().copy_from_slice(before.data()));
            } else {
                p.image(*id, before.clone(), 0, 0);
            }
            p.push_layer(whole, share.clamp(0.0, 1.0));
        }
        let marking = !fade && blend.is_none();
        if marking {
            p.mark(0, Some(whole));
        }
        if !self.see_through {
            p.fill(Rect { x: 0.0, y: 0.0, w: size.0 as f32, h: size.1 as f32 }, self.theme.bg);
        }
        self.marking.set(marking);
        self.paint_range(p, size, 0, self.spots.len(), 0.0, 0.0);
        if blend.is_some() {
            p.pop();
        }
        *self.sent_out.borrow_mut() = None;
        if marking {
            p.mark(1, None);
        }
        let room = (size.0 as f32, size.1 as f32);
        if !self.refused_on(p, room) {
            self.tip_on(p, room);
        }
        self.drop_on(p);
        self.copy_menu_on(p);
        self.bell_on(p, room);
        if fade {
            p.pop();
        }
        if marking {
            p.mark(2, None);
        }
        if inspected.is_some() || self.dev.speed {
            p.pixels(whole, move |c| self.paint_dev_with(c, inspected));
        }
    }

    pub(super) fn ready(&mut self, size: (u32, u32), scale: f32) -> Instant {
        let began = Instant::now();
        self.frame.set(began);
        self.texts.frame();
        self.icons.set_ink(self.theme.fg, self.theme.icon);
        self.size = size;
        self.scale = scale;
        self.settle();
        let entries: Vec<String> = self.spots.iter()
            .filter(|s| self.is_entry(&s.id) && (self.focus.as_deref() == Some(&s.id) || self.edits.contains_key(&s.id)))
            .map(|s| s.id.clone()).collect();
        for id in entries {
            self.keep_caret_in_view(&id);
        }
        self.cuts.borrow_mut().clear();
        self.said.borrow_mut().clear();
        self.touched.borrow_mut().clear();
        #[cfg(feature = "testing")]
        self.audit_what_is_painted();
        if self.blend.is_some_and(|(_, share)| share >= 1.0) {
            self.before = None;
        }
        began
    }

    pub fn paint_live(&self, canvas: &mut PixmapMut, scale: f32) -> Option<Vec<Rect>> {
        let began = Instant::now();
        self.frame.set(began);
        self.texts.frame();
        if self.laid != Some((canvas.width(), canvas.height(), scale, self.fonts.generation())) || self.shown < 1.0 || self.before.is_some() || self.inspecting()
            || self.drop.is_some() || self.copy_menu.is_some() || self.bell.is_some()
            || self.tip.as_ref().is_some_and(|t| matches!(t.state, TipState::Shown(_))) {
            return None;
        }
        let mut done = Vec::new();
        for spot in &self.spots {
            let Some(node) = self.tree.at(&spot.path) else { continue };
            match &node.what {
                What::Drawing { live: Some(frames), .. } => {
                    if spot.clip.is_some() || node.opacity < 1.0 {
                        return None;
                    }
                    w::put_frame(canvas, spot.rect, frames, true);
                    done.push(spot.rect);
                }
                What::Spinner { .. } | What::Drawing { alive: true, .. } => {
                    let whole = Rect { x: 0.0, y: 0.0, w: canvas.width() as f32, h: canvas.height() as f32 };
                    let seen = spot.clip.map_or(spot.rect, |c| c.intersect(&spot.rect)).intersect(&whole);
                    let (x, y) = (seen.x.floor(), seen.y.floor());
                    let part = Rect { x, y, w: (seen.x + seen.w).ceil() - x, h: (seen.y + seen.h).ceil() - y };
                    if part.w < 1.0 || part.h < 1.0 {
                        continue;
                    }
                    let painted = self.paint_part(part)?;
                    let place = [part.x as i32, part.y as i32, painted.width() as i32, painted.height() as i32];
                    crate::common::shm::put(canvas.data_mut(), (whole.w as i32, whole.h as i32), painted.data(), place);
                    done.push(part);
                }
                _ => {}
            }
        }
        for part in self.touched.take() {
            let painted = self.paint_part(part)?;
            let place = [part.x as i32, part.y as i32, painted.width() as i32, painted.height() as i32];
            let size = (canvas.width() as i32, canvas.height() as i32);
            crate::common::shm::put(canvas.data_mut(), size, painted.data(), place);
            done.push(part);
        }
        let area: f32 = done.iter().map(|r| r.w * r.h).sum();
        self.painted_in(began.elapsed(), area / (canvas.width() * canvas.height()).max(1) as f32);
        done.extend(self.paint_speed(canvas));
        Some(done)
    }

    pub(crate) fn touch(&self, id: &str) -> bool {
        let Some(spot) = self.spot(id) else { return false };
        let room = self.px(PART_ROOM);
        let seen = spot.clip.map_or(spot.rect, |c| c.intersect(&spot.rect));
        let whole = Rect { x: 0.0, y: 0.0, w: self.size.0 as f32, h: self.size.1 as f32 };
        let near = Rect { x: seen.x - room, y: seen.y - room, w: seen.w + 2.0 * room, h: seen.h + 2.0 * room }.intersect(&whole);
        let (x, y) = (near.x.floor(), near.y.floor());
        let part = Rect { x, y, w: (near.x + near.w).ceil() - x, h: (near.y + near.h).ceil() - y };
        if part.w >= 1.0 && part.h >= 1.0 {
            self.touched.borrow_mut().push(part);
        }
        true
    }

    pub(super) fn paint_part(&self, part: Rect) -> Option<Pixmap> {
        let mut pix = Pixmap::new(part.w as u32, part.h as u32)?;
        pix.fill(if self.see_through { tiny_skia::Color::TRANSPARENT } else { self.theme.bg.to_skia() });
        self.part.set(true);
        self.paint_on(&mut pix.as_mut(), -part.x, -part.y);
        self.part.set(false);
        Some(pix)
    }

    pub(super) fn near(&self, size: (u32, u32), spot: &Spot, dx: f32, dy: f32) -> bool {
        let room = self.px(PART_ROOM);
        let r = spot.clip.map_or(spot.rect, |c| c.intersect(&spot.rect));
        r.x + dx - room < size.0 as f32 && r.x + dx + r.w + room > 0.0
            && r.y + dy - room < size.1 as f32 && r.y + dy + r.h + room > 0.0
    }

    pub(super) fn paint_widgets(&self, canvas: &mut PixmapMut) {
        canvas.fill(if self.see_through { tiny_skia::Color::TRANSPARENT } else { self.theme.bg.to_skia() });
        self.paint_on(canvas, 0.0, 0.0);
    }

    fn paint_on(&self, canvas: &mut PixmapMut, dx: f32, dy: f32) {
        let mut p = Painter::default();
        self.paint_range(&mut p, (canvas.width(), canvas.height()), 0, self.spots.len(), dx, dy);
        cpu::play(p.ops, canvas, &self.shadows);
    }

    pub(super) fn on_pixels<'s>(&'s self, p: &mut Painter<'s>, size: (u32, u32), draw: impl FnOnce(&mut PixmapMut) + 's) {
        let Some(at) = self.pixels_at.get().or_else(|| IntRect::from_xywh(0, 0, size.0, size.1)) else { return };
        let under = self.lies_on();
        p.pixels(at, move |c| {
            self.under.borrow_mut().push((usize::MAX, under));
            draw(c);
            self.under.borrow_mut().pop();
        });
    }

    pub(super) fn lay_before_over(&self, canvas: &mut PixmapMut) {
        let (Some((_, before)), Some((_, share))) = (&self.before, self.blend) else { return };
        if (before.width(), before.height()) != (canvas.width(), canvas.height()) {
            return;
        }
        let k = (share.clamp(0.0, 1.0) * 255.0).round() as u32;
        for (now, was) in canvas.data_mut().iter_mut().zip(before.data()) {
            *now = ((u32::from(*now) * k + u32::from(*was) * (255 - k) + 127) / 255) as u8;
        }
    }

    pub(crate) fn blend_to(&mut self, cue: Option<(u64, f32)>) {
        let turned = matches!((self.blend, cue), (Some((was, _)), Some((now, _))) if was != now);
        if turned && self.size.0 > 0 && self.size.1 > 0 {
            if let Some(mut was) = Pixmap::new(self.size.0, self.size.1) {
                self.settle();
                self.paint_widgets(&mut was.as_mut());
                self.lay_before_over(&mut was.as_mut());
                self.before = Some((crate::paint::new_id(), Rc::new(was)));
            }
        }
        if cue.is_none() {
            self.before = None;
        }
        self.blend = cue;
    }

    pub(super) fn paint_range<'s>(&'s self, p: &mut Painter<'s>, size: (u32, u32), from: usize, to: usize, dx: f32, dy: f32) {
        let marking = self.marking.replace(false);
        let mut i = from;
        while i < to {
            self.enter(i);
            let spot = &self.spots[i];
            if marking {
                let mut key = std::hash::DefaultHasher::new();
                std::hash::Hash::hash(&spot.path, &mut key);
                p.mark(std::hash::Hasher::finish(&key), self.room_of(&moved(spot, dx, dy)));
            }
            let opacity = self.tree.at(&spot.path).map_or(1.0, |n| n.opacity);
            if opacity < 1.0 {
                let end = (i + 1..to).find(|&j| !self.spots[j].path.starts_with(&spot.path)).unwrap_or(to);
                if opacity > 0.0 {
                    self.paint_faded(p, size, i, end, dx, dy, opacity);
                }
                i = end;
                continue;
            }
            let far = !self.near(size, spot, dx, dy)
                && (self.part.get() || !matches!(self.tree.at(&spot.path).map(|n| &n.what), Some(What::Drawing { seen_only: false, .. })));
            let end = if self.is_scroll(spot) || self.is_scroll_x(spot) || self.is_slide(spot) {
                let end = (i + 1..to).find(|&j| !self.spots[j].path.starts_with(&spot.path)).unwrap_or(to);
                if !far {
                    self.paint_scroll(p, size, i, end, dx, dy);
                }
                end
            } else {
                if let Some(node) = self.tree.at(&spot.path).filter(|_| !far) {
                    self.one(p, size, node, &moved(spot, dx, dy));
                }
                i + 1
            };
            if self.keyboard_seen && self.focus.as_deref() == Some(&spot.id) {
                self.focus_ring(p, size, moved(spot, dx, dy));
            }
            i = end;
        }
    }

    pub(super) fn paint_faded<'s>(&'s self, p: &mut Painter<'s>, size: (u32, u32), i: usize, end: usize, dx: f32, dy: f32, opacity: f32) {
        let s = &self.spots[i];
        let seen = s.clip.map_or(s.rect, |c| c.intersect(&s.rect));
        let room = self.px(FADED_ROOM);
        let (cw, ch) = (size.0 as f32, size.1 as f32);
        let (ox, oy) = ((seen.x + dx - room).floor().max(0.0), (seen.y + dy - room).floor().max(0.0));
        let (w, h) = (((seen.x + dx + seen.w + room).ceil().min(cw) - ox), ((seen.y + dy + seen.h + room).ceil().min(ch) - oy));
        if w <= 0.0 || h <= 0.0 {
            return;
        }
        let Some(at) = IntRect::from_xywh(ox as i32, oy as i32, w as u32, h as u32) else { return };
        p.push_layer(at, opacity);
        let size = (at.width(), at.height());
        let (dx, dy) = (dx - ox, dy - oy);
        if self.is_scroll(s) || self.is_scroll_x(s) || self.is_slide(s) {
            self.paint_scroll(p, size, i, end, dx, dy);
        } else {
            if let Some(node) = self.tree.at(&s.path) {
                self.one(p, size, node, &moved(s, dx, dy));
            }
            self.paint_range(p, size, i + 1, end, dx, dy);
        }
        if self.keyboard_seen && self.focus.as_deref() == Some(&s.id) {
            self.focus_ring(p, size, moved(s, dx, dy));
        }
        p.pop();
    }

    fn one<'s>(&'s self, p: &mut Painter<'s>, size: (u32, u32), node: &'s Node<M>, spot: &Spot) {
        self.pixels_at.set(self.room_of(spot));
        self.paint_one(p, size, node, spot);
        self.pixels_at.set(None);
    }

    fn focus_ring<'s>(&'s self, p: &mut Painter<'s>, size: (u32, u32), spot: Spot) {
        self.pixels_at.set(self.room_of(&spot));
        self.on_pixels(p, size, move |c| self.paint_focus(c, &spot));
        self.pixels_at.set(None);
    }

    fn room_of(&self, spot: &Spot) -> Option<IntRect> {
        let room = self.px(PART_ROOM);
        let r = spot.clip.map_or(spot.rect, |c| c.intersect(&spot.rect));
        IntRect::from_ltrb((r.x - room).floor() as i32, (r.y - room).floor() as i32, (r.x + r.w + room).ceil() as i32, (r.y + r.h + room).ceil() as i32)
    }

    fn enter(&self, i: usize) {
        let spot = &self.spots[i];
        let mut known = self.under.borrow_mut();
        while known.last().is_some_and(|&(j, _)| j >= i || !spot.path.starts_with(&self.spots[j].path)) {
            known.pop();
        }
        let over = spot.path.split_last().is_some_and(|(&k, up)| k > 0 && matches!(self.tree.at(up).map(|n| &n.what), Some(What::Layers(_))));
        let ground = match known.last() {
            _ if over => None,
            Some(&(_, c)) => c,
            None => if self.see_through { None } else { laid(self.theme.bg, None) },
        };
        let inside = self.tree.at(&spot.path).map_or(ground, |node| match self.fills(node, spot) {
            Some(c) => laid(c, ground),
            None => ground,
        });
        known.push((i, inside));
    }

    fn fills(&self, node: &Node<M>, spot: &Spot) -> Option<Colour> {
        let What::Card(_, press) = &node.what else { return None };
        let faded;
        let t = if node.off { faded = self.theme.faded(); &faded } else { &self.theme };
        let lit = !node.off && press.is_some() && (self.hover.as_deref() == Some(&spot.id) || self.pressed.as_deref() == Some(&spot.id));
        if node.band {
            Some(t.band)
        } else if node.floats {
            Some(t.raised)
        } else if node.chip {
            Some(if lit { t.picked } else { self.control_at(&spot.path) })
        } else if node.sunk {
            Some(t.field)
        } else if node.flat {
            lit.then_some(t.hover)
        } else if node.side {
            let (which, _) = node.area?;
            let surface = match which {
                Area::Top | Area::Controls => t.top_bar,
                Area::Side | Area::Between => t.side_bar,
                Area::Foot => t.foot,
            };
            (surface.a > 0.0).then_some(surface)
        } else {
            let colour = t.card(self.card_depth(&spot.path));
            Some(if lit { theme::mix(colour, t.fg, 0.06) } else { colour })
        }
    }

    pub(super) fn lies_on(&self) -> Option<Colour> {
        self.under.borrow().last().and_then(|&(_, c)| c)
    }

    pub(super) fn paint_scroll<'s>(&'s self, p: &mut Painter<'s>, size: (u32, u32), i: usize, end: usize, dx: f32, dy: f32) {
        let s = &self.spots[i];
        let mut seen = s.clip.map_or(s.rect, |c| c.intersect(&s.rect));
        if self.part.get() {
            let on = Rect { x: -dx, y: -dy, w: size.0 as f32, h: size.1 as f32 };
            seen = seen.intersect(&on);
        }
        let (ox, oy) = ((seen.x + dx).floor(), (seen.y + dy).floor());
        let (w, h) = ((seen.x + dx + seen.w - ox).ceil(), (seen.y + dy + seen.h - oy).ceil());
        let Some(at) = IntRect::from_xywh(ox as i32, oy as i32, w.max(0.0) as u32, h.max(0.0) as u32) else { return };
        p.push_clip(at, 0.0);
        self.paint_range(p, (at.width(), at.height()), i + 1, end, dx - ox, dy - oy);
        let lit = self.bar_hover.as_deref() == Some(&s.id) || self.thumb.as_ref().is_some_and(|(a, _)| *a == s.id);
        let thick = self.px(if lit { theme::sheet().scrollbar_hover } else { theme::sheet().scrollbar });
        if let Some((track, thumb)) = self.bar(s, thick) {
            let t = &self.theme;
            let at = |r: Rect| Rect { x: r.x + dx - ox, y: r.y + dy - oy, ..r };
            let corner = [self.corner(theme::Part::Scrollbar); 4];
            if lit {
                p.round(at(track), corner, theme::alpha(t.muted, 0.15));
            }
            p.round(at(thumb), corner, theme::alpha(t.muted, if lit { theme::sheet().scrollbar_lit } else { theme::sheet().scrollbar_rest }));
        }
        p.pop();
    }

    pub(super) fn px(&self, points: f32) -> f32 {
        (points * self.scale).round()
    }

    pub(super) fn corner(&self, part: theme::Part) -> f32 {
        self.px(theme::sheet().corner_of(part))
    }

    pub(super) fn keep_caret_in_view(&mut self, id: &str) {
        let (Some(spot), Some(What::Entry { secret, .. })) = (self.spot(id), self.node(id).map(|n| &n.what)) else { return };
        let w = spot.rect.w - if *secret { self.eye_room(spot.rect) } else { 0.0 };
        let secret = self.entry_dots(id);
        let inner = w - 2.0 * self.entry_pad(id);
        let caret = self.px(theme::sheet().caret).max(1.0);
        if self.edit(id).is_none() {
            return;
        }
        let Some(e) = self.edits.get(id) else { return };
        let (text, at) = match self.preedit_in(id) {
            Some((p, c)) => (std::borrow::Cow::Owned(with_preedit(&e.text, e.caret, p)), e.caret + c.map_or(p.len(), |c| c.1)),
            None => (std::borrow::Cow::Borrowed(e.text.as_str()), e.caret),
        };
        let layout = self.entry_layout(id, &text, secret);
        let (x, top, line) = layout.caret(if secret { edit::to_dots(&text, at) } else { at });
        let len = text.len();
        let many = self.entry_lines(id) > 1;
        let tall = if many { self.entry_tall(id) } else { 0.0 };
        let Some(e) = self.edits.get_mut(id) else { return };
        if many {
            let key = (at, len);
            let slid = if e.viewed.replace(key) == Some(key) { e.scroll_y } else { e.scroll_y.max(top + line - tall).min(top) };
            e.scroll_y = slid.clamp(0.0, (layout.height - tall).max(0.0));
            return;
        }
        let room = (inner - caret).max(0.0);
        let mut scroll = e.scroll;
        if x - scroll > room {
            scroll = x - room;
        }
        if x < scroll {
            scroll = x;
        }
        e.scroll = scroll.clamp(0.0, (layout.width + caret - inner).max(0.0));
    }
}

fn laid(top: Colour, ground: Option<Colour>) -> Option<Colour> {
    let mut pix = Pixmap::new(1, 1)?;
    if let Some(g) = ground {
        pix.fill(g.to_skia());
    }
    fill(&mut pix.as_mut(), Rect { x: 0.0, y: 0.0, w: 1.0, h: 1.0 }, 0.0, top);
    let p = pix.pixel(0, 0)?.demultiply();
    (p.alpha() == 255).then(|| Colour::rgb(f32::from(p.red()) / 255.0, f32::from(p.green()) / 255.0, f32::from(p.blue()) / 255.0))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::widget as ww;

    #[test]
    fn the_colour_known_under_words_is_the_one_painted_there() {
        let kinds: [fn(Node<()>) -> Node<()>; 12] = [
            ww::card, |n| ww::card(ww::card(n)), |n| ww::card(n).chip(), |n| ww::card(n).sunk(), |n| ww::card(n).floats(),
            |n| ww::card(n).banded(), |n| ww::card(n).side(), |n| ww::card(n).side().area(Area::Top, [0.0; 4]),
            |n| ww::card(n).side().area(Area::Foot, [0.0; 4]), |n| ww::card(n).off(true), |n| ww::card(ww::scroll("s", n)), |n| ww::scroll("s", n),
        ];
        for (light, see_through) in [(false, false), (true, false), (false, true)] {
            for kind in kinds {
                let mut ui: Ui<()> = Ui::new(ww::column(vec![kind(ww::label(" ").key("w"))]).padding(10.0), Theme::new(light, "#fbc711"));
                ui.see_through = see_through;
                let mut pix = Pixmap::new(200, 100).unwrap();
                ui.paint(&mut pix.as_mut(), 1.0);
                let i = ui.spots.iter().position(|s| s.id == "w").unwrap();
                (0..=i).for_each(|j| ui.enter(j));
                let r = ui.spots[i].rect;
                let p = pix.pixel((r.x + r.w / 2.0) as u32, (r.y + r.h / 2.0) as u32).unwrap();
                let known = ui.lies_on().map(|c| c.to_skia().to_color_u8());
                assert_eq!(known, (p.alpha() == 255).then(|| p.demultiply()), "light {light}, see-through {see_through}, {:?}", ui.spots[i].path);
            }
        }
    }
}
