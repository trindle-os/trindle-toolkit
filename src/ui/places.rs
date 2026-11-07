//! Finding what lies under the pointer, and the cut and unsaid parts.

use super::*;

impl<M: Clone> Ui<M> {
    pub(super) fn keep_places(&mut self, puzzles: Vec<String>) {
        let areas: Vec<String> = self.spots.iter().filter(|s| self.is_scroll(s) || self.is_scroll_x(s)).map(|s| s.id.clone()).collect();
        #[cfg(any(test, feature = "testing"))]
        for (i, a) in areas.iter().enumerate() {
            assert!(!areas[..i].contains(a), "two scrolling areas are named {a}");
        }
        let new = areas.iter().any(|a| !self.areas.contains(a)) || puzzles.iter().any(|p| !self.puzzles.contains(p));
        if new {
            self.scrolls.retain(|id, _| areas.contains(id));
            self.held.keep_only(&puzzles);
            self.areas = areas;
            self.puzzles = puzzles;
        }
    }

    pub(super) fn forget_gone(&mut self) {
        let areas: Vec<String> = self.spots.iter().filter(|s| self.is_scroll(s) || self.is_scroll_x(s)).map(|s| s.id.clone()).collect();
        let puzzles: Vec<String> = puzzle_names(&self.tree).into_values().collect();
        self.scrolls.retain(|id, _| areas.contains(id));
        self.held.keep_only(&puzzles);
        self.areas = areas;
        self.puzzles = puzzles;
    }

    pub(super) fn spot(&self, id: &str) -> Option<&Spot> {
        self.spots.iter().find(|s| s.id == id)
    }

    pub(super) fn node(&self, id: &str) -> Option<&Node<M>> {
        self.tree.at(&self.spot(id)?.path)
    }

    pub(super) fn hit(&self, x: f32, y: f32) -> Option<String> {
        let solid = |s: &Spot| s.shows(x, y) && self.tree.at(&s.path).is_some_and(|n| matches!(n.what, What::Card(..)));
        let above = self.spots.iter().rposition(solid).unwrap_or(0);
        let on_top = || self.spots[above..].iter().rev();
        let found = on_top()
            .find(|s| s.shows(x, y) && (self.tree.at(&s.path).is_some_and(|n| n.focusable() || n.tip().is_some() || (n.off && n.why.is_some()))
                                         || self.cut_short(&s.id).is_some() || self.tells_more(s)
                                         || (self.piece_in(s, x, y).is_some() && !self.in_pressable(s))))
            .map(|s| s.id.clone());
        if found.is_some() || self.touch.is_none() {
            return found;
        }
        let reach = self.px(theme::TOUCH_TARGET) / 2.0;
        on_top().find(|s| {
            let small = self.tree.at(&s.path).is_some_and(|n| n.focusable() && matches!(n.what, What::Button { .. } | What::Switch { .. } | What::Tick { .. }));
            let (cx, cy) = (s.rect.x + s.rect.w / 2.0, s.rect.y + s.rect.h / 2.0);
            small && (x - cx).abs() <= reach.max(s.rect.w / 2.0) && (y - cy).abs() <= reach.max(s.rect.h / 2.0)
                && s.clip.is_none_or(|c| c.contains(x, y))
        }).map(|s| s.id.clone())
    }

    pub(super) fn note_cut(&self, id: &str, part: Option<usize>, whole: &str, shown: &Layout) {
        if !shown.cut {
            return;
        }
        let mut cuts = self.cuts.borrow_mut();
        let said = cuts.entry((id.to_string(), part)).or_default();
        if !said.split('\n').any(|line| line == whole) {
            if !said.is_empty() {
                said.push('\n');
            }
            said.push_str(whole);
        }
    }

    pub(super) fn part_at(&self, id: &str) -> Option<usize> {
        let (x, y) = self.pointer?;
        match self.node(id).map(|n| &n.what) {
            Some(What::Choice { .. }) => self.pill(id, x),
            Some(What::List { .. }) => self.row_at(id, y),
            Some(What::Drawing { .. }) => self.tool_at(id).or_else(|| self.spot(id).and_then(|s| self.piece_in(s, x, y))),
            _ => None,
        }
    }

    pub(super) fn tool_at(&self, id: &str) -> Option<usize> {
        let (x, y) = self.pointer?;
        let spot = self.spot(id)?;
        let Some(crate::role::Role::Toolbar { tools, .. }) = self.node(id)?.says.as_deref()?.role.as_ref() else { return None };
        let along = (x - spot.rect.x) / self.k();
        tools.iter().position(|&(_, a, w)| spot.shows(x, y) && along >= a && along < a + w)
    }

    pub(super) fn piece_in(&self, spot: &Spot, x: f32, y: f32) -> Option<usize> {
        if !spot.shows(x, y) {
            return None;
        }
        self.said.borrow().get(&spot.id)?.iter().position(|(r, _)| {
            let lift = r.h / 4.0;
            Rect { x: spot.rect.x + r.x, y: spot.rect.y + r.y - lift, w: r.w, h: r.h + 2.0 * lift }.contains(x, y)
        })
    }

    pub(super) fn in_pressable(&self, spot: &Spot) -> bool {
        (0..spot.path.len()).any(|k| self.tree.at(&spot.path[..k]).is_some_and(|a| !a.off && matches!(a.what, What::Card(_, Some(_)))))
    }

    pub(super) fn tells_more(&self, spot: &Spot) -> bool {
        let Some(node) = self.tree.at(&spot.path) else { return false };
        let Some(whole) = node.whole.as_deref() else { return false };
        let (cuts, said) = (self.cuts.borrow(), self.said.borrow());
        let cut = self.spots.iter().filter(|s| s.path.starts_with(&spot.path)).any(|s| {
            said.get(&s.id).is_some_and(|pieces| !pieces.is_empty()) || cuts.keys().any(|(id, _)| *id == s.id) || self.cut_short(&s.id).is_some()
        });
        cut || node.shown_words().trim() != whole.trim()
    }

    pub(super) fn whole_here(&self, id: &str) -> Option<String> {
        let spot = self.spot(id)?;
        if !self.tells_more(spot) {
            return None;
        }
        self.tree.at(&spot.path)?.whole.clone()
    }

    pub(super) fn said_here(&self, id: &str) -> Option<String> {
        let (x, y) = self.pointer?;
        let spot = self.spot(id)?;
        let said = self.said.borrow();
        self.spots.iter().filter(|s| s.path.starts_with(&spot.path))
            .find_map(|s| self.piece_in(s, x, y).and_then(|i| said.get(&s.id)?.get(i).map(|(_, whole)| whole.to_string())))
    }

    pub(super) fn tells_whole(&self) -> bool {
        let Some(t) = &self.tip else { return false };
        self.eyes.over.as_deref() != Some(t.id.as_str()) && !self.is_armed(&t.id)
            && (self.spot(&t.id).is_some_and(|s| self.tells_more(s)) || self.cut_here(&t.id).is_some())
    }

    pub(super) fn part_rect(&self, id: &str, part: usize) -> Option<Rect> {
        let spot = self.spot(id)?;
        match &self.node(id)?.what {
            What::List { .. } => {
                let row = self.row_of(id);
                Some(Rect { y: spot.rect.y + part as f32 * row, h: row, ..spot.rect })
            }
            What::Drawing { .. } => {
                if let Some(crate::role::Role::Toolbar { tools, .. }) = self.node(id)?.says.as_deref()?.role.as_ref() {
                    let (_, a, wide) = tools.get(part)?;
                    return Some(Rect { x: spot.rect.x + a * self.k(), w: wide * self.k(), ..spot.rect });
                }
                let r = self.said.borrow().get(id)?.get(part)?.0;
                Some(Rect { x: spot.rect.x + r.x, y: spot.rect.y + r.y, ..r })
            }
            What::Choice { options, arrows, .. } => {
                let pad = self.px(theme::sheet().tabs_padding);
                let subs = arrows.as_ref().map(|a| a.subs.as_slice());
                let widths = w::pills_in(w::pill_widths(&self.fonts, &self.texts, options, subs, self.scale), spot.rect.w - 2.0 * pad);
                let before: f32 = widths.iter().take(part).sum();
                Some(Rect { x: spot.rect.x + pad + before, w: *widths.get(part)?, ..spot.rect })
            }
            _ => None,
        }
    }

    #[cfg(any(test, feature = "testing"))]
    pub fn label_over(&mut self, x: f32, y: f32) -> Option<String> {
        self.pointer = Some((x, y));
        self.hit(x, y).and_then(|id| self.label_of(&id))
    }

    #[cfg(any(test, feature = "testing"))]
    pub fn pressable_at(&self, x: f32, y: f32) -> bool {
        self.hit(x, y).is_some_and(|id| matches!(self.node(&id).map(|n| &n.what), Some(What::Card(_, Some(_)))))
    }

    #[cfg(any(test, feature = "testing"))]
    pub fn cut_unsaid(&mut self) -> (usize, Vec<(String, Option<String>)>) {
        let mut pieces: Vec<(String, f32, f32)> = Vec::new();
        for (id, said) in self.said.borrow().iter() {
            let Some(spot) = self.spot(id) else { continue };
            for (r, whole) in said {
                let (x, y) = (spot.rect.x + r.x + r.w / 2.0, spot.rect.y + r.y + r.h / 2.0);
                if spot.shows(x, y) {
                    pieces.push((whole.to_string(), x, y));
                }
            }
        }
        for ((id, part), whole) in self.cuts.borrow().iter() {
            let at = part.and_then(|p| self.part_rect(id, p)).or_else(|| self.spot(id).map(|s| s.rect));
            if let Some(r) = at {
                let (x, y) = (r.x + r.w / 2.0, r.y + r.h / 2.0);
                if self.spot(id).is_some_and(|s| s.shows(x, y)) {
                    pieces.push((whole.clone(), x, y));
                }
            }
        }
        let mut unsaid = Vec::new();
        let all = pieces.len();
        for (whole, x, y) in pieces {
            if x < 0.0 || y < 0.0 || x >= self.size.0 as f32 || y >= self.size.1 as f32 {
                continue;
            }
            self.pointer = Some((x, y));
            let label = self.hit(x, y).and_then(|id| self.label_of(&id));
            if !label.as_ref().is_some_and(|l| whole.split('\n').all(|line| l.contains(line))) {
                unsaid.push((whole, label));
            }
        }
        (all, unsaid)
    }

    pub(super) fn cut_here(&self, id: &str) -> Option<String> {
        if let Some(words) = self.cut_short(id) {
            return Some(words.to_string());
        }
        let noted = self.cuts.borrow().get(&(id.to_string(), self.part_at(id))).cloned();
        noted.or_else(|| self.said_here(id))
    }

    pub(super) fn cut_short(&self, id: &str) -> Option<&str> {
        let spot = self.spot(id)?;
        let node = self.node(id).filter(|n| n.cut > 0)?;
        let What::Label { text, words, .. } = &node.what else { return None };
        if node.cut > 1 {
            let wrapped = self.texts.get(&self.fonts, text, &words.style(self.scale), Some(spot.rect.w.max(1.0)));
            return (wrapped.lines.len() > node.cut as usize).then_some(text.as_str());
        }
        let full = self.texts.get(&self.fonts, text, &words.style(self.scale), None);
        (full.width > spot.rect.w + 0.5).then_some(text.as_str())
    }

    pub(super) fn top_drawing(&self, x: f32, y: f32) -> Option<&Spot> {
        let see_through = |s: &&Spot| matches!(self.tree.at(&s.path).map(|n| &n.what), Some(What::Layers(_) | What::Placed { .. } | What::Drawing { passes: true, .. }));
        self.spots.iter().rev().find(|s| s.shows(x, y) && !see_through(s))
    }

    pub(super) fn drawing_at(&self, x: f32, y: f32) -> Option<String> {
        let top = self.top_drawing(x, y)?;
        self.is_pointed(&top.id).then(|| top.id.clone())
    }

    pub(super) fn scrolled_drawing(&self, x: f32, y: f32, lines: f32) -> Option<M> {
        let top = self.top_drawing(x, y)?;
        match self.node(&top.id).map(|n| &n.what) {
            Some(What::Drawing { scroll: Some(f), .. }) => Some(f(lines)),
            _ => None,
        }
    }

    pub(super) fn keys_of(&self, id: &str) -> Option<&dyn Fn(&Input) -> Option<M>> {
        match self.node(id).map(|n| &n.what) {
            Some(What::Drawing { keys: Some(f), .. }) => Some(f.as_ref()),
            _ => None,
        }
    }

    pub(super) fn to_keyed(&self, input: &Input, reply: &mut Reply<M>) -> bool {
        let Some(id) = self.focus.as_deref() else { return false };
        let Some(f) = self.keys_of(id) else { return false };
        match f(input) {
            Some(m) => {
                self.mark_pressed(id);
                reply.messages.push(m);
                reply.redraw = true;
                true
            }
            None => false,
        }
    }

    pub(super) fn tell_focus(&self, before: Option<&str>, reply: &mut Reply<M>) {
        if before == self.focus.as_deref() {
            return;
        }
        for (id, here) in [(before, false), (self.focus.as_deref(), true)] {
            if let Some(m) = id.and_then(|id| self.keys_of(id)).and_then(|f| f(&Input::Focus(here))) {
                reply.messages.push(m);
            }
        }
    }

    pub(super) fn is_pointed(&self, id: &str) -> bool {
        matches!(self.node(id).map(|n| &n.what), Some(What::Drawing { pointer: Some(_), .. }))
    }

    pub(super) fn point(&self, id: &str, x: f32, y: f32, make: fn(f32, f32) -> w::Pointer, reply: &mut Reply<M>) {
        let (Some(spot), Some(What::Drawing { pointer: Some(f), .. })) = (self.spot(id), self.node(id).map(|n| &n.what)) else { return };
        let k = self.k();
        reply.messages.extend(f(make((x - spot.rect.x) / k, (y - spot.rect.y) / k), self.mods));
    }

    pub(super) fn right_drags(&self, id: &str) -> bool {
        matches!(self.node(id).map(|n| &n.what), Some(What::Drawing { pointer: Some(_), right_drags: true, .. }))
    }

    pub(super) fn takes_drops(&self, id: &str) -> bool {
        matches!(self.node(id).map(|n| &n.what), Some(What::List { landed: Some(_), .. } | What::Choice { landed: Some(_), .. }))
    }

    fn takes_drop_at(&self, id: &str, x: f32, y: f32) -> bool {
        let says = |on: Option<usize>, drop_on: &Option<Box<dyn Fn(usize) -> bool>>| on.zip(drop_on.as_ref()).is_none_or(|(i, takes)| takes(i));
        match self.node(id).map(|n| &n.what) {
            Some(What::List { landed: Some(_), drop_on, .. }) => says(self.row_at(id, y), drop_on),
            Some(What::Choice { landed: Some(_), drop_on, .. }) => says(self.pill(id, x), drop_on),
            _ => false,
        }
    }

    pub(super) fn held_over(&mut self, id: &str, x: f32, y: f32, reply: &mut Reply<M>) {
        let over = self.drawing_at(x, y).filter(|o| o != id && self.hovers(o));
        self.hover_drawing(over.clone(), reply);
        if let Some(o) = over {
            self.point(&o, x, y, w::Pointer::Hover, reply);
        }
        let onto = self.hit(x, y).filter(|h| self.takes_drop_at(h, x, y));
        self.drop_hover(onto, reply);
        let gap = self.hit(x, y).filter(|h| matches!(self.node(h).map(|n| &n.what), Some(What::List { between: Some(_), .. })))
            .and_then(|h| self.gap_at(&h, y, false).map(|g| (h, g)));
        if gap != self.gap {
            self.gap = gap;
            reply.redraw = true;
        }
    }

    pub(super) fn gap_at(&self, id: &str, y: f32, whole: bool) -> Option<usize> {
        let (Some(s), Some(What::List { rows, row, .. })) = (self.spot(id), self.node(id).map(|n| &n.what)) else { return None };
        let h = self.px(*row);
        if h <= 0.0 {
            return None;
        }
        let at = (y - s.rect.y) / h;
        let n = rows.len();
        if whole {
            return Some((at.round().max(0.0) as usize).min(n));
        }
        if at < 0.0 || at > n as f32 {
            return None;
        }
        let part = at - at.floor();
        let gap = if part < 0.25 { at.floor() } else if part > 0.75 { at.floor() + 1.0 } else { return None };
        Some((gap as usize).min(n))
    }

    pub(super) fn paint_gap(&self, p: &mut Painter, spot: &Spot, rows: usize) {
        let Some((_, gap)) = self.gap.as_ref().filter(|(id, _)| *id == spot.id) else { return };
        let (row, thick) = (self.row_of(&spot.id), self.px(theme::sheet().focus_width).max(2.0));
        let r = spot.rect;
        let y = (r.y + (*gap).min(rows) as f32 * row - thick / 2.0).clamp(r.y, (r.y + r.h - thick).max(r.y));
        fill_on(p, Rect { x: r.x, y: y.round(), w: r.w, h: thick }, thick / 2.0, self.theme.accent);
    }

    pub(super) fn drop_hover(&mut self, onto: Option<String>, reply: &mut Reply<M>) {
        let was = self.hover.clone().filter(|h| self.takes_drops(h));
        if onto.is_some() || was.is_some() {
            if onto.is_some() || self.hover == was {
                self.hover = onto;
            }
            reply.redraw = true;
        }
    }

    pub(super) fn landed(&mut self, x: f32, y: f32, reply: &mut Reply<M>) {
        if let Some((id, gap)) = self.gap.take() {
            if let Some(What::List { between: Some(f), .. }) = self.node(&id).map(|n| &n.what) {
                reply.messages.push(f(gap));
            }
            return;
        }
        let Some(id) = self.hit(x, y).filter(|h| self.takes_drop_at(h, x, y)) else { return };
        match self.node(&id).map(|n| &n.what) {
            Some(What::List { landed: Some(f), .. }) => {
                if let Some(row) = self.row_at(&id, y) {
                    reply.messages.push(f(row));
                }
            }
            Some(What::Choice { landed: Some(f), .. }) => {
                if let Some(i) = self.pill(&id, x) {
                    reply.messages.push(f(i));
                }
            }
            _ => {}
        }
    }

    pub(super) fn hovers(&self, id: &str) -> bool {
        matches!(self.node(id).map(|n| &n.what), Some(What::Drawing { hovers: true, .. }))
    }

    pub(super) fn hover_drawing(&mut self, now: Option<String>, reply: &mut Reply<M>) {
        if self.hovered == now {
            return;
        }
        if let Some(old) = std::mem::replace(&mut self.hovered, now) {
            if let Some(What::Drawing { pointer: Some(f), .. }) = self.node(&old).map(|n| &n.what) {
                reply.messages.extend(f(w::Pointer::Leave, self.mods));
            }
        }
    }

    pub(super) fn is_scroll(&self, s: &Spot) -> bool {
        matches!(self.tree.at(&s.path).map(|n| &n.what), Some(What::Scroll(_)))
    }

    pub(super) fn is_scroll_x(&self, s: &Spot) -> bool {
        matches!(self.tree.at(&s.path).map(|n| &n.what), Some(What::ScrollX(_)))
    }

    pub(super) fn is_slide(&self, s: &Spot) -> bool {
        matches!(self.tree.at(&s.path).map(|n| &n.what), Some(What::Slide { .. }))
    }

    pub(super) fn wheel_slide(&self, x: f32, y: f32, by: f32, steps: bool, reply: &mut Reply<M>) {
        let found = self.spots.iter().rev().filter(|s| s.shows(x, y)).find_map(|s| match self.tree.at(&s.path).map(|n| &n.what) {
            Some(What::Slide { wheel: Some(f), .. }) => Some((s.rect.w, f)),
            _ => None,
        });
        let Some((w, f)) = found else { return };
        let along = if steps { by * notch(w, self.k()) } else { by };
        reply.messages.push(f(along));
    }
}
