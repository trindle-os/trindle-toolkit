//! A dropdown's open list, kept and drawn by the toolkit; it floats and takes no room in the layout.

use super::*;
use unicode_segmentation::UnicodeSegmentation;

const GAP: f32 = 4.0;
const EDGE: f32 = 4.0;
const SEARCH_LEAST: f32 = 220.0;
const SEARCH_MOST: usize = 100;
fn search() -> &'static str {
    crate::t!("kit-search")
}
fn nothing() -> &'static str {
    crate::t!("kit-nothing-matches")
}

pub(super) struct DropOpen {
    pub(super) id: String,
    pub(super) part: Option<usize>,
    folded: Vec<String>,
    shown: u64,
    dead: bool,
    pub(super) search: String,
    pub(super) lit: Option<usize>,
    pub(super) top: f32,
    pub(super) up: bool,
    pub(super) searching: bool,
    wide: f32,
    thumb: Option<f32>,
    down: bool,
    ahead: ahead::Typed,
    finger: Option<(f32, f32, bool)>,
}

pub(super) struct DropLaid {
    pub(super) panel: Rect,
    pub(super) search: Option<Rect>,
    pub(super) rows: Rect,
    pub(super) row: f32,
    pub(super) shown: Vec<usize>,
}

impl DropLaid {
    fn full(&self) -> f32 {
        self.shown.len() as f32 * self.row
    }

    fn most_top(&self) -> f32 {
        (self.full() - self.rows.h).max(0.0)
    }
}

pub(super) struct Source<'a> {
    pub(super) options: &'a [String],
    chosen: Option<usize>,
    dead: bool,
    pub(super) anchor: Rect,
}

fn in_short(options: &[String]) -> u64 {
    use std::hash::{Hash, Hasher};
    let mut h = std::collections::hash_map::DefaultHasher::new();
    options.hash(&mut h);
    h.finish()
}

pub(crate) fn folded(options: &[String]) -> Vec<String> {
    options.iter().map(|o| o.to_lowercase()).collect()
}

pub(crate) fn narrowed(folded: &[String], typed: &str) -> Vec<usize> {
    let wanted = typed.trim().to_lowercase();
    (0..folded.len()).filter(|&i| wanted.is_empty() || folded[i].contains(&wanted)).collect()
}

pub(crate) fn opens_up(button: Rect, tall: f32, room: f32, gap: f32, edge: f32) -> bool {
    let below = room - edge - (button.y + button.h + gap);
    let above = button.y - gap - edge;
    tall > below && above > below
}

impl<M: Clone> Ui<M> {
    pub fn dropdown_open(&self) -> bool {
        self.drop.is_some()
    }

    pub(super) fn is_dropdown(&self, id: &str) -> bool {
        matches!(self.node(id).map(|n| &n.what), Some(What::Dropdown { .. }))
    }

    pub(super) fn close_drop(&mut self) -> bool {
        self.drop.take().is_some()
    }

    pub(super) fn keep_drop(&mut self) {
        let Some(d) = &self.drop else { return };
        let (id, part, was) = (d.id.clone(), d.part, (d.shown, d.dead));
        let shows = self.spot(&id).is_some_and(|s| s.clip.is_none_or(|c| c.intersect(&s.rect).h > 0.0))
            && self.node(&id).is_some_and(|n| !n.off);
        let now = self.source_of(&id, part).filter(|s| shows && !s.options.is_empty()).map(|s| (if part.is_some() { in_short(s.options) } else { was.0 }, s.dead));
        match now {
            None => self.drop = None,
            Some(now) if part.is_some() && now != was => self.open_drop_at(&id, part),
            Some(_) => {}
        }
    }

    pub(super) fn source_of(&self, id: &str, part: Option<usize>) -> Option<Source<'_>> {
        match (&self.node(id)?.what, part) {
            (What::Dropdown { options, chosen, .. }, None) => Some(Source { options, chosen: *chosen, dead: false, anchor: self.spot(id)?.rect }),
            (What::Choice { arrows: Some(a), .. }, Some(i)) => {
                let anchor = *self.pill_rects(id).get(i)?;
                match a.subs.get(i)? {
                    w::Sub::Rows(rows, marked) if !rows.is_empty() => {
                        Some(Source { options: rows.as_slice(), chosen: marked.filter(|m| *m < rows.len()), dead: false, anchor })
                    }
                    w::Sub::Words(words) => Some(Source { options: std::slice::from_ref(words), chosen: None, dead: true, anchor }),
                    _ => None,
                }
            }
            _ => None,
        }
    }

    pub(super) fn pill_rects(&self, id: &str) -> Vec<Rect> {
        let (Some(spot), Some(What::Choice { options, arrows, .. })) = (self.spot(id), self.node(id).map(|n| &n.what)) else { return Vec::new() };
        let inner = spot.rect.inset(self.px(theme::sheet().tabs_padding));
        let subs = arrows.as_ref().map(|a| a.subs.as_slice());
        let mut x = inner.x;
        w::pills_in(w::pill_widths(&self.fonts, &self.texts, options, subs, self.scale), inner.w).into_iter().map(|wide| {
            let r = Rect { x, y: inner.y, w: wide, h: inner.h };
            x += wide;
            r
        }).collect()
    }

    pub(super) fn arrow_at(&self, id: &str, x: f32) -> Option<usize> {
        let Some(What::Choice { arrows: Some(a), .. }) = self.node(id).map(|n| &n.what) else { return None };
        let reach = self.px(theme::sheet().entry_padding + w::ARROW + w::ARROW_GAP);
        self.pill_rects(id).iter().enumerate()
            .find(|(i, r)| x >= r.x + r.w - reach && x < r.x + r.w && w::has_arrow(&a.subs, *i))
            .map(|(i, _)| i)
    }

    pub(super) fn open_sub(&mut self, id: &str, i: usize, reply: &mut Reply<M>) -> bool {
        let Some(What::Choice { arrows: Some(a), .. }) = self.node(id).filter(|n| !n.off).map(|n| &n.what) else { return false };
        if !w::has_arrow(&a.subs, i) {
            return false;
        }
        reply.messages.push((a.opening)(i));
        self.open_drop_at(id, Some(i));
        reply.redraw = true;
        self.drop.is_some()
    }

    pub(super) fn along_key(&mut self, id: &str, sym: Keysym, reply: &mut Reply<M>) -> bool {
        let Some(What::Choice { options, chosen, arrows: Some(_), .. }) = self.node(id).map(|n| &n.what) else { return false };
        let (last, chosen) = (options.len().saturating_sub(1), *chosen);
        let at = self.along.as_ref().filter(|(a, _)| a == id).map_or(chosen, |(_, i)| (*i).min(last));
        let to = match sym {
            Keysym::Left | Keysym::KP_Left => at.saturating_sub(1),
            Keysym::Right | Keysym::KP_Right => (at + 1).min(last),
            Keysym::Home | Keysym::KP_Home => 0,
            Keysym::End | Keysym::KP_End => last,
            Keysym::Down | Keysym::KP_Down | Keysym::Return | Keysym::KP_Enter | Keysym::space | Keysym::KP_Space => {
                self.along = Some((id.to_string(), at));
                let opens = !matches!(sym, Keysym::space | Keysym::KP_Space);
                if !(opens && self.open_sub(id, at, reply)) && at != chosen {
                    if let Some(What::Choice { choose, .. }) = self.node(id).map(|n| &n.what) {
                        reply.messages.push(choose(at));
                    }
                }
                reply.redraw = true;
                return true;
            }
            _ => return false,
        };
        self.news = options.get(to).cloned().unwrap_or_default();
        self.along = Some((id.to_string(), to));
        reply.redraw = true;
        true
    }

    fn drop_room(&self, b: Rect, up: bool) -> f32 {
        let (gap, edge) = (self.px(GAP), self.px(EDGE));
        if up { b.y - gap - edge } else { self.size.1 as f32 - edge - (b.y + b.h + gap) }
    }

    fn drop_bar_room(&self) -> f32 {
        self.px(theme::sheet().scrollbar_hover) + 2.0 * self.px(theme::sheet().scrollbar_inset)
    }

    pub(super) fn open_drop(&mut self, id: &str) {
        self.open_drop_at(id, None);
    }

    pub(super) fn open_drop_at(&mut self, id: &str, part: Option<usize>) {
        if self.node(id).is_none_or(|n| n.off) {
            return;
        }
        let Some(src) = self.source_of(id, part) else { return };
        if src.options.is_empty() {
            return;
        }
        let sheet = theme::sheet();
        let (b, count, dead, shown) = (src.anchor, src.options.len(), src.dead, in_short(src.options));
        let style = Words::Plain.style(self.scale);
        let longest = src.options.iter().map(|o| self.texts.get(&self.fonts, o, &style, None).width).fold(0.0, f32::max).ceil();
        let lit = src.chosen.filter(|_| !dead);
        let (pad, row) = (self.px(sheet.menu_padding), self.px(sheet.dropdown_row));
        let most = sheet.dropdown_rows as usize;
        let long = count > most;
        let search_h = self.px(sheet.entry_height) + pad;
        let tall = 2.0 * pad + count.min(most) as f32 * row + if long { search_h } else { 0.0 };
        let up = opens_up(b, tall, self.size.1 as f32, self.px(GAP), self.px(EDGE));
        let fit = (((self.drop_room(b, up) - 2.0 * pad) / row).floor().max(1.0)) as usize;
        let searching = count > most.min(fit);
        let mut wide = longest + 2.0 * self.px(sheet.entry_padding) + 2.0 * pad;
        if searching {
            wide = (wide + self.drop_bar_room()).max(self.px(SEARCH_LEAST));
        }
        let wide = wide.min(self.px(sheet.dropdown_width));
        self.drop = Some(DropOpen { id: id.to_string(), part, folded: folded(src.options), shown, dead, search: String::new(), lit, top: 0.0, up, searching, wide,
                                    thumb: None, down: false, ahead: ahead::Typed::default(), finger: None });
        self.focus = Some(id.to_string());
        self.tip = None;
        if let (Some(l), Some(at)) = (self.drop_laid(), lit) {
            let rows = (l.rows.h / l.row).floor().max(1.0);
            let top = ((at as f32 - ((rows - 1.0) / 2.0).floor()) * l.row).clamp(0.0, l.most_top());
            if let Some(d) = &mut self.drop {
                d.top = top;
            }
        }
        self.say_lit();
    }

    pub(super) fn drop_laid(&self) -> Option<DropLaid> {
        let d = self.drop.as_ref()?;
        let src = self.source_of(&d.id, d.part)?;
        let (b, options) = (src.anchor, src.options);
        let sheet = theme::sheet();
        let (pad, gap, edge, row) = (self.px(sheet.menu_padding), self.px(GAP), self.px(EDGE), self.px(sheet.dropdown_row));
        let shown = if d.searching { narrowed(&d.folded, &d.search) } else { (0..options.len()).collect() };
        let search_h = if d.searching { self.px(sheet.entry_height) + pad } else { 0.0 };
        let chrome = 2.0 * pad + search_h;
        let fit = (((self.drop_room(b, d.up) - chrome) / row).floor().max(1.0)) as usize;
        let n = shown.len().clamp(1, (sheet.dropdown_rows as usize).min(fit).max(1));
        let rows_h = n as f32 * row;
        let h = chrome + rows_h;
        let ww = self.size.0 as f32;
        let w = d.wide.max(b.w).min((ww - 2.0 * edge).max(1.0));
        let x = b.x.min(ww - edge - w).max(edge);
        let y = if d.up { b.y - gap - h } else { b.y + b.h + gap };
        let panel = Rect { x: x.round(), y: y.round(), w: w.round(), h };
        let inner = Rect { x: panel.x + pad, y: panel.y + pad, w: panel.w - 2.0 * pad, h: self.px(sheet.entry_height) };
        let search = d.searching.then_some(inner);
        let rows = Rect { y: panel.y + pad + search_h, h: rows_h, ..inner };
        Some(DropLaid { panel, search, rows, row, shown })
    }

    fn drop_bar(&self, l: &DropLaid, top: f32, thick: f32) -> Option<(Rect, Rect)> {
        if l.full() <= l.rows.h + 0.5 {
            return None;
        }
        let inset = self.px(theme::sheet().scrollbar_inset);
        let r = l.rows;
        let track = Rect { x: r.x + r.w - inset - thick, y: r.y + inset, w: thick, h: (r.h - 2.0 * inset).max(0.0) };
        let th = (track.h * r.h / l.full()).max(self.px(theme::SCROLLBAR_MIN)).min(track.h);
        let y = track.y + (track.h - th) * top.clamp(0.0, l.most_top()) / l.most_top().max(1.0);
        Some((track, Rect { y, h: th, ..track }))
    }

    fn on_drop_bar(&self, l: &DropLaid, x: f32, y: f32) -> bool {
        l.full() > l.rows.h + 0.5 && l.rows.contains(x, y) && x >= l.rows.x + l.rows.w - self.drop_bar_room()
    }

    fn drop_row_at(&self, l: &DropLaid, x: f32, y: f32) -> Option<usize> {
        let top = self.drop.as_ref()?.top;
        if !l.rows.contains(x, y) || self.on_drop_bar(l, x, y) {
            return None;
        }
        l.shown.get(((y - l.rows.y + top) / l.row).floor().max(0.0) as usize).copied()
    }

    fn scroll_drop(&mut self, to: f32) {
        let Some(l) = self.drop_laid() else { return };
        if let Some(d) = &mut self.drop {
            d.top = to.clamp(0.0, l.most_top());
        }
    }

    fn reveal_lit(&mut self) {
        let Some(l) = self.drop_laid() else { return };
        let Some(d) = &mut self.drop else { return };
        d.top = d.top.clamp(0.0, l.most_top());
        let Some(at) = d.lit.and_then(|i| l.shown.iter().position(|&s| s == i)) else { return };
        let y = at as f32 * l.row;
        if y < d.top {
            d.top = y;
        } else if y + l.row > d.top + l.rows.h {
            d.top = y + l.row - l.rows.h;
        }
    }

    fn say_lit(&mut self) {
        let Some(d) = &self.drop else { return };
        let Some(src) = self.source_of(&d.id, d.part) else { return };
        let options = src.options;
        let row = d.lit.or(src.dead.then_some(0)).and_then(|i| options.get(i)).cloned().unwrap_or_default();
        self.news = if d.searching && !d.search.is_empty() {
            let n = narrowed(&d.folded, &d.search).len();
            if n == 0 { nothing().to_string() } else { crate::t!("kit-search-count", row = &row, n = n, total = options.len()) }
        } else {
            row
        };
    }

    fn step_lit(&mut self, by: i32) {
        let Some(l) = self.drop_laid() else { return };
        let Some(d) = &mut self.drop else { return };
        if l.shown.is_empty() {
            d.lit = None;
            return;
        }
        let last = l.shown.len() as i32 - 1;
        let at = d.lit.and_then(|i| l.shown.iter().position(|&s| s == i)).map(|p| p as i32);
        let to = match at {
            Some(p) => (p + by).clamp(0, last),
            None if by > 0 => 0,
            None => last,
        };
        d.lit = Some(l.shown[to as usize]);
        self.reveal_lit();
        self.say_lit();
    }

    pub(super) fn pick_drop(&mut self, i: usize, reply: &mut Reply<M>) {
        let Some(d) = self.drop.take() else { return };
        match (self.node(&d.id).map(|n| &n.what), d.part) {
            (Some(What::Dropdown { options, chosen, choose, .. }), None) => {
                if i < options.len() && *chosen != Some(i) {
                    reply.messages.push(choose(i));
                }
            }
            (Some(What::Choice { arrows: Some(a), .. }), Some(part)) => {
                if let Some(w::Sub::Rows(rows, _)) = a.subs.get(part) {
                    if i < rows.len() {
                        reply.messages.push((a.pick)(part, i));
                    }
                }
            }
            _ => {}
        }
        self.news = String::new();
        reply.redraw = true;
    }

    fn narrow_drop(&mut self) {
        let Some(d) = &self.drop else { return };
        let Some(src) = self.source_of(&d.id, d.part) else { return };
        let shown = narrowed(&d.folded, &d.search);
        let lit = src.chosen.filter(|c| shown.contains(c) && d.search.is_empty()).or(shown.first().copied()).filter(|_| !src.dead);
        if let Some(d) = &mut self.drop {
            d.lit = lit;
            d.top = 0.0;
        }
        self.reveal_lit();
        self.say_lit();
    }

    fn drop_typed(&mut self, text: &str, now: Instant) {
        let Some(d) = &mut self.drop else { return };
        if !d.searching {
            self.drop_ahead(text, now);
            return;
        }
        if d.search.chars().count() + text.chars().count() <= SEARCH_MOST {
            d.search.push_str(text);
        }
        self.narrow_drop();
    }

    fn drop_ahead(&mut self, text: &str, now: Instant) {
        let Some(d) = &self.drop else { return };
        let Some(src) = self.source_of(&d.id, d.part).filter(|s| !s.dead) else { return };
        let options = src.options;
        let mut typed = d.ahead.clone();
        if !typed.takes(text, now) {
            return;
        }
        typed.add(text, now);
        let found = options.iter().position(|o| ahead::fold(o).starts_with(&typed.letters))
            .or_else(|| options.iter().position(|o| ahead::fold(o).contains(&typed.letters)));
        if let Some(d) = &mut self.drop {
            d.ahead = typed;
            if found.is_some() {
                d.lit = found;
            }
        }
        self.reveal_lit();
        self.say_lit();
    }

    fn drop_move(&mut self, l: &DropLaid, x: f32, y: f32) {
        self.pointer = Some((x, y));
        self.hover = None;
        self.tip = None;
        if let Some(grab) = self.drop.as_ref().and_then(|d| d.thumb) {
            if let Some((track, thumb)) = self.drop_bar(l, 0.0, self.px(theme::sheet().scrollbar_hover)) {
                let room = track.h - thumb.h;
                let t = if room > 0.0 { ((y - grab - track.y) / room).clamp(0.0, 1.0) } else { 0.0 };
                self.scroll_drop(t * l.most_top());
            }
            return;
        }
        if let Some(row) = self.drop_row_at(l, x, y) {
            if let Some(d) = &mut self.drop {
                d.lit = Some(row);
            }
        }
    }

    fn drop_press(&mut self, l: &DropLaid, x: f32, y: f32) {
        let top = self.drop.as_ref().map_or(0.0, |d| d.top);
        if self.on_drop_bar(l, x, y) {
            if let Some((_, thumb)) = self.drop_bar(l, top, self.px(theme::sheet().scrollbar_hover)) {
                if y >= thumb.y && y < thumb.y + thumb.h {
                    if let Some(d) = &mut self.drop {
                        d.thumb = Some(y - thumb.y);
                    }
                } else {
                    let page = l.rows.h * if y < thumb.y { -1.0 } else { 1.0 };
                    self.scroll_drop(top + page);
                }
            }
            return;
        }
        let row = self.drop_row_at(l, x, y);
        if let Some(d) = &mut self.drop {
            d.down = true;
            if row.is_some() {
                d.lit = row;
            }
        }
    }

    fn drop_scroll(&mut self, l: &DropLaid, y: f32, steps: bool, k: f32) {
        let by = if steps { y * 2.0 * l.row } else { y * k };
        let top = self.drop.as_ref().map_or(0.0, |d| d.top);
        self.scroll_drop(top + by);
        if let Some((px, py)) = self.pointer {
            let Some(l) = self.drop_laid() else { return };
            if let Some(row) = self.drop_row_at(&l, px, py) {
                if let Some(d) = &mut self.drop {
                    d.lit = Some(row);
                }
            }
        }
    }

    fn drop_key(&mut self, l: &DropLaid, sym: Keysym, text: Option<&str>, mods: Mods, now: Instant, reply: &mut Reply<M>) -> bool {
        let page = (l.rows.h / l.row).floor().max(1.0) as i32;
        let searching = self.drop.as_ref().is_some_and(|d| d.searching);
        match sym {
            Keysym::Escape => {
                self.close_drop();
                self.news = String::new();
            }
            Keysym::Tab | Keysym::ISO_Left_Tab => {
                self.close_drop();
                return false;
            }
            Keysym::Up | Keysym::KP_Up if mods.alt => {
                self.close_drop();
            }
            Keysym::Up | Keysym::KP_Up => self.step_lit(-1),
            Keysym::Down | Keysym::KP_Down => self.step_lit(1),
            Keysym::Page_Up | Keysym::KP_Page_Up => self.step_lit(-page),
            Keysym::Page_Down | Keysym::KP_Page_Down => self.step_lit(page),
            Keysym::Home | Keysym::KP_Home => self.step_lit(-(l.shown.len() as i32)),
            Keysym::End | Keysym::KP_End => self.step_lit(l.shown.len() as i32),
            Keysym::Return | Keysym::KP_Enter => {
                match self.drop.as_ref().and_then(|d| d.lit).filter(|i| l.shown.contains(i)) {
                    Some(i) => self.pick_drop(i, reply),
                    None => {
                        self.close_drop();
                    }
                }
            }
            Keysym::space | Keysym::KP_Space if !searching => {
                if let Some(i) = self.drop.as_ref().and_then(|d| d.lit) {
                    self.pick_drop(i, reply);
                }
            }
            Keysym::BackSpace if searching => {
                if let Some(d) = &mut self.drop {
                    if mods.ctrl {
                        d.search.clear();
                    } else {
                        let cut = d.search.grapheme_indices(true).next_back().map_or(0, |(i, _)| i);
                        d.search.truncate(cut);
                    }
                }
                self.narrow_drop();
            }
            _ => {
                let typed = text.filter(|t| !mods.ctrl && !mods.alt && !mods.logo && !t.is_empty() && !t.chars().any(char::is_control));
                if let Some(t) = typed {
                    self.drop_typed(t, now);
                }
            }
        }
        true
    }

    fn drop_release(&mut self, l: &DropLaid, x: f32, y: f32, reply: &mut Reply<M>) {
        let Some(d) = &mut self.drop else { return };
        let (held, down) = (d.thumb.take().is_some(), std::mem::take(&mut d.down));
        if !held && down {
            if let Some(row) = self.drop_row_at(l, x, y) {
                self.pick_drop(row, reply);
            }
        }
    }

    fn drop_touch(&mut self, l: &DropLaid, x: f32, y: f32) {
        if !l.panel.contains(x, y) {
            self.close_drop();
            return;
        }
        self.pointer = Some((x, y));
        let row = self.drop_row_at(l, x, y);
        if let Some(d) = &mut self.drop {
            d.finger = Some((y, d.top, false));
            if row.is_some() {
                d.lit = row;
            }
        }
    }

    fn drop_swipe(&mut self, x: f32, y: f32, k: f32) {
        self.pointer = Some((x, y));
        let Some((y0, top0, moved)) = self.drop.as_ref().and_then(|d| d.finger) else { return };
        if moved || (y - y0).abs() > SWIPE_SLOP * k {
            if let Some(d) = &mut self.drop {
                d.finger = Some((y0, top0, true));
            }
            self.scroll_drop(top0 + y0 - y);
        }
    }

    fn drop_lift(&mut self, l: &DropLaid, reply: &mut Reply<M>) {
        let finger = self.drop.as_mut().and_then(|d| d.finger.take());
        let at = self.pointer.take();
        if let (Some((_, _, false)), Some((x, y))) = (finger, at) {
            if let Some(row) = self.drop_row_at(l, x, y) {
                self.pick_drop(row, reply);
            }
        }
    }

    fn drop_pointer(&mut self, input: &Input, l: &DropLaid, k: f32, reply: &mut Reply<M>) {
        match *input {
            Input::PointerIn { x, y } | Input::PointerMove { x, y } => {
                self.drop_move(l, x * k, y * k);
            }
            Input::PointerOut => {
                self.pointer = None;
            }
            Input::Press { button: Button::Left, x, y } if l.panel.contains(x * k, y * k) => {
                self.drop_press(l, x * k, y * k);
            }
            Input::Release { button: Button::Left, x, y } => {
                self.drop_release(l, x * k, y * k, reply);
            }
            Input::Press { .. } => {
                self.close_drop();
            }
            Input::Release { .. } => {}
            Input::TouchDown { x, y, .. } => {
                self.drop_touch(l, x * k, y * k);
            }
            Input::TouchMove { x, y, .. } => {
                self.drop_swipe(x * k, y * k, k);
            }
            Input::TouchUp { .. } => {
                self.drop_lift(l, reply);
            }
            Input::TouchCancel => {
                if let Some(d) = &mut self.drop {
                    d.finger = None;
                }
            }
            _ => {}
        }
    }

    pub(super) fn drop_input(&mut self, input: &Input, now: Instant) -> Option<Reply<M>> {
        let k = self.scale.max(f32::EPSILON);
        let l = self.drop_laid()?;
        let mut reply = Reply::new();
        reply.redraw = true;
        match *input {
            Input::PointerIn { .. } | Input::PointerMove { .. } | Input::PointerOut | Input::Press { .. } | Input::Release { .. }
            | Input::TouchDown { .. } | Input::TouchMove { .. } | Input::TouchUp { .. } | Input::TouchCancel => {
                self.drop_pointer(input, &l, k, &mut reply);
                Some(reply)
            }
            Input::Scroll { y, steps, .. } => {
                if !self.pointer.is_some_and(|(px, py)| l.panel.contains(px, py)) {
                    self.close_drop();
                    return None;
                }
                self.drop_scroll(&l, y, steps, k);
                Some(reply)
            }
            Input::Key { sym, ref text, mods, .. } => {
                self.keyboard_seen = true;
                if !self.drop_key(&l, sym, text.as_deref(), mods, now, &mut reply) {
                    return None;
                }
                Some(reply)
            }
            Input::Composed { ref commit, .. } => {
                if !commit.is_empty() {
                    self.drop_typed(commit, now);
                }
                Some(reply)
            }
            Input::Focus(false) => {
                self.close_drop();
                None
            }
            _ => None,
        }
    }

    pub(super) fn paint_dropdown(&self, canvas: &mut PixmapMut, spot: &Spot, shown: &str, lit: bool) {
        let t = &self.theme;
        let r = spot.rect;
        let open = self.drop.as_ref().filter(|d| d.id == spot.id);
        let glow = self.glow_at(&spot.id, Instant::now());
        let rest = if lit || open.is_some() { t.accent } else { t.field_edge };
        self.paint_sunk(canvas, r, self.corner(theme::Part::Box), theme::mix(rest, t.accent, glow), glow);
        let pad = self.px(theme::sheet().entry_padding);
        let mark = self.px(w::CHEVRON);
        let style = Words::Plain.style(self.scale);
        let dots = self.texts.get(&self.fonts, "\u{2026}", &style, None);
        let room = (r.w - 2.0 * pad - self.px(w::CHEVRON_GAP) - mark).max(0.0);
        let words = self.texts.get(&self.fonts, shown, &style, None).cut(room, &dots);
        self.note_cut(&spot.id, None, shown, &words);
        self.fonts.draw(canvas, &words, r.x + pad, r.y + ((r.h - words.height) / 2.0).round(), t.fg);
        let icon = if open.is_some_and(|d| d.up) { "pan-up-symbolic" } else { "pan-down-symbolic" };
        self.icons.draw(canvas, icon, (r.x + r.w - pad - mark).round() as i32, (r.y + ((r.h - mark) / 2.0).round()) as i32, mark as u32, 1, t.fg);
    }

    fn drop_search_on(&self, p: &mut Painter, s: Rect, d: &DropOpen, shown: usize, total: usize) {
        let t = &self.theme;
        let sheet = theme::sheet();
        let pad = self.px(sheet.entry_padding);
        let style = Words::Plain.style(self.scale);
        let dots = self.texts.get(&self.fonts, "\u{2026}", &style, None);
        self.sunk_on(p, s, self.corner(theme::Part::Box), t.accent, 0.0);
        let count = self.texts.get(&self.fonts, &crate::t!("kit-search-of", n = shown, total = total), &Words::Note.style(self.scale), None);
        let end = s.x + s.w - pad - count.width;
        self.fonts.paint(p, &count, end, s.y + ((s.h - count.height) / 2.0).round(), t.muted);
        let room = (end - self.px(sheet.gap) - s.x - pad).max(0.0);
        let (text, ink) = if d.search.is_empty() { (search(), t.muted) } else { (d.search.as_str(), t.fg) };
        let words = self.texts.get(&self.fonts, text, &style, None).cut(room, &dots);
        let ty = s.y + ((s.h - words.height) / 2.0).round();
        self.fonts.paint(p, &words, s.x + pad, ty, ink);
        if self.keys_here {
            let cw = self.px(sheet.caret).max(1.0);
            let x = s.x + pad + if d.search.is_empty() { 0.0 } else { words.width.min(room) };
            fill_on(p, Rect { x: x.round(), y: ty, w: cw, h: words.height }, 0.0, t.fg);
        }
    }

    pub(super) fn paint_drop(&self, canvas: &mut PixmapMut) {
        let mut p = Painter::default();
        self.drop_on(&mut p);
        crate::paint::cpu::play(p.ops, canvas, &self.shadows);
    }

    pub(super) fn drop_on(&self, p: &mut Painter) {
        let (Some(d), Some(l)) = (&self.drop, self.drop_laid()) else { return };
        let Some(src) = self.source_of(&d.id, d.part) else { return };
        let (options, chosen, dead) = (src.options, &src.chosen, src.dead);
        let t = &self.theme;
        let sheet = theme::sheet();
        self.flat_on(p, l.panel, self.corner(theme::Part::Menu), &t.float, t.raised, t.card_edge);
        let pad = self.px(sheet.entry_padding);
        let style = Words::Plain.style(self.scale);
        let dots = self.texts.get(&self.fonts, "\u{2026}", &style, None);
        if let Some(s) = l.search {
            self.drop_search_on(p, s, d, l.shown.len(), options.len());
        }
        let r = l.rows;
        if r.w < 1.0 || r.h < 1.0 {
            return;
        }
        if let Some(at) = tiny_skia::IntRect::from_xywh(r.x.round() as i32, r.y.round() as i32, r.w as u32, r.h as u32) {
            p.push_clip(at, 0.0);
            if l.shown.is_empty() {
                let words = self.texts.get(&self.fonts, nothing(), &style, None);
                self.fonts.paint(p, &words, pad, ((l.row - words.height) / 2.0).round(), t.muted);
            }
            let bar_room = if l.full() > r.h + 0.5 { self.drop_bar_room() } else { 0.0 };
            let radius = self.corner(theme::Part::Row);
            let first = (d.top / l.row).floor().max(0.0) as usize;
            let last = (((d.top + r.h) / l.row).ceil() as usize).min(l.shown.len());
            for (at, &i) in l.shown.iter().enumerate().take(last).skip(first) {
                let b = Rect { x: 0.0, y: (at as f32 * l.row - d.top).round(), w: r.w - bar_room, h: l.row };
                let kept = *chosen == Some(i);
                if kept {
                    fill_on(p, b, radius, t.picked);
                } else if d.lit == Some(i) && !dead {
                    fill_on(p, b, radius, t.raised_more);
                }
                if kept && d.lit == Some(i) && self.keyboard_seen {
                    self.hover_edge_on(p, b, radius);
                }
                let words = self.texts.get(&self.fonts, &options[i], &style, None).cut((b.w - 2.0 * pad).max(0.0), &dots);
                let ink = if kept { t.on_picked } else if dead { t.muted } else { t.fg };
                self.fonts.paint(p, &words, b.x + pad, b.y + ((b.h - words.height) / 2.0).round(), ink);
            }
            p.pop();
        }
        let over = self.pointer.is_some_and(|(x, y)| l.panel.contains(x, y)) || d.thumb.is_some();
        let thick = self.px(if over { sheet.scrollbar_hover } else { sheet.scrollbar });
        if let Some((track, thumb)) = self.drop_bar(&l, d.top, thick) {
            if over {
                fill_on(p, track, self.corner(theme::Part::Scrollbar), theme::alpha(t.muted, 0.15));
            }
            fill_on(p, thumb, self.corner(theme::Part::Scrollbar), theme::alpha(t.muted, if over { 0.8 } else { 0.5 }));
        }
    }
}
