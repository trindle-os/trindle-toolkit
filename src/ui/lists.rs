//! Lists and menus that scroll, their rows and keys.

use super::*;

impl<M: Clone> Ui<M> {
    pub(super) fn is_list(&self, id: &str) -> bool {
        matches!(self.node(id).map(|n| &n.what), Some(What::List { .. }))
    }

    pub(super) fn is_tree(&self, id: &str) -> bool {
        matches!(self.node(id).map(|n| &n.what), Some(What::List { tree: Some(_), .. }))
    }

    pub fn keyboard(&self) -> Keyboard {
        let focus = self.focus.as_deref();
        let text = focus.is_some_and(|id| self.is_entry(id) || self.keys_of(id).is_some());
        let radio = focus.and_then(|id| self.node(id)).is_some_and(|n| n.says.as_ref().is_some_and(|s| s.radio.is_some()));
        let arrows = text || radio || focus.is_some_and(|id| matches!(self.node(id).map(|n| &n.what),
            Some(What::List { .. } | What::Slider { .. } | What::Choice { .. })));
        Keyboard { keys: self.keyboard_seen && focus.is_some(), text, arrows }
    }

    pub(super) fn has_menu(&self, id: &str) -> bool {
        matches!(self.node(id).map(|n| &n.what), Some(What::List { menu: Some(_), .. }))
    }

    pub(super) fn list_menu(&mut self, id: &str, row: usize, x: f32, y: f32, reply: &mut Reply<M>) -> bool {
        let k = self.k();
        let Some(What::List { chosen, choose, many, menu: Some(menu), .. }) = self.node(id).map(|n| &n.what) else { return false };
        let among = many.as_ref().map_or(*chosen == Some(row), |m| m.picked.contains(&row));
        if !among {
            if *chosen != Some(row) {
                reply.messages.push(choose(row));
            }
            if let Some(m) = many {
                reply.messages.push((m.pick)(vec![row]));
            }
        }
        reply.messages.push(menu(row, x / k, y / k));
        if many.is_some() && !among {
            self.anchor = Some((id.to_string(), row));
        }
        self.focus = Some(id.to_string());
        reply.redraw = true;
        true
    }

    pub(super) fn menu_row_at(&self, x: f32, y: f32) -> Option<(String, usize)> {
        let id = self.hit(x, y).filter(|id| self.has_menu(id))?;
        let row = self.row_at(&id, y)?;
        Some((id, row))
    }

    pub(super) fn list_ahead(&mut self, id: &str, text: &str, now: Instant, reply: &mut Reply<M>) -> bool {
        if !matches!(self.node(id).map(|n| &n.what), Some(What::List { ahead: true, .. })) {
            return false;
        }
        let mut typed = match self.ahead.clone().filter(|(l, _, _)| l == id) {
            Some((_, letters, at)) => ahead::Typed { letters, at: Some(at) },
            None => ahead::Typed::default(),
        };
        if !typed.takes(text, now) {
            return false;
        }
        let row = self.row_of(id);
        let found = {
            let Some(What::List { rows, chosen, .. }) = self.node(id).map(|n| &n.what) else { return false };
            typed.jump(text, now, *chosen, rows.len(), |i| rows[i].text.clone())
        };
        self.ahead = Some((id.to_string(), typed.letters, now));
        let Some(What::List { chosen, choose, many, .. }) = self.node(id).map(|n| &n.what) else { return false };
        let Some(to) = found else { return true };
        if *chosen != Some(to) {
            reply.messages.push(choose(to));
        }
        if let Some(m) = many {
            if m.picked != [to] {
                reply.messages.push((m.pick)(vec![to]));
            }
            self.anchor = Some((id.to_string(), to));
        }
        self.reveal(id, Some((to as f32 * row, row)));
        reply.redraw = true;
        true
    }

    pub(super) fn span(&self, s: &Spot) -> f32 {
        (s.full - if self.is_scroll_x(s) { s.rect.w } else { s.rect.h }).max(0.0)
    }

    pub(super) fn offset(&self, s: &Spot) -> f32 {
        self.scrolls.get(&s.id).copied().unwrap_or(0.0).clamp(0.0, self.span(s)).round()
    }

    pub(super) fn k(&self) -> f32 {
        self.scale.max(0.01)
    }

    pub(super) fn coast(&mut self, now: Instant) -> bool {
        let Some((area, by)) = self.fling.step(now, theme::sheet().fling_friction, self.scale) else { return false };
        let Some(s) = self.spot(&area) else {
            self.fling.stop();
            return false;
        };
        let moved = self.scroll_area(&area, self.offset(s) + by);
        if !moved {
            self.fling.stop();
        }
        moved
    }

    pub fn scroll_area(&mut self, id: &str, v: f32) -> bool {
        let Some(s) = self.spot(id) else { return false };
        let before = self.offset(s);
        let v = v.clamp(0.0, self.span(s));
        self.scrolls.insert(id.to_string(), v);
        if v.round() == before {
            return false;
        }
        self.laid = None;
        self.settle();
        true
    }

    pub fn aim_at(&mut self, key: &str, above: f32) -> Option<(String, f32, f32)> {
        self.settle();
        let s = self.spot(key)?;
        if s.rect.h <= 0.0 {
            return None;
        }
        let (top, bottom) = (s.rect.y, s.rect.y + s.rect.h);
        let area = self.areas_round(key).pop()?;
        let a = self.spot(&area)?;
        let off = self.offset(a);
        let seen = top >= a.rect.y && bottom <= a.rect.y + a.rect.h;
        let to = if seen { off } else { off + top - a.rect.y - self.px(above) };
        Some((area, off, to.clamp(0.0, (a.full - a.rect.h).max(0.0))))
    }

    #[cfg(any(test, feature = "testing"))]
    pub fn places_at(&mut self, size: (u32, u32)) -> Vec<(String, Rect)> {
        self.places_at_scale(size, 1.0)
    }

    #[cfg(any(test, feature = "testing"))]
    pub fn places_at_scale(&mut self, size: (u32, u32), scale: f32) -> Vec<(String, Rect)> {
        (self.size, self.scale, self.laid) = (size, scale, None);
        self.settle();
        self.spots.iter().map(|s| (s.id.clone(), s.rect)).collect()
    }

    pub(super) fn page(&self, h: f32) -> f32 {
        (h - self.px(theme::sheet().row)).max(h / 2.0)
    }

    pub(super) fn areas_at(&self, x: f32, y: f32) -> Vec<String> {
        self.spots.iter().rev().filter(|s| self.is_scroll(s) && s.full > s.rect.h + 0.5 && s.shows(x, y))
            .map(|s| s.id.clone()).collect()
    }

    pub(super) fn areas_round(&self, id: &str) -> Vec<String> {
        let Some(path) = self.spot(id).map(|s| s.path.clone()) else { return Vec::new() };
        self.spots.iter().rev()
            .filter(|s| s.path.len() < path.len() && path.starts_with(&s.path) && (self.is_scroll(s) || self.is_scroll_x(s)))
            .map(|s| s.id.clone()).collect()
    }

    pub(super) fn areas_x_at(&self, x: f32, y: f32) -> Vec<String> {
        self.spots.iter().rev().filter(|s| self.is_scroll_x(s) && s.full > s.rect.w + 0.5 && s.shows(x, y))
            .map(|s| s.id.clone()).collect()
    }

    pub(super) fn bar(&self, s: &Spot, thick: f32) -> Option<(Rect, Rect)> {
        let r = s.rect;
        let inset = self.px(theme::sheet().scrollbar_inset);
        if self.is_scroll_x(s) {
            if s.full <= r.w + 0.5 {
                return None;
            }
            let track = Rect { x: r.x + inset, y: r.y + r.h - inset - thick, w: (r.w - 2.0 * inset).max(0.0), h: thick };
            let tw = (track.w * r.w / s.full).max(self.px(theme::SCROLLBAR_MIN)).min(track.w);
            let x = track.x + (track.w - tw) * self.offset(s) / (s.full - r.w);
            return Some((track, Rect { x, w: tw, ..track }));
        }
        if s.full <= r.h + 0.5 {
            return None;
        }
        let track = Rect { x: r.x + r.w - inset - thick, y: r.y + inset, w: thick, h: (r.h - 2.0 * inset).max(0.0) };
        let th = (track.h * r.h / s.full).max(self.px(theme::SCROLLBAR_MIN)).min(track.h);
        let y = track.y + (track.h - th) * self.offset(s) / (s.full - r.h);
        Some((track, Rect { y, h: th, ..track }))
    }

    pub(super) fn bar_at(&self, x: f32, y: f32) -> Option<String> {
        let reach = self.px(theme::sheet().scrollbar_hover) + 2.0 * self.px(theme::sheet().scrollbar_inset);
        self.areas_at(x, y).into_iter().find(|id| self.spot(id).is_some_and(|s| x >= s.rect.x + s.rect.w - reach))
            .or_else(|| self.bar_x_at(x, y))
    }

    pub(super) fn bar_x_at(&self, x: f32, y: f32) -> Option<String> {
        let reach = self.px(theme::sheet().scrollbar_hover) + 2.0 * self.px(theme::sheet().scrollbar_inset);
        self.areas_x_at(x, y).into_iter().find(|id| self.spot(id).is_some_and(|s| y >= s.rect.y + s.rect.h - reach))
    }

    pub(super) fn row_of(&self, id: &str) -> f32 {
        match self.node(id).map(|n| &n.what) {
            Some(What::List { row, .. }) => self.px(*row),
            _ => self.px(theme::sheet().row),
        }
    }

    pub(super) fn row_at(&self, id: &str, y: f32) -> Option<usize> {
        let (Some(s), Some(What::List { rows, row, .. })) = (self.spot(id), self.node(id).map(|n| &n.what)) else { return None };
        let i = ((y - s.rect.y) / self.px(*row)).floor();
        (i >= 0.0 && (i as usize) < rows.len()).then_some(i as usize)
    }

    pub(super) fn tree_step(&self, depth: usize) -> f32 {
        self.px(theme::sheet().tree_indent) * depth as f32
    }

    pub(super) fn on_arrow(&self, id: &str, r: usize, x: f32) -> bool {
        let (Some(s), Some(What::List { tree: Some(t), .. })) = (self.spot(id), self.node(id).map(|n| &n.what)) else { return false };
        let Some(&(depth, Some(_))) = t.rows.get(r) else { return false };
        let start = s.rect.x + self.tree_step(depth);
        x >= start && x < start + self.px(theme::sheet().entry_padding + w::CHEVRON + theme::sheet().gap / 2.0)
    }

    pub(super) fn tree_toggle(&self, id: &str, r: usize, reply: &mut Reply<M>) -> bool {
        let Some(What::List { tree: Some(t), open, .. }) = self.node(id).map(|n| &n.what) else { return false };
        match t.rows.get(r) {
            Some((_, Some(_))) => reply.messages.push((t.toggle)(r)),
            Some(_) => reply.messages.extend(open.as_ref().map(|o| o(r))),
            None => {}
        }
        true
    }

    fn tree_key(&self, id: &str, sym: Keysym, at: Option<usize>, reply: &mut Reply<M>) -> Option<Option<usize>> {
        let Some(What::List { tree: Some(t), .. }) = self.node(id).map(|n| &n.what) else { return None };
        let right = match sym {
            Keysym::Right | Keysym::KP_Right => true,
            Keysym::Left | Keysym::KP_Left => false,
            _ => return None,
        };
        let Some(r) = at else { return Some(Some(0)) };
        let Some(&(depth, open)) = t.rows.get(r) else { return Some(None) };
        Some(match (right, open) {
            (true, Some(false)) | (false, Some(true)) => {
                reply.messages.push((t.toggle)(r));
                None
            }
            (true, Some(true)) => t.rows.get(r + 1).filter(|n| n.0 > depth).map(|_| r + 1),
            (true, None) => None,
            (false, _) => t.parent(r),
        })
    }

    pub(super) fn hover_row(&self) -> Option<(String, usize)> {
        let id = self.hover.clone().filter(|id| self.is_list(id))?;
        let row = self.row_at(&id, self.pointer?.1)?;
        Some((id, row))
    }

    pub(super) fn reveal(&mut self, id: &str, part: Option<(f32, f32)>) {
        let margin = if part.is_some() { 0.0 } else { self.px(theme::sheet().focus_offset + theme::sheet().focus_width) };
        for area in self.areas_round(id) {
            let (Some(s), Some(a)) = (self.spot(id), self.spot(&area)) else { return };
            let along = self.is_scroll_x(a);
            let (start, len) = if along { (s.rect.x, s.rect.w) } else { (s.rect.y, s.rect.h) };
            let (top, h) = part.map_or((start, len), |(t, h)| (start + t, h));
            let (top, bottom) = (top - margin, top + h + margin);
            let off = self.offset(a);
            let (view_start, view_len) = if along { (a.rect.x, a.rect.w) } else { (a.rect.y, a.rect.h) };
            let to = if bottom - top > view_len || top < view_start {
                off - (view_start - top)
            } else if bottom > view_start + view_len {
                off + bottom - (view_start + view_len)
            } else {
                continue;
            };
            self.scroll_area(&area, to);
        }
    }

    pub(super) fn choose_row(&mut self, id: &str, row: usize, now: Instant, reply: &mut Reply<M>) {
        let mods = self.mods;
        let start = self.anchor.as_ref().filter(|(a, _)| a == id).map(|(_, r)| *r);
        let Some(What::List { chosen, choose, open, many, .. }) = self.node(id).map(|n| &n.what) else { return };
        let fresh = *chosen != Some(row);
        if fresh {
            reply.messages.push(choose(row));
        }
        if let Some(m) = many {
            let to = if mods.ctrl {
                toggled(&m.picked, row)
            } else if mods.shift {
                run(start.or(*chosen).unwrap_or(row), row)
            } else {
                vec![row]
            };
            if to != m.picked {
                reply.messages.push((m.pick)(to));
            }
        }
        let several = many.is_some();
        let again = !(several && (mods.ctrl || mods.shift)) && self.last_row.as_ref()
            .is_some_and(|(at, last, r)| last == id && *r == row && now.saturating_duration_since(*at) < crate::host::host().double_click());
        if again {
            if !self.tree_toggle(id, row, reply) {
                if let Some(open) = open {
                    reply.messages.push(open(row));
                }
            }
            self.last_row = None;
        } else if several && (mods.ctrl || mods.shift) {
            self.last_row = None;
        } else {
            self.last_row = Some((now, id.to_string(), row));
        }
        if several && !mods.shift {
            self.anchor = Some((id.to_string(), row));
        }
        if fresh {
            self.chose = Some((id.to_string(), row));
        }
    }

    fn list_tree_keys(&mut self, id: &str, sym: Keysym, at: Option<usize>, row: f32, reply: &mut Reply<M>) -> bool {
        if let Some(to) = self.tree_key(id, sym, at, reply) {
            if let Some(to) = to.filter(|&to| at != Some(to)) {
                if let Some(What::List { choose, .. }) = self.node(id).map(|n| &n.what) {
                    reply.messages.push(choose(to));
                }
                self.reveal(id, Some((to as f32 * row, row)));
            }
            reply.redraw = true;
            return true;
        }
        if matches!(sym, Keysym::Return | Keysym::KP_Enter | Keysym::space | Keysym::KP_Space) && self.is_tree(id) {
            if let Some(i) = at {
                self.tree_toggle(id, i, reply);
            }
            return true;
        }
        false
    }

    pub(super) fn list_key(&mut self, id: &str, sym: Keysym, mods: Mods, reply: &mut Reply<M>) -> bool {
        let row = self.row_of(id);
        let seen = self.areas_round(id).first().and_then(|a| self.spot(a)).map_or(self.size.1 as f32, |a| a.rect.h);
        let per = ((seen / row).floor() as usize).max(1);
        let start = self.anchor.as_ref().filter(|(a, _)| a == id).map(|(_, r)| *r);
        let Some(What::List { rows, chosen, .. }) = self.node(id).map(|n| &n.what) else { return false };
        let (n, at) = (rows.len(), *chosen);
        if n == 0 {
            return false;
        }
        let last = n - 1;
        if self.list_tree_keys(id, sym, at, row, reply) {
            return true;
        }
        let Some(What::List { choose, open, many, .. }) = self.node(id).map(|n| &n.what) else { return false };
        if let Some(m) = many {
            if let Some(to) = many_keys(sym, mods, n, at, &m.picked) {
                if to != m.picked {
                    reply.messages.push((m.pick)(to));
                }
                reply.redraw = true;
                return true;
            }
        }
        let to = match sym {
            Keysym::Down | Keysym::KP_Down => at.map_or(0, |i| (i + 1).min(last)),
            Keysym::Up | Keysym::KP_Up => at.map_or(0, |i| i.saturating_sub(1)),
            Keysym::Page_Down | Keysym::KP_Page_Down => at.map_or(0, |i| i.saturating_add(per).min(last)),
            Keysym::Page_Up | Keysym::KP_Page_Up => at.map_or(0, |i| i.saturating_sub(per)),
            Keysym::Home | Keysym::KP_Home => 0,
            Keysym::End | Keysym::KP_End => last,
            Keysym::Return | Keysym::KP_Enter => {
                if let (Some(open), Some(i)) = (open, at) {
                    reply.messages.push(open(i));
                }
                return true;
            }
            Keysym::Menu => {
                let (Some(i), Some(s)) = (at, self.spot(id)) else { return false };
                let (x, y) = (s.rect.x + self.px(theme::sheet().entry_padding), s.rect.y + (i as f32 + 1.0) * row);
                return self.list_menu(id, i, x, y, reply);
            }
            _ => return false,
        };
        if at != Some(to) {
            reply.messages.push(choose(to));
        }
        if let Some(m) = many {
            let p = if mods.shift { run(start.or(at).unwrap_or(to), to) } else { vec![to] };
            if p != m.picked {
                reply.messages.push((m.pick)(p));
            }
            if !mods.shift {
                self.anchor = Some((id.to_string(), to));
            }
        }
        self.reveal(id, Some((to as f32 * row, row)));
        reply.redraw = true;
        true
    }

    pub(super) fn page_key(&mut self, sym: Keysym, reply: &mut Reply<M>) -> bool {
        let area = self.focus.as_deref().and_then(|f| self.areas_round(f).into_iter().next())
            .or_else(|| self.pointer.and_then(|(x, y)| self.areas_at(x, y).into_iter().next()))
            .or_else(|| self.spots.iter().find(|s| self.is_scroll(s) && s.full > s.rect.h + 0.5).map(|s| s.id.clone()));
        let Some(s) = area.as_deref().and_then(|a| self.spot(a)) else { return false };
        let (off, page, end) = (self.offset(s), self.page(s.rect.h), s.full);
        let to = match sym {
            Keysym::Page_Down | Keysym::KP_Page_Down => off + page,
            Keysym::Page_Up | Keysym::KP_Page_Up => off - page,
            Keysym::Home | Keysym::KP_Home => 0.0,
            Keysym::End | Keysym::KP_End => end,
            _ => return false,
        };
        let Some(area) = area else { return false };
        if self.scroll_area(&area, to) {
            reply.redraw = true;
            self.rehover(reply);
        }
        true
    }

    pub(super) fn rehover(&mut self, reply: &mut Reply<M>) {
        if let (Some((x, y)), None) = (self.pointer, self.touch) {
            let hit = self.hit(x, y);
            if hit != self.hover {
                self.hover = hit;
                self.tip = None;
                reply.redraw = true;
            }
            let eye = self.eye_at(x, y);
            if eye != self.eyes.over {
                self.eyes.over = eye;
                self.tip = None;
                reply.redraw = true;
            }
        }
    }
}

fn many_keys(sym: Keysym, mods: Mods, n: usize, at: Option<usize>, picked: &[usize]) -> Option<Vec<usize>> {
    match sym {
        Keysym::a | Keysym::A if mods.ctrl => Some((0..n).collect()),
        Keysym::space => at.map(|i| toggled(picked, i)),
        _ => None,
    }
}
