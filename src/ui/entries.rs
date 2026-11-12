//! Typing boxes, pickers and the focus.

use super::*;

impl<M: Clone> Ui<M> {
    pub(super) fn pick_at(&self, x: f32, y: f32) -> Option<String> {
        let s = self.content_at(x, y)?;
        let n = self.tree.at(&s.path)?;
        if !matches!(n.what, What::Label { .. }) || self.touch.is_some() {
            return None;
        }
        let pressed = (0..s.path.len()).any(|k| {
            matches!(self.tree.at(&s.path[..k]).map(|a| &a.what), Some(What::Card(_, Some(_)) | What::Button { .. } | What::Drawing { .. }))
        });
        (!pressed).then(|| s.id.clone())
    }

    pub(super) fn pick_layout(&self, id: &str) -> Option<(Rc<Layout>, Rect)> {
        let spot = self.spot(id)?;
        let node = self.node(id)?;
        let What::Label { text, words, .. } = &node.what else { return None };
        let width = if node.cut == 1 { None } else { Some(spot.rect.w.max(1.0)) };
        Some((self.texts.get(&self.fonts, text, &words.style(self.scale), width), spot.rect))
    }

    pub(super) fn pick_hit(&self, id: &str, x: f32, y: f32) -> usize {
        self.pick_layout(id).map_or(0, |(l, r)| l.hit(x - r.x, y - r.y))
    }

    pub(super) fn pick_press(&mut self, x: f32, y: f32, now: Instant) -> bool {
        let Some(id) = self.pick_at(x, y) else {
            self.picked = None;
            return false;
        };
        let Some(What::Label { text, .. }) = self.node(&id).map(|n| &n.what) else { return false };
        let mut e = Edit::new(text);
        let count = match &self.last_click {
            Some((at, last, n)) if *last == id && now.saturating_duration_since(*at) < crate::host::host().double_click() => n % 3 + 1,
            _ => 1,
        };
        self.last_click = Some((now, id.clone(), count));
        let at = self.pick_hit(&id, x, y);
        if count == 1 && self.lift_press(&id, (x, y), at) {
            return true;
        }
        match count {
            1 => e.move_to(at, false),
            2 => e.select(e.word_round(at)),
            _ => e.select(0..e.text.len()),
        }
        self.focus = None;
        self.picked = Some((id.clone(), e));
        self.dragging = Some(id);
        true
    }

    pub(super) fn pick_key(&mut self, sym: Keysym, mods: Mods, reply: &mut Reply<M>) -> bool {
        let Some((_, e)) = &mut self.picked else { return false };
        match sym {
            Keysym::c | Keysym::C | Keysym::Insert | Keysym::KP_Insert if mods.ctrl => {
                let r = e.selection();
                if r.is_empty() {
                    return false;
                }
                reply.copy = Some(e.selected().to_string());
                true
            }
            Keysym::a | Keysym::A if mods.ctrl => {
                let all = 0..e.text.len();
                e.select(all);
                reply.redraw = true;
                true
            }
            _ => false,
        }
    }

    pub(super) fn is_entry(&self, id: &str) -> bool {
        matches!(self.node(id).map(|n| &n.what), Some(What::Entry { .. }))
    }

    pub(super) fn entry_secret(&self, id: &str) -> bool {
        matches!(self.node(id).map(|n| &n.what), Some(What::Entry { secret: true, .. }))
    }

    pub(super) fn edit(&mut self, id: &str) -> Option<&mut Edit> {
        let Some(What::Entry { text, secret, .. }) = self.tree.at(&self.spots.iter().find(|s| s.id == id)?.path).map(|n| &n.what)
            else { return None };
        let e = self.edits.entry(id.to_string()).or_insert_with(|| Edit::new(text));
        e.sync(text);
        e.keep_history = !*secret;
        Some(e)
    }

    pub(super) fn entry_pad(&self, id: &str) -> f32 {
        let snug = self.node(id).is_some_and(|n| n.snug);
        self.px(if snug { theme::sheet().snug_padding } else { theme::sheet().entry_padding })
    }

    pub(super) fn entry_lead(&self, id: &str, words: f32, room: f32) -> f32 {
        if self.entry_lines(id) > 1 {
            return 0.0;
        }
        match self.node(id).map(|n| &n.what) {
            Some(What::Entry { centre: true, .. }) => ((room - words) / 2.0).floor().max(0.0),
            Some(What::Entry { text, .. }) if !self.entry_dots(id) && edit::right_to_left(text) => (room - words).floor().max(0.0),
            _ => 0.0,
        }
    }

    pub(super) fn entry_style(&self, id: &str) -> Style {
        let plain = Words::Plain.style(self.scale);
        match self.node(id).map(|n| &n.what) {
            Some(What::Entry { weight: Some(w), .. }) => Style { weight: *w, ..plain },
            _ => plain,
        }
    }

    pub(super) fn entry_lines(&self, id: &str) -> usize {
        match self.node(id).map(|n| &n.what) {
            Some(What::Entry { lines, .. }) => *lines,
            _ => 1,
        }
    }

    pub(super) fn entry_down(&self, id: &str, high: f32, words: f32) -> f32 {
        if self.entry_lines(id) <= 1 {
            return ((high - words) / 2.0).round();
        }
        self.entry_rise(id) - self.edits.get(id).map_or(0.0, |e| e.scroll_y)
    }

    pub(super) fn entry_rise(&self, id: &str) -> f32 {
        w::lines_pad(self.px(theme::sheet().entry_height), self.entry_line(id))
    }

    pub(super) fn entry_line(&self, id: &str) -> f32 {
        self.texts.get(&self.fonts, " ", &self.entry_style(id), None).height
    }

    pub(super) fn entry_tall(&self, id: &str) -> f32 {
        self.spot(id).map_or(0.0, |s| s.rect.h) - 2.0 * self.entry_rise(id)
    }

    pub(super) fn wheel_entry(&mut self, x: f32, y: f32, lines: f32) -> bool {
        let Some(id) = self.spots.iter().rev().find(|s| s.shows(x, y)).map(|s| s.id.clone()).filter(|id| self.entry_lines(id) > 1) else { return false };
        if self.edit(&id).is_none() {
            return false;
        }
        let Some(e) = self.edits.get(&id) else { return false };
        let reach = (self.entry_layout(&id, &e.text, self.entry_dots(&id)).height - self.entry_tall(&id)).max(0.0);
        let by = lines * self.entry_line(&id);
        let Some(e) = self.edit(&id) else { return false };
        let to = (e.scroll_y + by).clamp(0.0, reach);
        if to == e.scroll_y {
            return false;
        }
        e.viewed.get_or_insert((e.caret, e.text.len()));
        e.scroll_y = to;
        true
    }

    pub(super) fn drag_edge(&self) -> Option<(String, f32)> {
        let id = self.dragging.as_deref().filter(|id| self.entry_lines(id) > 1)?;
        let (r, (_, y)) = (self.spot(id)?.rect, self.pointer?);
        let way = if y < r.y { -1.0 } else if y > r.y + r.h { 1.0 } else { return None };
        Some((id.to_string(), way))
    }

    pub(super) fn drag_scroll(&mut self, now: Instant) -> bool {
        let Some((id, way)) = self.drag_edge() else {
            self.edge = None;
            return false;
        };
        if self.edge.is_some_and(|t| now < t + EDGE_STEP) {
            return false;
        }
        self.edge = Some(now);
        if self.edit(&id).is_none() {
            return false;
        }
        let (Some((x, y)), Some(e)) = (self.pointer, self.edits.get(&id)) else { return false };
        let reach = (self.entry_layout(&id, &e.text, false).height - self.entry_tall(&id)).max(0.0);
        let by = way * self.entry_line(&id);
        let Some(e) = self.edit(&id) else { return false };
        let to = (e.scroll_y + by).clamp(0.0, reach);
        let slid = to != e.scroll_y;
        e.scroll_y = to;
        let at = self.entry_hit_at(&id, x, self.drag_y(&id, y));
        let Some(e) = self.edit(&id) else { return false };
        let grew = e.caret != at;
        e.move_to(at, true);
        slid || grew
    }

    pub(super) fn drag_y(&self, id: &str, y: f32) -> f32 {
        let Some(r) = self.spot(id).map(|s| s.rect).filter(|_| self.entry_lines(id) > 1) else { return y };
        let top = r.y + self.entry_rise(id);
        y.clamp(top, (r.y + r.h - self.entry_rise(id) - 1.0).max(top))
    }

    pub(super) fn entry_layout(&self, id: &str, text: &str, dots: bool) -> Rc<Layout> {
        let shown = if dots { std::borrow::Cow::Owned(edit::dots(text)) } else { std::borrow::Cow::Borrowed(text) };
        let across = self.spot(id).filter(|_| self.entry_lines(id) > 1).map(|s| (s.rect.w - 2.0 * self.entry_pad(id)).max(1.0));
        self.texts.get(&self.fonts, &shown, &self.entry_style(id), across)
    }

    pub(super) fn entry_hit_at(&self, id: &str, x: f32, y: f32) -> usize {
        let (Some(spot), Some(What::Entry { text, .. })) = (self.spot(id), self.node(id).map(|n| &n.what)) else { return 0 };
        let dots = self.entry_dots(id);
        let scroll = self.edits.get(id).map_or(0.0, |e| e.scroll);
        let pad = self.entry_pad(id);
        let layout = self.entry_layout(id, text, dots);
        let room = spot.rect.w - 2.0 * pad - if self.entry_secret(id) { self.eye_room(spot.rect) } else { 0.0 };
        let lead = self.entry_lead(id, layout.width, room);
        let down = if self.entry_lines(id) > 1 { self.entry_down(id, spot.rect.h, layout.height) } else { 0.0 };
        let at = layout.hit(x - spot.rect.x - pad - lead + scroll, y - spot.rect.y - down);
        if dots { edit::from_dots(text, at) } else { at }
    }

    pub(super) fn entry_words(&self, id: &str, text: &str) -> String {
        if self.entry_lines(id) > 1 { outside::many_lines(text) } else { outside::one_line(text) }
    }

    pub(super) fn edit_key(&mut self, id: &str, sym: Keysym, text: Option<&str>, mods: Mods, reply: &mut Reply<M>) {
        let secret = self.entry_secret(id);
        if matches!(sym, Keysym::Return | Keysym::KP_Enter) {
            if self.entry_lines(id) > 1 && !mods.ctrl {
                self.insert_in(id, "\n", reply);
            } else {
                self.submit_entry(id, reply);
            }
            return;
        }
        if let (Keysym::Up | Keysym::KP_Up | Keysym::Down | Keysym::KP_Down, Some(What::Entry { updown: Some(f), .. })) = (sym, self.node(id).map(|n| &n.what)) {
            reply.messages.push(f(matches!(sym, Keysym::Down | Keysym::KP_Down)));
            return;
        }
        if sym == Keysym::BackSpace && self.edit(id).is_some_and(|e| e.text.is_empty()) {
            if let Some(What::Entry { back: Some(m), .. }) = self.node(id).map(|n| &n.what) {
                reply.messages.push(m.clone());
                return;
            }
        }
        if self.entry_lines(id) > 1 && self.move_by_line(id, sym, mods) {
            reply.redraw = true;
            return;
        }
        let Some(e) = self.edit(id) else { return };
        let before = e.stamp();
        if !(edit_moving(e, sym, mods, secret, reply) || edit_clipboard(e, sym, mods, secret, reply) || edit_typing(e, text, mods)) {
            return;
        }
        reply.redraw = true;
        self.changed(id, before, reply);
    }

    fn insert_in(&mut self, id: &str, words: &str, reply: &mut Reply<M>) {
        let Some(e) = self.edit(id) else { return };
        let before = e.stamp();
        e.insert(words);
        reply.redraw = true;
        self.changed(id, before, reply);
    }

    fn move_by_line(&mut self, id: &str, sym: Keysym, mods: Mods) -> bool {
        let (up, down) = (matches!(sym, Keysym::Up | Keysym::KP_Up), matches!(sym, Keysym::Down | Keysym::KP_Down));
        let (home, end) = (matches!(sym, Keysym::Home | Keysym::KP_Home), matches!(sym, Keysym::End | Keysym::KP_End));
        if !(up || down || home || end) || (mods.ctrl && (home || end)) {
            return false;
        }
        let Some((caret, kept)) = self.edit(id).map(|e| (e.caret, e.column)) else { return false };
        let Some(text) = self.edits.get(id).map(|e| e.text.as_str()) else { return false };
        let layout = self.entry_layout(id, text, false);
        let here = layout.line_of(caret);
        let Some(line) = layout.lines.get(here) else { return false };
        let column = kept.unwrap_or_else(|| layout.caret(caret).0);
        let at = if home {
            line.range.start
        } else if end {
            if text[line.range.end..].chars().next().is_none_or(|c| c == '\n') { line.range.end } else { line.shown_end }
        } else {
            let wanted = if up { here.checked_sub(1) } else { Some(here + 1).filter(|&n| n < layout.lines.len()) };
            match wanted.and_then(|n| layout.lines.get(n)) {
                Some(to) => layout.hit(column, to.top + to.height / 2.0),
                None if up => 0,
                None => text.len(),
            }
        };
        if let Some(e) = self.edit(id) {
            e.move_to(at, mods.shift);
            e.column = (up || down).then_some(column);
        }
        true
    }

    fn submit_entry(&mut self, id: &str, reply: &mut Reply<M>) {
        if let Some(Some(m)) = self.node(id).map(|n| match &n.what { What::Entry { submit, .. } => submit.clone(), _ => None }) {
            reply.messages.push(m);
            self.eyes_shut(reply);
        }
    }

    pub(super) fn changed(&self, id: &str, before: (usize, u64), reply: &mut Reply<M>) {
        let (Some(e), Some(What::Entry { change, .. })) = (self.edits.get(id), self.node(id).map(|n| &n.what)) else { return };
        if e.stamp() != before {
            reply.messages.push(change(e.text.clone()));
        }
    }

    pub fn paste(&mut self, text: &str) -> Reply<M> {
        self.settle();
        let mut reply = Reply::new();
        let Some(id) = self.focus.clone().filter(|id| self.is_entry(id)) else { return reply };
        let words = self.entry_words(&id, text);
        self.insert_in(&id, &words, &mut reply);
        reply
    }

    pub fn cursor(&self) -> Cursor {
        let drawing = match self.dragging.as_deref() {
            Some(id) if self.is_pointed(id) => Some(id.to_string()),
            Some(_) => None,
            None => self.pointer.and_then(|(x, y)| self.drawing_at(x, y)),
        };
        if let (Some(id), Some((x, y))) = (drawing, self.pointer) {
            if let (Some(spot), Some(What::Drawing { cursor: Some(f), .. })) = (self.spot(&id), self.node(&id).map(|n| &n.what)) {
                let k = self.k();
                return f((x - spot.rect.x) / k, (y - spot.rect.y) / k);
            }
        }
        if let Some(id) = self.dragging.as_deref() {
            if self.picked.as_ref().is_some_and(|(p, _)| p == id) {
                return Cursor::Text;
            }
        } else if self.pointer.is_some_and(|(x, y)| self.pick_at(x, y).is_some()) {
            return Cursor::Text;
        }
        match self.dragging.as_deref().or(self.hover.as_deref()) {
            Some(id) if self.is_divider(id) => self.divider_cursor(id),
            Some(id) if self.is_entry(id) && self.eyes.over.is_none() && self.eyes.down.is_none() => Cursor::Text,
            _ => Cursor::Arrow,
        }
    }

    pub(super) fn move_focus(&mut self, forward: bool) {
        if self.eye_step(forward) {
            return;
        }
        let order: Vec<&str> = self.spots.iter()
            .filter(|s| self.tree.at(&s.path).is_some_and(Node::focusable) && self.tab_stop(s)).map(|s| s.id.as_str()).collect();
        if order.is_empty() {
            return;
        }
        let at = self.focus.as_deref().and_then(|f| order.iter().position(|o| *o == f));
        let next = match (at, forward) {
            (None, true) => 0,
            (None, false) => order.len() - 1,
            (Some(i), true) => (i + 1) % order.len(),
            (Some(i), false) => (i + order.len() - 1) % order.len(),
        };
        let next = order[next].to_string();
        self.focus = Some(next.clone());
        if !forward {
            self.eye_land(&next);
        }
    }

    pub fn held(&mut self, n: u64) -> Reply<M> {
        let mut reply = Reply::new();
        if self.hold.is_some_and(|h| h.2 == n) {
            self.hold = None;
            if let (Some(id), Some((x, y))) = (self.dragging.clone(), self.pointer) {
                self.mark_pressed(&id);
                self.point(&id, x, y, w::Pointer::Menu, &mut reply);
            } else if let Some(((id, row), (x, y))) = self.pointer.and_then(|(x, y)| self.menu_row_at(x, y).map(|r| (r, (x, y)))) {
                self.list_menu(&id, row, x, y, &mut reply);
                self.pressed = None;
                self.last_row = None;
            } else if self.pointer.is_some_and(|(x, y)| self.context_menu_at(x, y)) {
                self.pressed = None;
                self.dragging = None;
                reply.redraw = true;
            }
        }
        reply
    }

    pub(super) fn stands(&self) -> (bool, bool, usize) {
        (self.tip.as_ref().is_some_and(|t| matches!(t.state, TipState::Shown(_))), self.armed.is_some(), self.eyes_open())
    }
}

fn selected_copy(e: &Edit, secret: bool) -> Option<String> {
    Some(e.selected().to_string()).filter(|s| !s.is_empty() && !secret)
}

fn cut_selection<M>(e: &mut Edit, secret: bool, reply: &mut Reply<M>) {
    reply.copy = selected_copy(e, secret);
    if reply.copy.is_some() {
        e.delete_to(e.caret);
    }
}

fn edit_moving<M>(e: &mut Edit, sym: Keysym, mods: Mods, secret: bool, reply: &mut Reply<M>) -> bool {
    let (shift, ctrl) = (mods.shift, mods.ctrl);
    let collapse = !shift && e.caret != e.anchor;
    let word_back = |e: &Edit| if secret { 0 } else { e.prev_word(e.caret) };
    let word_on = |e: &Edit| if secret { e.text.len() } else { e.next_word(e.caret) };
    match edit::facing(sym, if secret { "" } else { &e.text }) {
        Keysym::Left | Keysym::KP_Left => {
            let to = if ctrl { word_back(e) } else if collapse { e.selection().start } else { e.prev(e.caret) };
            e.move_to(to, shift);
        }
        Keysym::Right | Keysym::KP_Right => {
            let to = if ctrl { word_on(e) } else if collapse { e.selection().end } else { e.next(e.caret) };
            e.move_to(to, shift);
        }
        Keysym::Home | Keysym::KP_Home | Keysym::Up | Keysym::KP_Up => e.move_to(0, shift),
        Keysym::End | Keysym::KP_End | Keysym::Down | Keysym::KP_Down => e.move_to(e.text.len(), shift),
        Keysym::BackSpace => {
            let to = if ctrl { word_back(e) } else { e.prev(e.caret) };
            e.delete_to(to);
        }
        Keysym::Delete | Keysym::KP_Delete if shift && !ctrl => cut_selection(e, secret, reply),
        Keysym::Delete | Keysym::KP_Delete => {
            let to = if ctrl { word_on(e) } else { e.next(e.caret) };
            e.delete_to(to);
        }
        _ => return false,
    }
    true
}

fn edit_clipboard<M>(e: &mut Edit, sym: Keysym, mods: Mods, secret: bool, reply: &mut Reply<M>) -> bool {
    let (shift, ctrl) = (mods.shift, mods.ctrl);
    match sym {
        Keysym::Insert | Keysym::KP_Insert if ctrl => reply.copy = selected_copy(e, secret),
        Keysym::Insert | Keysym::KP_Insert if shift => reply.paste = true,
        Keysym::a | Keysym::A if ctrl => e.select(0..e.text.len()),
        Keysym::z if ctrl => {
            e.undo();
        }
        Keysym::Z | Keysym::y | Keysym::Y if ctrl => {
            e.redo();
        }
        Keysym::c | Keysym::C if ctrl => reply.copy = selected_copy(e, secret),
        Keysym::x | Keysym::X if ctrl => cut_selection(e, secret, reply),
        Keysym::v | Keysym::V if ctrl => reply.paste = true,
        _ => return false,
    }
    true
}

fn edit_typing(e: &mut Edit, text: Option<&str>, mods: Mods) -> bool {
    match text {
        Some(t) if !mods.ctrl && !mods.alt && !mods.logo && !t.chars().any(char::is_control) => {
            e.insert(t);
            true
        }
        _ => false,
    }
}
