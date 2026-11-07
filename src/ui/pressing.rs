//! Pointer moves, presses and releases.

use super::*;

impl<M: Clone> Ui<M> {
    pub(super) fn pointer_to(&mut self, x: f32, y: f32, now: Instant, reply: &mut Reply<M>) {
        let before = self.pointer.replace((x, y));
        if let Some((area, grab)) = self.thumb.clone() {
            self.thumb_drag(&area, grab, (x, y), reply);
            return;
        }
        if self.lift_moved(x, y, reply) || self.band_to(x, y, reply) || self.row_drag_to(x, y, reply) || self.right_to(x, y, reply) || self.held_to(x, y, reply) {
            return;
        }
        self.hover_to(x, y, before, now, reply);
    }

    fn thumb_drag(&mut self, area: &str, grab: f32, (x, y): (f32, f32), reply: &mut Reply<M>) {
        let thick = self.px(theme::sheet().scrollbar_hover);
        let Some(s) = self.spot(area) else { return };
        let Some((track, thumb)) = self.bar(s, thick) else { return };
        let (room, at) = if self.is_scroll_x(s) { (track.w - thumb.w, x - grab - track.x) } else { (track.h - thumb.h, y - grab - track.y) };
        let t = if room > 0.0 { (at / room).clamp(0.0, 1.0) } else { 0.0 };
        let to = t * self.span(s);
        reply.redraw |= self.scroll_area(area, to);
    }

    fn row_drag_to(&mut self, x: f32, y: f32, reply: &mut Reply<M>) -> bool {
        if let Some((id, row, from, moved)) = self.row_drag.clone() {
            let across = self.spot(&id).is_some_and(|s| x < s.rect.x || x > s.rect.x + s.rect.w);
            if moved || across || (y - from).abs() > self.px(RIGHT_SLOP) {
                self.row_drag = Some((id.clone(), row, from, true));
                let to = self.row_drop_list(&id, x, y);
                let gap = to.and_then(|to| self.gap_at(&to, y, true).map(|g| (to, g)));
                self.gap = gap;
                self.tip = None;
                reply.redraw = true;
                return true;
            }
        }
        false
    }

    fn row_drop_list(&self, id: &str, x: f32, y: f32) -> Option<String> {
        let other = self.hit(x, y).filter(|o| o != id && matches!(self.node(o).map(|n| &n.what), Some(What::List { move_in: Some(_), .. })));
        other.or_else(|| matches!(self.node(id).map(|n| &n.what), Some(What::List { reorder: Some(_), .. })).then(|| id.to_string()))
    }

    fn right_to(&mut self, x: f32, y: f32, reply: &mut Reply<M>) -> bool {
        let Some((id, from, dragged)) = self.right.clone() else { return false };
        let slop = self.px(RIGHT_SLOP);
        if !dragged && (x - from.0).abs() <= slop && (y - from.1).abs() <= slop {
            return true;
        }
        if !dragged {
            self.right = Some((id.clone(), from, true));
            self.point(&id, from.0, from.1, w::Pointer::RightDown, reply);
        }
        self.point(&id, x, y, w::Pointer::RightMove, reply);
        self.held_over(&id, x, y, reply);
        true
    }

    fn held_to(&mut self, x: f32, y: f32, reply: &mut Reply<M>) -> bool {
        let Some(id) = self.dragging.clone() else { return false };
        if self.is_pointed(&id) {
            self.point(&id, x, y, w::Pointer::Move, reply);
            self.held_over(&id, x, y, reply);
        } else if self.picked.as_ref().is_some_and(|(p, _)| *p == id) {
            let at = self.pick_hit(&id, x, y);
            if let Some((_, e)) = &mut self.picked {
                if e.caret != at {
                    e.move_to(at, true);
                    reply.redraw = true;
                }
            }
        } else if self.is_entry(&id) {
            let at = self.entry_hit(&id, x);
            if let Some(e) = self.edit(&id) {
                if e.caret != at {
                    e.move_to(at, true);
                    reply.redraw = true;
                }
            }
        } else if self.is_divider(&id) {
            self.divider_to(&id, (x, y), reply);
        } else {
            self.slide_to(&id, x, reply);
        }
        true
    }

    fn hover_to(&mut self, x: f32, y: f32, before: Option<(f32, f32)>, now: Instant, reply: &mut Reply<M>) {
        let (was, name_shown) = (self.hover.clone(), matches!(self.tip.as_ref().map(|t| &t.state), Some(TipState::Shown(_))));
        self.hover_to_whole(x, y, before, now, reply);
        if self.hover != was && !reply.redraw && !name_shown {
            let both = [was.as_deref(), self.hover.as_deref()];
            reply.patch = both.iter().flatten().all(|id| self.touch(id));
        }
        reply.redraw |= self.hover != was && !reply.patch;
    }

    fn hover_to_whole(&mut self, x: f32, y: f32, before: Option<(f32, f32)>, now: Instant, reply: &mut Reply<M>) {
        let over = if self.touch.is_none() { self.drawing_at(x, y).filter(|id| self.hovers(id)) } else { None };
        self.hover_drawing(over.clone(), reply);
        if let Some(id) = over {
            self.point(&id, x, y, w::Pointer::Hover, reply);
        }
        let bar = if self.touch.is_none() { self.bar_at(x, y) } else { None };
        if bar != self.bar_hover {
            self.bar_hover = bar.clone();
            reply.redraw = true;
        }
        let row = self.hover_row();
        let hit = if bar.is_some() { None } else { self.hit(x, y) };
        if hit != self.hover {
            self.hover = hit.clone();
            self.tip = hit.map(|id| Tip { id, state: TipState::Waiting(now) });
        } else if let Some(t) = &mut self.tip {
            if let TipState::Waiting(_) = t.state {
                t.state = TipState::Waiting(now);
            }
        }
        if self.hover_row() != row {
            reply.redraw = true;
        }
        let part = self.hover.clone().and_then(|id| self.part_at(&id));
        if part != self.tip_part {
            self.tip_part = part;
            if let Some(t) = &mut self.tip {
                reply.redraw |= matches!(t.state, TipState::Shown(_));
                t.state = TipState::Waiting(now);
            }
        }
        let eye = if self.touch.is_none() && bar.is_none() { self.eye_at(x, y) } else { None };
        if eye != self.eyes.over {
            self.eyes.over = eye;
            self.tip = self.hover.clone().map(|id| Tip { id, state: TipState::Waiting(now) });
            reply.redraw = true;
        }
        let tabs = self.hover.clone();
        let tabs = tabs.as_deref().filter(|id| matches!(self.node(id).map(|n| &n.what), Some(What::Choice { .. })));
        if let Some(id) = tabs {
            if before.map(|(bx, _)| self.pill(id, bx)) != Some(self.pill(id, x)) {
                reply.redraw = true;
            }
        }
    }

    pub(super) fn press(&mut self, x: f32, y: f32, now: Instant, reply: &mut Reply<M>) {
        reply.redraw = true;
        let (focus, keys, name_shown) = (self.focus.clone(), self.keyboard_seen, matches!(self.tip.as_ref().map(|t| &t.state), Some(TipState::Shown(_))));
        self.keyboard_seen = false;
        self.tip = self.tip.take().map(|t| Tip { state: TipState::Gone, ..t });
        if let Some(area) = self.bar_at(x, y) {
            self.press_bar(area, x, y);
            return;
        }
        if self.pick_press(x, y, now) {
            return;
        }
        if let Some(id) = self.drawing_at(x, y).filter(|id| self.keys_of(id).is_some()) {
            self.focus = Some(id.clone());
            self.press_drawing(id, x, y, now, reply);
            return;
        }
        match self.hit(x, y).filter(|id| self.node(id).is_some_and(Node::focusable)) {
            Some(id) => {
                let button = matches!(self.node(&id).map(|n| &n.what), Some(What::Button { .. }));
                self.press_widget(id.clone(), x, y, now, reply);
                if button && !name_shown && self.eyes.down.is_none() && self.dragging.is_none() && self.row_drag.is_none() {
                    let old = focus.filter(|_| keys);
                    reply.patch = [Some(id), old].iter().flatten().all(|id| self.touch(id));
                    reply.redraw = !reply.patch;
                }
            }
            None => self.press_blank(x, y, now, reply),
        }
    }

    fn press_bar(&mut self, area: String, x: f32, y: f32) {
        let thick = self.px(theme::sheet().scrollbar_hover);
        let Some(s) = self.spot(&area) else { return };
        let Some((_, thumb)) = self.bar(s, thick) else { return };
        let (at, start, len, view) = if self.is_scroll_x(s) { (x, thumb.x, thumb.w, s.rect.w) } else { (y, thumb.y, thumb.h, s.rect.h) };
        if at >= start && at < start + len {
            self.thumb = Some((area, at - start));
        } else {
            let page = self.page(view) * if at < start { -1.0 } else { 1.0 };
            let to = self.offset(s) + page;
            self.scroll_area(&area, to);
        }
    }

    fn press_drawing(&mut self, id: String, x: f32, y: f32, now: Instant, reply: &mut Reply<M>) {
        if let Some(Node { what: What::Drawing { twice: Some(m), .. }, .. }) = self.node(&id) {
            let near = |(at, last, (px, py)): &(Instant, String, (f32, f32))| *last == id && now.saturating_duration_since(*at) < crate::host::host().double_click()
                && (x - px).hypot(y - py) <= self.px(TWICE_SLOP);
            if self.last_down.as_ref().is_some_and(near) {
                reply.messages.push(m.clone());
                self.last_down = None;
                return;
            }
            self.last_down = Some((now, id.clone(), (x, y)));
        }
        let before = reply.messages.len();
        self.point(&id, x, y, w::Pointer::Down, reply);
        let quiet = reply.messages.len() == before && matches!(self.node(&id).map(|n| &n.what), Some(What::Drawing { pointer: Some(_), .. }));
        if quiet && self.drawing_menu(&id, x, y, false) {
            return;
        }
        self.dragging = Some(id);
    }

    fn press_blank(&mut self, x: f32, y: f32, now: Instant, reply: &mut Reply<M>) {
        if let Some(id) = self.drawing_at(x, y) {
            self.focus = None;
            self.press_drawing(id, x, y, now, reply);
        } else if matches!(self.focus.as_deref().and_then(|id| self.node(id)).map(|n| &n.what), Some(What::Entry { leave: Some(_), .. })) {
            self.focus = None;
        }
    }

    fn press_widget(&mut self, id: String, x: f32, y: f32, now: Instant, reply: &mut Reply<M>) {
        self.focus = Some(id.clone());
        self.pressed = Some(id.clone());
        self.row_drag = None;
        self.band = None;
        if !self.band_press(&id, x, y) && matches!(self.node(&id).map(|n| &n.what), Some(What::List { reorder: Some(_), .. } | What::List { move_in: Some(_), .. })) {
            if let Some(row) = self.row_at(&id, y) {
                self.row_drag = Some((id.clone(), row, y, false));
            }
        }
        self.eyes.on = None;
        if self.eye_at(x, y).as_deref() == Some(id.as_str()) {
            self.eyes.down = Some(id);
            return;
        }
        if matches!(self.node(&id).map(|n| &n.what), Some(What::Rich { .. })) && !self.rich_press(&id, x, y) {
            self.pressed = None;
        } else if matches!(self.node(&id).map(|n| &n.what), Some(What::Slider { .. })) {
            self.dragging = Some(id.clone());
            self.slide_to(&id, x, reply);
        } else if self.is_divider(&id) {
            self.divider_press(id, (x, y), now);
        } else if self.is_entry(&id) {
            self.press_entry(id, x, y, now);
        }
    }

    fn press_entry(&mut self, id: String, x: f32, y: f32, now: Instant) {
        let count = match &self.last_click {
            Some((at, last, n)) if *last == id && now.saturating_duration_since(*at) < crate::host::host().double_click() => n % 3 + 1,
            _ => 1,
        };
        self.last_click = Some((now, id.clone(), count));
        let at = self.entry_hit(&id, x);
        let secret = self.entry_secret(&id);
        if count == 1 && !secret && self.lift_press(&id, (x, y), at) {
            return;
        }
        let Some(e) = self.edit(&id) else { return };
        match count {
            1 => e.move_to(at, false),
            2 if !secret => e.select(e.word_round(at)),
            _ => e.select(0..e.text.len()),
        }
        self.dragging = Some(id);
    }

    pub(super) fn release(&mut self, x: f32, y: f32, now: Instant, reply: &mut Reply<M>) {
        reply.redraw = true;
        if self.lift_let_go() || self.band_let_go(reply) {
            return;
        }
        if let Some(id) = self.dragging.take().filter(|id| self.is_pointed(id)) {
            self.landed(x, y, reply);
            self.point(&id, x, y, w::Pointer::Up, reply);
            self.drop_hover(None, reply);
            return;
        }
        if self.thumb.take().is_some() {
            self.pointer_to(x, y, now, reply);
            return;
        }
        if let Some((id, row, _, true)) = self.row_drag.take() {
            self.pressed = None;
            let Some((to, gap)) = self.gap.take() else { return };
            match self.node(&to).map(|n| &n.what) {
                Some(What::List { reorder: Some(f), .. }) if to == id && gap != row && gap != row + 1 => reply.messages.push(f(row, gap)),
                Some(What::List { move_in: Some(f), .. }) if to != id => reply.messages.push(f(id, row, gap)),
                _ => {}
            }
            return;
        }
        let Some(id) = self.pressed.take() else { return };
        if self.eyes.down.take().is_some_and(|eye| eye == id) {
            if self.eye_at(x, y).as_deref() == Some(id.as_str()) {
                self.eye_press(&id, now, reply);
            }
            return;
        }
        if self.hit(x, y).as_deref() == Some(&id) {
            if self.is_list(&id) {
                if let Some(row) = self.row_at(&id, y) {
                    if self.on_arrow(&id, row, x) {
                        self.tree_toggle(&id, row, reply);
                    } else {
                        self.choose_row(&id, row, now, reply);
                    }
                }
            } else if let Some(i) = self.arrow_at(&id, x) {
                self.along = Some((id.clone(), i));
                self.open_sub(&id, i, reply);
            } else if let Some(i) = self.pill(&id, x) {
                if let Some(What::Choice { chosen, choose, .. }) = self.node(&id).map(|n| &n.what) {
                    if i != *chosen {
                        reply.messages.push(choose(i));
                    }
                }
            } else {
                self.activate(&id, now, reply);
            }
        }
    }

    pub(super) fn activate(&mut self, id: &str, now: Instant, reply: &mut Reply<M>) {
        let twice = self.node(id).filter(|n| !n.off).and_then(|n| match &n.what {
            What::Button { press: Some(_), tip, .. } if n.twice => Some(tip.clone()),
            _ => None,
        });
        if let Some(tip) = twice {
            if let Some(act) = self.node(id).and_then(|n| n.own_act) {
                if !crate::window::persons_own(act) {
                    self.armed = None;
                    self.follow_armed();
                    reply.redraw = true;
                    return;
                }
            }
            if !self.is_armed(id) {
                self.armed = Some((id.to_string(), now));
                self.follow_armed();
                self.tip = Some(Tip { id: id.to_string(), state: TipState::Shown(now) });
                self.news = crate::common::ui::click_again(&tip);
                reply.redraw = true;
                return;
            }
            self.armed = None;
            self.follow_armed();
            reply.redraw = true;
        }
        if let Some(k) = self.node(id).and_then(|n| n.when_armed.clone()) {
            if let Some((_, at)) = self.armed.as_mut().filter(|(a, _)| *a == k) {
                *at = now;
            }
        }
        if self.is_dropdown(id) {
            self.open_drop(id);
            reply.redraw = true;
            return;
        }
        match self.node(id).filter(|n| !n.off).map(|n| &n.what) {
            Some(What::Button { press: Some(m), .. } | What::Card(_, Some(m)) | What::Drawing { click: Some(m), .. }) => {
                self.mark_pressed(id);
                reply.messages.push(m.clone());
                self.eyes_shut(reply);
            }
            Some(What::Switch { on, change, .. } | What::Tick { on, change, .. }) => reply.messages.push(change(!on)),
            Some(What::Rich { .. }) => reply.messages.extend(self.rich_follow(id)),
            _ => {}
        }
    }

    pub(super) fn mark_pressed(&self, id: &str) {
        if let Some(s) = self.spot(id) {
            let k = self.k();
            self.last_pressed.set(Some(Rect { x: s.rect.x / k, y: s.rect.y / k, w: s.rect.w / k, h: s.rect.h / k }));
        }
    }

    pub fn take_pressed(&mut self) -> Option<Rect> {
        self.last_pressed.take()
    }

    pub(super) fn slide_to(&self, id: &str, x: f32, reply: &mut Reply<M>) {
        let (Some(spot), Some(node)) = (self.spot(id), self.node(id).filter(|n| !n.off)) else { return };
        let What::Slider { value, min, max, step, change, .. } = &node.what else { return };
        let knob = (theme::sheet().knob * self.scale).round();
        let (left, right) = (spot.rect.x + knob / 2.0, spot.rect.x + spot.rect.w - knob / 2.0);
        let t = if right > left { ((x - left) / (right - left)).clamp(0.0, 1.0) } else { 0.0 };
        let v = snap(min + t * (max - min), *min, *max, *step);
        if v != *value {
            reply.messages.push(change(v));
            reply.redraw = true;
        }
    }
}
