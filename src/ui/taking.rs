//! Taking what the person did and sending it to the right widget.

use super::*;

impl<M: Clone> Ui<M> {
    pub fn input(&mut self, input: &Input, now: Instant) -> Reply<M> {
        self.settle();
        if matches!(input, Input::Refused | Input::NoLayout) {
            if matches!(input, Input::NoLayout) {
                self.say_refused_as(now, crate::t!("kit-no-layout"));
            } else {
                self.say_refused(now);
            }
            let mut reply = Reply::new();
            reply.redraw = true;
            return reply;
        }
        if matches!(input, Input::Press { .. } | Input::TouchDown { .. } | Input::Key { .. } | Input::Scroll { steps: true, .. }) {
            self.fling.stop();
        }
        if self.dev_key(input) {
            let mut reply = Reply::new();
            reply.redraw = true;
            return reply;
        }
        if matches!(input, Input::Press { .. } | Input::TouchDown { .. } | Input::Key { .. }) {
            self.forget_gone();
            self.chose = None;
        }
        if matches!(input, Input::Key { .. } | Input::Composed { .. }) {
            self.eyes_key(now);
        }
        let before = self.focus.clone();
        let claimed = self.claimed_by(input);
        let mut reply = self.input_here(input, now);
        if let Some(m) = claimed {
            reply.messages.insert(0, m);
            reply.redraw = true;
        }
        if self.eyes.on.is_some() && self.eyes.on != self.focus {
            self.eyes.on = None;
        }
        self.tell_focus(before.as_deref(), &mut reply);
        self.tell_left(before.as_deref(), &mut reply);
        reply.redraw |= self.inspecting() && matches!(input, Input::PointerIn { .. } | Input::PointerMove { .. } | Input::PointerOut);
        reply
    }

    pub(super) fn claimed_by(&self, input: &Input) -> Option<M> {
        let (Input::Press { x, y, .. } | Input::TouchDown { x, y, .. }) = *input else { return None };
        if self.copy_menu.is_some() || self.drop.is_some() {
            return None;
        }
        let (x, y) = (x * self.scale, y * self.scale);
        self.spots.iter().rev().filter(|s| s.shows(x, y)).find_map(|s| self.tree.at(&s.path).and_then(|n| n.claims.clone()))
    }

    pub(super) fn tell_left(&self, before: Option<&str>, reply: &mut Reply<M>) {
        if before == self.focus.as_deref() {
            return;
        }
        if let Some(What::Entry { leave: Some(m), .. }) = before.and_then(|id| self.node(id)).map(|n| &n.what) {
            reply.messages.push(m.clone());
            reply.redraw = true;
        }
    }

    pub(super) fn input_here(&mut self, input: &Input, now: Instant) -> Reply<M> {
        if self.copy_menu.is_some() {
            if let Some(reply) = self.menu_input(input) {
                return reply;
            }
        }
        if self.drop.is_some() {
            if let Some(reply) = self.drop_input(input, now) {
                return reply;
            }
        }
        let mut reply = Reply::new();
        if self.outside_input(input, &mut reply) {
            return reply;
        }
        let scale = self.scale;
        let px = |v: f32| v * scale;
        match *input {
            Input::PointerIn { x, y } | Input::PointerMove { x, y } => self.pointer_to(px(x), px(y), now, &mut reply),
            Input::PointerOut => self.pointer_out(&mut reply),
            Input::Press { button: Button::Left, x, y } => self.press(px(x), px(y), now, &mut reply),
            Input::Release { button: Button::Left, x, y } => self.release(px(x), px(y), now, &mut reply),
            Input::Press { button: Button::Right, x, y } => self.right_press(px(x), px(y), &mut reply),
            Input::Release { button: Button::Right, x, y } => self.right_release(px(x), px(y), &mut reply),
            Input::Press { button: Button::Middle, x, y } => self.middle_press(px(x), px(y), &mut reply),
            Input::Scroll { x, y, steps } if y != 0.0 || x != 0.0 => self.wheel(x, y, steps, now, &mut reply),
            Input::Scroll { steps: false, .. } => self.fling.let_go(now, scale),
            Input::TouchDown { id, x, y } if self.touch.is_none() => self.touch_down(id, px(x), px(y), now, &mut reply),
            Input::TouchMove { id, x, y } if self.touch == Some(id) => self.touch_move(px(x), px(y), now, &mut reply),
            Input::TouchUp { id } if self.touch == Some(id) => self.touch_up(now, &mut reply),
            Input::TouchCancel => self.touch_cancel(&mut reply),
            Input::Key { sym, ref text, mods, .. } => self.key_down(input, sym, text.as_deref(), mods, now, &mut reply),
            Input::KeyUp { sym, mods } => self.mods = held(mods, sym, false),
            Input::Mods(mods) => self.mods = mods,
            Input::Focus(here) => self.focus_to(here, &mut reply),
            Input::Composed { .. } if self.to_keyed(input, &mut reply) => {}
            Input::Composed { delete, ref commit, ref preedit, caret } => self.compose(delete, commit, preedit, caret, &mut reply),
            _ => {}
        }
        reply
    }

    fn pointer_out(&mut self, reply: &mut Reply<M>) {
        self.pointer = None;
        self.hover_drawing(None, reply);
        reply.redraw |= self.eyes.over.take().is_some();
        reply.redraw |= self.hover.take().is_some() | self.tip.take().is_some_and(|t| matches!(t.state, TipState::Shown(_)));
    }

    fn touch_up(&mut self, now: Instant, reply: &mut Reply<M>) {
        if self.swipe.as_ref().is_some_and(|s| s.3) {
            self.fling.let_go(now, self.scale);
        }
        self.touch = None;
        self.hold = None;
        self.swipe = None;
        self.swipe_x = None;
        let (x, y) = self.pointer.unwrap_or((-1.0, -1.0));
        self.release(x, y, now, reply);
        self.pointer = None;
        self.hover = None;
        self.eyes.over = None;
        self.tip = None;
    }

    fn touch_cancel(&mut self, reply: &mut Reply<M>) {
        self.eyes.down = None;
        self.eyes.over = None;
        self.touch = None;
        self.hold = None;
        self.swipe = None;
        self.swipe_x = None;
        self.thumb = None;
        self.pressed = None;
        self.dragging = None;
        self.hover = None;
        reply.redraw = true;
    }

    fn key_down(&mut self, input: &Input, sym: Keysym, text: Option<&str>, mods: Mods, now: Instant, reply: &mut Reply<M>) {
        self.mods = held(mods, sym, true);
        let had_tip = self.tip.take().is_some();
        reply.redraw |= !self.keyboard_seen || had_tip;
        self.keyboard_seen = true;
        if !self.to_keyed(input, reply) {
            self.key(sym, text, mods, now, reply);
        }
    }

    fn focus_to(&mut self, here: bool, reply: &mut Reply<M>) {
        self.keys_here = here;
        if !here {
            self.mods = Mods::default();
            self.pressed = None;
            self.dragging = None;
            self.eyes.down = None;
            self.eyes_shut(reply);
        }
        reply.redraw = true;
    }

    pub(super) fn right_press(&mut self, x: f32, y: f32, reply: &mut Reply<M>) {
        self.right = None;
        if let Some((id, row)) = self.menu_row_at(x, y) {
            self.list_menu(&id, row, x, y, reply);
        } else if let Some(id) = self.drawing_at(x, y) {
            self.mark_pressed(&id);
            if self.keys_of(&id).is_some() {
                self.focus = Some(id.clone());
            }
            if self.right_drags(&id) {
                self.right = Some((id, (x, y), false));
            } else {
                if !self.drawing_menu(&id, x, y, false) {
                    self.context_menu_at(x, y);
                }
                self.point(&id, x, y, w::Pointer::Menu, reply);
            }
        } else {
            self.menu_for(x, y);
        }
        reply.redraw = true;
    }

    pub(super) fn right_release(&mut self, x: f32, y: f32, reply: &mut Reply<M>) {
        if let Some((id, from, dragged)) = self.right.take() {
            if dragged {
                self.landed(x, y, reply);
                self.point(&id, x, y, w::Pointer::RightUp, reply);
                self.drop_hover(None, reply);
            } else {
                if !self.drawing_menu(&id, from.0, from.1, false) {
                    self.context_menu_at(from.0, from.1);
                }
                self.point(&id, from.0, from.1, w::Pointer::Menu, reply);
            }
            reply.redraw = true;
        }
    }

    pub(super) fn middle_press(&mut self, x: f32, y: f32, reply: &mut Reply<M>) {
        if let Some(id) = self.drawing_at(x, y) {
            self.point(&id, x, y, w::Pointer::Middle, reply);
        } else if let Some(id) = self.hit(x, y) {
            match self.node(&id).filter(|n| !n.off).map(|n| &n.what) {
                Some(What::List { middle: Some(f), .. }) => {
                    if let Some(row) = self.row_at(&id, y) {
                        reply.messages.push(f(row));
                    }
                }
                Some(What::Choice { middle: Some(f), .. }) => {
                    if let Some(i) = self.pill(&id, x) {
                        reply.messages.push(f(i));
                    }
                }
                _ => {}
            }
        }
    }

    pub(super) fn wheel(&mut self, x: f32, y: f32, steps: bool, now: Instant, reply: &mut Reply<M>) {
        let scale = self.scale;
        let Some((x0, y0)) = self.pointer else { return };
        if let (true, Some(id)) = (steps && y != 0.0, self.focus.clone()) {
            if let (Some(What::Entry { steps: Some(f), .. }), Some(s)) = (self.node(&id).map(|n| &n.what), self.spot(&id)) {
                if s.shows(x0, y0) {
                    reply.messages.push(f(if y < 0.0 { 1 } else { -1 }));
                    reply.redraw = true;
                    return;
                }
            }
        }
        if y != 0.0 {
            let lines = if steps { y * WHEEL_LINES } else { y / TOUCHPAD_LINE };
            if let Some(m) = self.scrolled_drawing(x0, y0, lines) {
                reply.messages.push(m);
                reply.redraw = true;
                return;
            }
        }
        let along = if x != 0.0 { x } else if self.mods.shift { y } else { 0.0 };
        if along != 0.0 && self.wheel_along(along, steps, reply) {
            return;
        }
        let area = self.areas_at(x0, y0).into_iter().find(|id| y != 0.0 && self.spot(id).is_some_and(|s| {
            let off = self.offset(s);
            if y > 0.0 { off < s.full - s.rect.h - 0.5 } else { off > 0.5 }
        }));
        let Some(area) = area else {
            if along == 0.0 && y != 0.0 && self.wheel_along(y, steps, reply) {
                return;
            }
            self.wheel_slide(x0, y0, if y != 0.0 { y } else { x }, steps, reply);
            return;
        };
        let Some(s) = self.spot(&area) else { return };
        let by = if steps { y * notch(s.rect.h, scale) * scale } else { y * scale };
        let to = self.scrolls.get(&area).copied().unwrap_or(0.0).clamp(0.0, (s.full - s.rect.h).max(0.0)) + by;
        if !steps {
            self.fling.note(&area, now, to);
        }
        if self.scroll_area(&area, to) {
            reply.redraw = true;
            self.tip = None;
            self.pointer_to(x0, y0, now, reply);
        }
    }

    fn wheel_along(&mut self, by: f32, steps: bool, reply: &mut Reply<M>) -> bool {
        let Some((x0, y0)) = self.pointer else { return false };
        let area = self.areas_x_at(x0, y0).into_iter().find(|id| self.spot(id).is_some_and(|s| {
            let off = self.offset(s);
            if by > 0.0 { off < s.full - s.rect.w - 0.5 } else { off > 0.5 }
        }));
        let Some(area) = area else { return false };
        let Some(s) = self.spot(&area) else { return false };
        let by = if steps { by * notch(s.rect.w, self.scale) * self.scale } else { by * self.scale };
        let to = self.scrolls.get(&area).copied().unwrap_or(0.0).clamp(0.0, self.span(s)) + by;
        if self.scroll_area(&area, to) {
            reply.redraw = true;
            self.tip = None;
        }
        true
    }

    pub(super) fn touch_down(&mut self, id: i32, x: f32, y: f32, now: Instant, reply: &mut Reply<M>) {
        self.touch = Some(id);
        self.pointer_to(x, y, now, reply);
        self.tip = None;
        self.press(x, y, now, reply);
        self.touches += 1;
        self.hold = None;
        if self.dragging.as_deref().is_some_and(|d| self.is_pointed(d)) || self.menu_row_at(x, y).is_some() || self.has_context_at(x, y) {
            self.hold = Some((x, y, self.touches));
            reply.hold = Some(self.touches);
        }
        if self.dragging.is_none() && self.thumb.is_none() {
            self.swipe = self.areas_at(x, y).into_iter().next()
                .and_then(|a| self.spot(&a).map(|s| (a.clone(), y, self.offset(s), false)));
        }
        self.touch_down_x(x, y);
    }

    pub(super) fn touch_move(&mut self, x: f32, y: f32, now: Instant, reply: &mut Reply<M>) {
        if self.hold.is_some_and(|(x0, y0, _)| (x - x0).hypot(y - y0) > HOLD_SLOP * self.scale) {
            self.hold = None;
        }
        if self.touch_move_x(x, y, reply) {
            return;
        }
        let slop = SWIPE_SLOP * self.scale;
        match self.swipe.clone() {
            Some((area, y0, off0, started)) if started || (y - y0).abs() > slop => {
                if !started {
                    self.swipe = Some((area.clone(), y0, off0, true));
                    self.pressed = None;
                    self.eyes.down = None;
                    self.hover = None;
                    reply.redraw = true;
                }
                self.pointer = Some((x, y));
                reply.redraw |= self.scroll_area(&area, off0 + y0 - y);
                self.fling.note(&area, now, off0 + y0 - y);
            }
            _ => self.pointer_to(x, y, now, reply),
        }
    }

    pub(super) fn compose(&mut self, delete: (u32, u32), commit: &str, preedit: &str, caret: Option<(usize, usize)>, reply: &mut Reply<M>) {
        let Some(id) = self.focus.clone().filter(|id| self.is_entry(id)) else { return };
        self.eyes.on = None;
        let had = self.preedit.take().is_some();
        let Some(e) = self.edit(&id) else { return };
        let before = e.text.clone();
        if delete != (0, 0) {
            let mut start = e.caret.saturating_sub(delete.0 as usize);
            while !e.text.is_char_boundary(start) {
                start -= 1;
            }
            let mut end = (e.caret + delete.1 as usize).min(e.text.len());
            while !e.text.is_char_boundary(end) {
                end += 1;
            }
            e.select(start..end);
            e.delete_to(start);
        }
        if !commit.is_empty() {
            e.insert(commit);
        }
        if !preedit.is_empty() {
            let edge = |at: usize| at.min(preedit.len());
            let caret = caret.map(|(a, b)| (edge(a), edge(b)))
                .filter(|&(a, b)| preedit.is_char_boundary(a) && preedit.is_char_boundary(b));
            self.preedit = Some((id.clone(), preedit.to_string(), caret));
        }
        self.changed(&id, &before, reply);
        reply.redraw |= had || self.preedit.is_some() || !reply.messages.is_empty();
    }

    pub(super) fn preedit_in(&self, id: &str) -> Option<(&str, Option<(usize, usize)>)> {
        self.preedit.as_ref().filter(|(at, ..)| at == id && self.focus.as_deref() == Some(id)).map(|(_, t, c)| (t.as_str(), *c))
    }

    pub fn typing(&mut self) -> Option<Typing> {
        self.settle();
        if !self.keys_here {
            return None;
        }
        let id = self.focus.clone().filter(|id| self.is_entry(id))?;
        let Some(What::Entry { secret, .. }) = self.node(&id).map(|n| &n.what) else { return None };
        let secret = *secret;
        let dots = self.entry_dots(&id);
        let rect = self.spot(&id)?.rect;
        let pad = self.entry_pad(&id);
        let e = self.edit(&id)?;
        let (text, caret, anchor, scroll) = (e.text.clone(), e.caret, e.anchor, e.scroll);
        let layout = self.entry_layout(&id, &text, dots);
        let scroll = scroll - self.entry_lead(&id, if text.is_empty() { 0.0 } else { layout.width }, rect.w - 2.0 * pad);
        let at = if dots { edit::to_dots(&text, caret) } else { caret };
        let (x, top, h) = if text.is_empty() { (0.0, 0.0, layout.height) } else { layout.caret(at) };
        let ty = ((rect.h - layout.height) / 2.0).round();
        let pt = |v: f32| (v / self.scale).round() as i32;
        let area = (pt(rect.x + pad + x - scroll), pt(rect.y + ty + top), pt(self.px(theme::sheet().caret).max(1.0)).max(1), pt(h));
        let purpose = self.node(&id).and_then(|n| n.says.as_deref()).and_then(|s| s.purpose).unwrap_or_default();
        Some(Typing { id, text, caret, anchor, area, secret, purpose })
    }
}
