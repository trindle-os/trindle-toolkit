//! The editing area: a drawing that lays out only the lines in sight, the view kept as the top line and how far into it.

use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::ops::Range;
use std::rc::Rc;
use std::time::{Duration, Instant};

use tiny_skia::PixmapMut;

use super::doc::{Doc, Kind};
use crate::colour::Colour;
use crate::draw::words_on;
use crate::paint::Painter;
use crate::input::{Input, Keysym, Mods};
use crate::text::{Fonts, Layout, Line, Style};
use crate::theme::{self, alpha, mix};
use crate::widget::{Pen, Pointer};

const PAD: f32 = 10.0;
const TAB_SPACES: f32 = 4.0;
const CLICK_GAP: Duration = Duration::from_millis(450);
const BAR_GRAB: f32 = 14.0;
const WHEEL_LINES: f32 = 3.0;
pub const LINE: f32 = 1.35;
const SIDE_ROOM: f32 = 40.0;

#[derive(Clone, Debug, PartialEq)]
pub struct Look {
    pub font: Option<String>,
    pub size: f32,
    pub wrap: bool,
    pub numbers: bool,
}

pub type Checker = Rc<dyn Fn(&str) -> bool>;

struct Row {
    start: usize,
    end: usize,
    top: f32,
    height: f32,
    stops: Vec<(usize, f32)>,
}

impl Row {
    fn x_of(&self, at: usize) -> f32 {
        match self.stops.binary_search_by_key(&at, |s| s.0) {
            Ok(i) => self.stops[i].1,
            Err(i) => self.stops.get(i.saturating_sub(1)).map(|s| s.1).unwrap_or(0.0),
        }
    }

    fn nearest(&self, x: f32) -> usize {
        let mut best = (f32::MAX, self.start);
        for &(at, sx) in &self.stops {
            let d = (sx - x).abs();
            if d < best.0 {
                best = (d, at);
            }
        }
        best.1
    }
}

struct Para {
    key: u64,
    text: u64,
    height: f32,
    layout: Layout,
    rows: Vec<Row>,
    bad: Vec<Range<usize>>,
}

#[derive(Clone, Copy)]
struct Seen {
    para: usize,
    row: usize,
    y: f32,
}

#[derive(Debug, PartialEq)]
pub enum Did {
    Nothing,
    Moved,
    Edited,
    Copy(String),
    Paste,
}

pub struct Pane {
    pub doc: Doc,
    pub look: Look,
    pub editing: bool,
    pub marks: Vec<Range<usize>>,
    preedit: (String, Option<usize>),
    pub checker: Option<Checker>,
    pub checked: u64,
    pub over: bool,
    pub origin: (f32, f32),
    top: usize,
    off: f32,
    pending: f32,
    follow: bool,
    hscroll: f32,
    goal: Option<f32>,
    shift: bool,
    clicks: (Option<Instant>, f32, f32, u32),
    dragging: Drag,
    cache: HashMap<usize, Para>,
    seen: Vec<Seen>,
    size: (f32, f32),
    scale: f32,
    text_x: f32,
    space: f32,
    pub caret_box: (f32, f32, f32, f32),
    pub families: Vec<String>,
}

#[derive(Clone, Copy, PartialEq)]
enum Drag {
    None,
    Text,
    Bar,
}

fn hash<T: Hash + ?Sized>(t: &T) -> u64 {
    let mut h = DefaultHasher::new();
    t.hash(&mut h);
    h.finish()
}

pub fn rect(canvas: &mut PixmapMut, x: f32, y: f32, w: f32, h: f32, c: Colour) {
    let mut p = Painter::default();
    rect_on(&mut p, x, y, w, h, c);
    p.play(canvas);
}

pub fn rect_on(p: &mut Painter, x: f32, y: f32, w: f32, h: f32, c: Colour) {
    p.fill(crate::widget::Rect { x, y, w: w.max(0.5), h: h.max(0.5) }, c);
}

impl Pane {
    pub fn new(doc: Doc, look: Look) -> Pane {
        Pane {
            doc,
            look,
            editing: false,
            marks: Vec::new(),
            checker: None,
            checked: 0,
            over: false,
            origin: (0.0, 0.0),
            top: 0,
            off: 0.0,
            pending: 0.0,
            follow: true,
            hscroll: 0.0,
            goal: None,
            shift: false,
            clicks: (None, 0.0, 0.0, 0),
            dragging: Drag::None,
            cache: HashMap::new(),
            seen: Vec::new(),
            size: (0.0, 0.0),
            scale: 1.0,
            text_x: 0.0,
            space: 0.0,
            caret_box: (0.0, 0.0, 1.0, 16.0),
            families: Vec::new(),
            preedit: (String::new(), None),
        }
    }

    pub fn hold_shift(&mut self, on: bool) {
        self.shift = on;
    }

    pub fn reset(&mut self) {
        self.marks.clear();
        self.goal = None;
        self.preedit = (String::new(), None);
        self.top = 0;
        self.off = 0.0;
        self.pending = 0.0;
        self.hscroll = 0.0;
        self.cache.clear();
        self.follow = true;
    }

    pub fn takes(input: &Input) -> bool {
        let Input::Key { sym, text, mods, .. } = input else { return matches!(input, Input::Focus(_) | Input::Composed { .. }) };
        let (ctrl, shift) = (mods.ctrl, mods.shift);
        match *sym {
            Keysym::Left | Keysym::Right | Keysym::Up | Keysym::Down | Keysym::Page_Up | Keysym::Page_Down | Keysym::Home | Keysym::KP_Home
            | Keysym::End | Keysym::KP_End | Keysym::BackSpace | Keysym::Delete | Keysym::KP_Delete | Keysym::Return | Keysym::KP_Enter => true,
            Keysym::Tab => !ctrl && !mods.alt,
            Keysym::Insert => shift || ctrl,
            Keysym::a | Keysym::A | Keysym::c | Keysym::C | Keysym::x | Keysym::X | Keysym::v | Keysym::V | Keysym::z | Keysym::Z | Keysym::y | Keysym::Y => ctrl || (!mods.alt && !mods.logo && text.as_deref().is_some_and(typed)),
            Keysym::Escape => false,
            _ => !(ctrl || mods.alt || mods.logo) && text.as_deref().is_some_and(typed),
        }
    }

    pub fn passes(&self, input: &Input) -> bool {
        let Input::Key { sym, mods, .. } = input else { return false };
        if !mods.ctrl || mods.alt || mods.logo {
            return false;
        }
        match *sym {
            Keysym::z | Keysym::Z if !mods.shift => !self.doc.can_undo(),
            Keysym::z | Keysym::Z | Keysym::y | Keysym::Y => !self.doc.can_redo(),
            _ => false,
        }
    }

    pub fn first_line(&self) -> usize {
        self.top
    }

    pub fn show_caret(&mut self) {
        self.follow = true;
    }

    pub fn scroll(&mut self, x: f32, y: f32, steps: bool) {
        let line = self.look.size * LINE;
        let by = |v: f32| if steps { v * WHEEL_LINES * line } else { v };
        self.pending += by(y);
        if !self.look.wrap {
            self.hscroll = (self.hscroll + by(x) * self.scale).max(0.0);
        }
        self.follow = false;
    }

    pub fn text_left(&self) -> f32 {
        self.text_x / self.scale
    }

    pub fn width(&self) -> f32 {
        self.size.0 / self.scale
    }

    fn style(&self, scale: f32) -> Style {
        Style { family: self.look.font.clone().filter(|f| !f.is_empty()), ..Style::new(self.look.size * scale) }
    }

    fn para(&mut self, fonts: &Fonts, p: usize, style: &Style, wrap: Option<f32>) -> &Para {
        let range = self.doc.line_range(p);
        let line = &self.doc.text()[range];
        let text = hash(line);
        let key = hash(&(text, wrap.map(f32::to_bits), style.size.to_bits(), &style.family, self.checked, self.checker.is_some()));
        if self.cache.get(&p).is_none_or(|c| c.key != key) {
            let built = build(fonts, line, style, wrap, self.space, self.checker.as_ref());
            self.cache.insert(p, Para { key, text, ..built });
        }
        &self.cache[&p]
    }

    fn laid(&self, p: usize) -> Option<&Para> {
        let c = self.cache.get(&p)?;
        (p < self.doc.lines() && c.text == hash(self.doc.line(p))).then_some(c)
    }

    fn row_of(&self, at: usize) -> Option<(usize, usize)> {
        let p = self.doc.line_of(at);
        let rel = at - self.doc.line_range(p).start;
        let para = self.laid(p)?;
        let last = para.rows.len().saturating_sub(1);
        let r = para.rows.iter().position(|r| rel >= r.start && rel < r.end).unwrap_or(last);
        Some((p, r))
    }

    pub fn paint(&mut self, canvas: &mut PixmapMut, pen: &Pen) {
        let mut p = Painter::default();
        self.paint_on(&mut p, pen, (canvas.width(), canvas.height()));
        p.play(canvas);
    }

    pub fn paint_on<'a>(&mut self, p: &mut Painter<'a>, pen: &Pen<'a>, size: (u32, u32)) {
        let s = pen.scale;
        let t = pen.theme;
        if self.families.is_empty() {
            self.families = pen.fonts.families();
        }
        if (s - self.scale).abs() > f32::EPSILON {
            self.off *= s / self.scale;
            self.hscroll *= s / self.scale;
        }
        self.scale = s;
        let (w, h) = (size.0 as f32, size.1 as f32);
        self.size = (w, h);
        let style = self.style(s);
        let digits = self.doc.lines().to_string().len().max(3);
        let number_style = Style { size: style.size * 0.85, ..style.clone() };
        let wide = |text: &str| pen.fonts.layout(text, &style, None).width;
        self.space = wide("a a") - wide("aa");
        let gutter = if self.look.numbers { pen.fonts.layout(&"0".repeat(digits), &number_style, None).width + 2.0 * PAD * s } else { 0.0 };
        self.text_x = gutter + PAD * s;
        let bar = theme::sheet().scrollbar_hover * s;
        let wrap = self.look.wrap.then(|| (w - self.text_x - PAD * s - bar).max(self.space * 4.0));
        if self.look.wrap {
            self.hscroll = 0.0;
        }
        self.top = self.top.min(self.doc.lines() - 1);
        self.apply_scroll(pen.fonts, &style, wrap);
        if self.follow {
            self.follow = false;
            self.follow_caret(pen.fonts, &style, wrap, h);
        }
        self.lay_rows_in_sight(pen.fonts, &style, wrap, h);
        if !self.look.wrap {
            self.keep_caret_across(w - self.text_x - PAD * s - bar);
        }

        rect_on(p, 0.0, 0.0, w, h, t.field);
        let field = theme::over(t.field, theme::over(t.panel, t.bg));
        if let Some(at) = self.paint_rows(p, pen, field) {
            self.paint_caret(p, pen, &style, at);
        }
        if self.look.numbers {
            self.paint_numbers(p, pen, &number_style, gutter, field);
        }
        self.paint_bar(p, pen);
    }

    fn apply_scroll(&mut self, fonts: &Fonts, style: &Style, wrap: Option<f32>) {
        let lines = self.doc.lines();
        self.off += self.pending * self.scale;
        self.pending = 0.0;
        while self.off < 0.0 && self.top > 0 {
            self.top -= 1;
            self.off += self.para(fonts, self.top, style, wrap).height;
        }
        loop {
            let height = self.para(fonts, self.top, style, wrap).height;
            if self.off >= height && self.top + 1 < lines {
                self.off -= height;
                self.top += 1;
            } else {
                break;
            }
        }
        self.off = self.off.max(0.0).min(self.para(fonts, self.top, style, wrap).height - 1.0).max(0.0);
    }

    fn lay_rows_in_sight(&mut self, fonts: &Fonts, style: &Style, wrap: Option<f32>, h: f32) {
        let lines = self.doc.lines();
        self.seen.clear();
        let mut y = -self.off;
        let mut p = self.top;
        if p > 0 {
            self.para(fonts, p - 1, style, wrap);
        }
        while p < lines && y < h {
            let para = self.para(fonts, p, style, wrap);
            let (height, rows) = (para.height, para.rows.iter().map(|r| r.top).collect::<Vec<_>>());
            for (i, top) in rows.into_iter().enumerate() {
                self.seen.push(Seen { para: p, row: i, y: y + top });
            }
            y += height;
            p += 1;
        }
        if p < lines {
            self.para(fonts, p, style, wrap);
        }
        let keep = self.top.saturating_sub(1)..p + 1;
        self.cache.retain(|k, _| keep.contains(k));
    }

    fn keep_caret_across(&mut self, room: f32) {
        let Some((cp, cr)) = self.row_of(self.doc.caret) else { return };
        let rel = self.doc.caret - self.doc.line_range(cp).start;
        let x = self.cache[&cp].rows[cr].x_of(rel);
        let margin = (SIDE_ROOM * self.scale).min(room / 3.0);
        if x - self.hscroll > room - margin {
            self.hscroll = x - room + margin;
        } else if x - self.hscroll < margin {
            self.hscroll = (x - margin).max(0.0);
        }
    }

    fn paint_rows<'a>(&self, p: &mut Painter<'a>, pen: &Pen<'a>, field: Colour) -> Option<(f32, f32, f32)> {
        let (s, t) = (pen.scale, pen.theme);
        let w = self.size.0;
        let on_row = t.wash(alpha(t.fg, 0.05), &[field]);
        let found = t.wash(alpha(theme::warn(), 0.32), &[field, theme::over(on_row, field)]);
        let picked = t.wash(alpha(t.accent, theme::sheet().selection), &[field, theme::over(found, field)]);
        let sel = self.doc.selection();
        let caret_row = self.row_of(self.doc.caret);
        let x0 = self.text_x - self.hscroll;
        let mut caret_box = None;
        let mut drawn = usize::MAX;
        for seen in &self.seen {
            let start = self.doc.line_range(seen.para).start;
            let end = self.doc.line_range(seen.para).end;
            let para = &self.cache[&seen.para];
            let row = &para.rows[seen.row];
            let (rs, re) = (start + row.start, start + row.end);
            if sel.is_empty() && caret_row == Some((seen.para, seen.row)) {
                rect_on(p, 0.0, seen.y, w, row.height, on_row);
            }
            for (range, colour) in self.marks.iter().map(|m| (m.clone(), found))
                .chain(std::iter::once((sel.clone(), picked)))
            {
                let (a, b) = (range.start.max(rs), range.end.min(re));
                let past = range.end > re && (re < end || range.end > end);
                if range.is_empty() || a > b || (a == b && !past) {
                    continue;
                }
                let (xa, xb) = (row.x_of(a - start), row.x_of(b - start));
                let (lo, hi) = (xa.min(xb), xa.max(xb) + if past { self.space } else { 0.0 });
                rect_on(p, x0 + lo, seen.y, hi - lo, row.height, colour);
            }
            if drawn != seen.para {
                drawn = seen.para;
                words_on(p, pen.fonts, para.layout.clone(), x0, seen.y - row.top, t.fg);
                for bad in &para.bad {
                    for r in &para.rows {
                        let (a, b) = (bad.start.max(r.start), bad.end.min(r.end));
                        if a < b {
                            let thick = (theme::sheet().caret * s).max(1.0);
                            let top = seen.y - row.top + r.top + r.height - thick;
                            crate::spelling::squiggle_on(p, x0 + r.x_of(a), x0 + r.x_of(b), top, thick, t.spelling);
                        }
                    }
                }
            }
            if caret_row == Some((seen.para, seen.row)) {
                let x = x0 + row.x_of(self.doc.caret - start);
                caret_box = Some((x, seen.y, row.height));
            }
        }
        caret_box
    }

    fn paint_caret<'a>(&mut self, p: &mut Painter<'a>, pen: &Pen<'a>, style: &Style, (x, y, height): (f32, f32, f32)) {
        let (s, t) = (pen.scale, pen.theme);
        let mut x = x;
        if self.editing && !self.preedit.0.is_empty() {
            let layout = pen.fonts.layout(&self.preedit.0, style, None);
            let (width, next) = (layout.width, self.preedit.1.map_or(layout.width, |at| layout.caret(at).0));
            rect_on(p, x, y, width + theme::sheet().caret * s, height, t.field);
            words_on(p, pen.fonts, layout, x, y, t.fg);
            rect_on(p, x, y + height - 2.0 * s, width, s, t.fg);
            x += next;
        }
        if self.editing {
            rect_on(p, x.round(), y, theme::sheet().caret * s, height, t.accent);
        }
        self.caret_box = (x / s, y / s, theme::sheet().caret, height / s);
    }

    fn paint_numbers<'a>(&self, p: &mut Painter<'a>, pen: &Pen<'a>, number_style: &Style, gutter: f32, field: Colour) {
        let (s, t) = (pen.scale, pen.theme);
        let behind = mix(t.field, t.panel, 0.6);
        let under = [theme::over(behind, field)];
        let (here, others) = (t.words_on(&under), t.quiet_on(&under));
        rect_on(p, 0.0, 0.0, gutter, self.size.1, behind);
        let caret_para = self.doc.line_of(self.doc.caret);
        let mut last = usize::MAX;
        for seen in &self.seen {
            if seen.para == last || seen.row != 0 {
                continue;
            }
            last = seen.para;
            let n = pen.fonts.layout(&(seen.para + 1).to_string(), number_style, None);
            let row = &self.cache[&seen.para].rows[0];
            let line = n.lines.first().map(|l| l.baseline).unwrap_or(0.0);
            let base = self.cache[&seen.para].layout.lines.first().map(|l| l.baseline - l.top).unwrap_or(row.height * 0.8);
            let ink = if seen.para == caret_para { here } else { others };
            let x = gutter - PAD * s - n.width;
            words_on(p, pen.fonts, n, x, seen.y + base - line, ink);
        }
    }

    fn paint_bar(&self, p: &mut Painter, pen: &Pen) {
        if !self.has_bar() {
            return;
        }
        let (s, t) = (pen.scale, pen.theme);
        let (w, h) = self.size;
        let shown = self.shown_lines();
        let total = self.doc.lines() as f32;
        let top_height = self.cache.get(&self.top).map(|p| p.height).unwrap_or(1.0).max(1.0);
        let at = (self.top as f32 + self.off / top_height) / total;
        let len = ((shown as f32 / total) * h).max(theme::SCROLLBAR_MIN * s).min(h);
        let y = at * (h - len) / (1.0 - (shown as f32 / total)).max(0.01);
        let bw = if self.dragging == Drag::Bar { theme::sheet().scrollbar_hover } else { theme::sheet().scrollbar } * s;
        rect_on(p, w - bw - theme::sheet().scrollbar_inset * s, y.clamp(0.0, h - len), bw, len, alpha(t.fg, 0.3));
    }

    fn follow_caret(&mut self, fonts: &Fonts, style: &Style, wrap: Option<f32>, h: f32) {
        let caret = self.doc.caret;
        let cp = self.doc.line_of(caret);
        let rel = caret - self.doc.line_range(cp).start;
        let para = self.para(fonts, cp, style, wrap);
        let last = para.rows.len().saturating_sub(1);
        let r = para.rows.iter().position(|r| rel >= r.start && rel < r.end).unwrap_or(last);
        let (row_top, row_bottom) = (para.rows[r].top, para.rows[r].top + para.rows[r].height);
        if cp < self.top || (cp == self.top && row_top < self.off) {
            self.top = cp;
            self.off = row_top;
            return;
        }
        let mut y = -self.off;
        for p in self.top..cp {
            y += self.para(fonts, p, style, wrap).height;
            if y > h {
                break;
            }
        }
        if y + row_bottom <= h {
            return;
        }
        let mut room = h - row_bottom;
        let mut top = cp;
        while room > 0.0 && top > 0 {
            let above = self.para(fonts, top - 1, style, wrap).height;
            if above > room {
                self.top = top - 1;
                self.off = above - room;
                return;
            }
            room -= above;
            top -= 1;
        }
        self.top = top;
        self.off = (-room).max(0.0);
    }

    pub fn hit(&self, x: f32, y: f32) -> usize {
        let (x, y) = (x * self.scale, y * self.scale);
        let Some(first) = self.seen.first() else { return self.doc.caret };
        let seen = if y < first.y {
            *first
        } else {
            *self.seen.iter().rev().find(|s| s.y <= y).unwrap_or(first)
        };
        if y < first.y && self.top > 0 && self.off <= 0.0 {
            return self.doc.line_range((self.top - 1).min(self.doc.lines() - 1)).start;
        }
        let Some(para) = self.laid(seen.para) else { return self.doc.caret };
        let Some(row) = para.rows.get(seen.row) else { return self.doc.caret };
        let last_y = self.seen.last().map(|l| l.y + para.rows.get(l.row).map(|r| r.height).unwrap_or(0.0)).unwrap_or(0.0);
        if y > last_y && seen.para + 1 >= self.doc.lines() {
            return self.doc.len();
        }
        let at = row.nearest(x - self.text_x + self.hscroll);
        let at = if at == row.end && seen.row + 1 < para.rows.len() { self.prev_in(seen.para, at) } else { at };
        self.doc.line_range(seen.para).start + at
    }

    fn prev_in(&self, p: usize, rel: usize) -> usize {
        let start = self.doc.line_range(p).start;
        self.doc.prev_char(start + rel) - start
    }

    pub fn pointer(&mut self, p: Pointer) -> Did {
        match p {
            Pointer::Hover(..) => {
                self.over = true;
                Did::Nothing
            }
            Pointer::Leave => {
                self.over = false;
                Did::Nothing
            }
            Pointer::Down(x, y) => self.press_at(x, y),
            Pointer::Move(x, y) => match self.dragging {
                Drag::Bar => {
                    self.bar_to(y);
                    Did::Moved
                }
                Drag::Text => {
                    let h = self.size.1 / self.scale;
                    if y < 0.0 {
                        self.pending += y;
                    } else if y > h {
                        self.pending += y - h;
                    }
                    let at = self.hit(x, y.clamp(0.0, h));
                    self.doc.go(at, true);
                    Did::Moved
                }
                Drag::None => Did::Nothing,
            },
            Pointer::Up(..) => {
                self.dragging = Drag::None;
                Did::Nothing
            }
            Pointer::Menu(x, y) => {
                self.editing = true;
                let at = self.hit(x, y);
                let sel = self.doc.selection();
                match self.misspelt_at(at) {
                    Some(word) => self.doc.select(word.start, word.end),
                    None if !(sel.start <= at && at <= sel.end && !sel.is_empty()) => self.doc.go(at, false),
                    None => {}
                }
                Did::Moved
            }
            Pointer::Middle(..) | Pointer::RightDown(..) | Pointer::RightMove(..) | Pointer::RightUp(..) => Did::Nothing,
        }
    }

    fn press_at(&mut self, x: f32, y: f32) -> Did {
        self.over = true;
        self.editing = true;
        if self.has_bar() && x * self.scale > self.size.0 - BAR_GRAB * self.scale {
            self.dragging = Drag::Bar;
            self.bar_to(y);
            return Did::Moved;
        }
        let now = Instant::now();
        let near = (x - self.clicks.1).abs() < 4.0 && (y - self.clicks.2).abs() < 4.0;
        let count = match self.clicks.0 {
            Some(t) if near && now - t < CLICK_GAP => self.clicks.3 % 3 + 1,
            _ => 1,
        };
        self.clicks = (Some(now), x, y, count);
        self.dragging = Drag::Text;
        self.goal = None;
        let at = self.hit(x, y);
        match count {
            2 => {
                let w = self.doc.word_at(at);
                self.doc.select(w.start, w.end);
            }
            3 => {
                let line = self.doc.line_range(self.doc.line_of(at));
                let end = if line.end < self.doc.len() { line.end + 1 } else { line.end };
                self.doc.select(line.start, end);
            }
            _ => self.doc.go(at, self.shift),
        }
        Did::Moved
    }

    pub fn misspelt_at(&self, at: usize) -> Option<Range<usize>> {
        let check = self.checker.as_ref()?;
        let line = self.doc.line_range(self.doc.line_of(at));
        let rel = at - line.start;
        let text = &self.doc.text()[line.clone()];
        words(text)
            .find(|w| w.start <= rel && rel <= w.end)
            .filter(|w| !check(&text[w.clone()]))
            .map(|w| line.start + w.start..line.start + w.end)
    }

    pub(super) fn shown_lines(&self) -> usize {
        self.seen.iter().map(|s| s.para).collect::<std::collections::BTreeSet<_>>().len().max(1)
    }

    fn has_bar(&self) -> bool {
        self.doc.lines() > self.shown_lines() || self.top > 0
    }

    fn bar_to(&mut self, y: f32) {
        let h = (self.size.1 / self.scale).max(1.0);
        let lines = self.doc.lines();
        self.top = (((y / h).clamp(0.0, 1.0) * lines as f32) as usize).min(lines - 1);
        self.off = 0.0;
        self.follow = false;
    }

    fn vertical(&self, at: usize, down: bool) -> (usize, Option<f32>) {
        let p = self.doc.line_of(at);
        let start = self.doc.line_range(p).start;
        let rel = at - start;
        let laid = self.row_of(at).and_then(|(_, r)| self.laid(p).map(|para| (para, r)));
        let Some((para, r)) = laid else {
            let col = self.doc.text()[start..at].chars().count();
            let to = if down { p + 1 } else { p.wrapping_sub(1) };
            if to >= self.doc.lines() {
                return (if down { self.doc.len() } else { 0 }, self.goal);
            }
            let line = self.doc.line_range(to);
            let byte = self.doc.text()[line.clone()].char_indices().nth(col).map_or(line.len(), |(i, _)| i);
            return (line.start + byte, self.goal);
        };
        let x = self.goal.unwrap_or_else(|| para.rows[r].x_of(rel));
        let target = if down {
            if r + 1 < para.rows.len() {
                Some((p, r + 1))
            } else if p + 1 < self.doc.lines() {
                Some((p + 1, 0))
            } else {
                None
            }
        } else if r > 0 {
            Some((p, r - 1))
        } else if p > 0 {
            Some((p - 1, usize::MAX))
        } else {
            None
        };
        let Some((tp, tr)) = target else { return (if down { self.doc.len() } else { 0 }, Some(x)) };
        let tstart = self.doc.line_range(tp).start;
        match self.laid(tp) {
            Some(tpara) => {
                let tr = tr.min(tpara.rows.len() - 1);
                let row = &tpara.rows[tr];
                let mut to = row.nearest(x);
                if to == row.end && tr + 1 < tpara.rows.len() {
                    to = self.prev_in(tp, to);
                }
                (tstart + to, Some(x))
            }
            None => {
                let col = self.doc.text()[start..at].chars().count();
                let line = self.doc.line_range(tp);
                let byte = self.doc.text()[line.clone()].char_indices().nth(col).map_or(line.len(), |(i, _)| i);
                (line.start + byte, self.goal.or(Some(x)))
            }
        }
    }

    fn row_ends(&self, at: usize) -> (usize, usize) {
        let p = self.doc.line_of(at);
        let line = self.doc.line_range(p);
        match self.row_of(at).and_then(|(_, r)| self.laid(p).map(|para| (para, r))) {
            Some((para, r)) => {
                let row = &para.rows[r];
                let end = if r + 1 < para.rows.len() { self.prev_in(p, row.end) } else { row.end };
                (line.start + row.start, line.start + end)
            }
            None => (line.start, line.end),
        }
    }

    pub fn key(&mut self, input: &Input) -> Option<Did> {
        match input {
            Input::Focus(here) => {
                self.editing = *here;
                self.preedit = (String::new(), None);
                Some(Did::Moved)
            }
            Input::Composed { delete, commit, preedit, caret } => {
                self.preedit = (preedit.clone(), caret.map(|c| c.1.min(preedit.len())));
                if delete.0 == 0 && delete.1 == 0 && commit.is_empty() {
                    self.follow = true;
                    return Some(Did::Moved);
                }
                let (before, after) = (delete.0 as usize, delete.1 as usize);
                if before > 0 || after > 0 {
                    let c = self.doc.caret;
                    let a = c.saturating_sub(before);
                    let b = (c + after).min(self.doc.len());
                    if self.doc.text().is_char_boundary(a) && self.doc.text().is_char_boundary(b) {
                        self.doc.replace(a..b, "", Kind::Delete);
                    }
                }
                let commit = clean(commit);
                if !commit.is_empty() {
                    self.doc.put(&commit, Kind::Type);
                }
                self.after_edit();
                Some(Did::Edited)
            }
            Input::Key { sym, text, mods, .. } => self.press(*sym, text.as_deref(), *mods),
            _ => None,
        }
    }

    fn edited_if(&mut self, changed: bool) -> Did {
        if !changed {
            return Did::Nothing;
        }
        self.after_edit();
        Did::Edited
    }

    fn after_edit(&mut self) {
        self.goal = None;
        self.follow = true;
    }

    fn press_move(&mut self, sym: Keysym, mods: Mods) -> Option<Did> {
        let (ctrl, shift) = (mods.ctrl, mods.shift);
        let caret = self.doc.caret;
        let sel = self.doc.selection();
        let moved = |pane: &mut Pane, to: usize, keep_goal: bool| {
            if !keep_goal {
                pane.goal = None;
            }
            pane.doc.go(to, shift);
            pane.follow = true;
            Some(Did::Moved)
        };
        let collapse = !shift && !sel.is_empty();
        match crate::edit::facing(sym, self.doc.line(self.doc.line_of(caret))) {
            Keysym::Left if collapse && !ctrl => moved(self, sel.start, false),
            Keysym::Right if collapse && !ctrl => moved(self, sel.end, false),
            Keysym::Left => {
                let to = if ctrl { self.doc.word_left(caret) } else { self.doc.prev_char(caret) };
                moved(self, to, false)
            }
            Keysym::Right => {
                let to = if ctrl { self.doc.word_right(caret) } else { self.doc.next_char(caret) };
                moved(self, to, false)
            }
            Keysym::Up | Keysym::Down => {
                let (to, x) = self.vertical(caret, sym == Keysym::Down);
                self.goal = x;
                moved(self, to, true)
            }
            Keysym::Page_Up | Keysym::Page_Down => {
                let rows = self.seen.len().saturating_sub(2).max(1);
                let mut at = caret;
                for _ in 0..rows {
                    let (to, x) = self.vertical(at, sym == Keysym::Page_Down);
                    self.goal = x;
                    at = to;
                }
                self.pending += if sym == Keysym::Page_Down { 1.0 } else { -1.0 } * (rows as f32) * self.look.size * LINE;
                moved(self, at, true)
            }
            Keysym::Home | Keysym::KP_Home => {
                let to = if ctrl {
                    0
                } else {
                    let (start, _) = self.row_ends(caret);
                    let line = self.doc.line_range(self.doc.line_of(caret));
                    let indent = line.start + self.doc.text()[line.clone()].len() - self.doc.text()[line.clone()].trim_start().len();
                    if start == line.start && caret != indent { indent } else { start }
                };
                moved(self, to, false)
            }
            Keysym::End | Keysym::KP_End => {
                let to = if ctrl { self.doc.len() } else { self.row_ends(caret).1 };
                moved(self, to, false)
            }
            _ => None,
        }
    }

    fn press_delete(&mut self, sym: Keysym, mods: Mods) -> Option<Did> {
        let (ctrl, shift) = (mods.ctrl, mods.shift);
        let caret = self.doc.caret;
        let sel = self.doc.selection();
        match sym {
            Keysym::BackSpace => {
                if sel.is_empty() {
                    let from = if ctrl { self.doc.word_left(caret) } else { self.doc.prev_char(caret) };
                    self.doc.replace(from..caret, "", Kind::Delete);
                } else {
                    self.doc.put("", Kind::Other);
                }
                self.after_edit();
                Some(Did::Edited)
            }
            Keysym::Delete | Keysym::KP_Delete if shift && !sel.is_empty() => {
                let cut = self.doc.selected().to_string();
                self.doc.put("", Kind::Other);
                self.after_edit();
                Some(Did::Copy(cut))
            }
            Keysym::Delete | Keysym::KP_Delete => {
                if sel.is_empty() {
                    let to = if ctrl { self.doc.word_right(caret) } else { self.doc.next_char(caret) };
                    self.doc.replace(caret..to, "", Kind::Delete);
                } else {
                    self.doc.put("", Kind::Other);
                }
                self.after_edit();
                Some(Did::Edited)
            }
            Keysym::x | Keysym::X if ctrl => {
                if sel.is_empty() {
                    return Some(Did::Nothing);
                }
                let cut = self.doc.selected().to_string();
                self.doc.put("", Kind::Other);
                self.after_edit();
                Some(Did::Copy(cut))
            }
            _ => None,
        }
    }

    fn press(&mut self, sym: Keysym, text: Option<&str>, mods: Mods) -> Option<Did> {
        if let Some(did) = self.press_move(sym, mods).or_else(|| self.press_delete(sym, mods)) {
            return Some(did);
        }
        let (ctrl, shift) = (mods.ctrl, mods.shift);
        let sel = self.doc.selection();
        match sym {
            Keysym::Return | Keysym::KP_Enter => {
                let line = self.doc.line_range(self.doc.line_of(sel.start));
                let upto = &self.doc.text()[line.start..sel.start];
                let indent: String = upto.chars().take_while(|c| *c == ' ' || *c == '\t').collect();
                self.doc.put(&format!("\n{indent}"), Kind::Type);
                self.after_edit();
                Some(Did::Edited)
            }
            Keysym::Tab if !ctrl && !mods.alt => {
                self.doc.put("\t", Kind::Type);
                self.after_edit();
                Some(Did::Edited)
            }
            Keysym::Insert if shift => Some(Did::Paste),
            Keysym::a | Keysym::A if ctrl => {
                self.doc.select(0, self.doc.len());
                Some(Did::Moved)
            }
            Keysym::c | Keysym::C | Keysym::Insert if ctrl => {
                Some(if sel.is_empty() { Did::Nothing } else { Did::Copy(self.doc.selected().to_string()) })
            }
            Keysym::v | Keysym::V if ctrl => Some(Did::Paste),
            Keysym::z | Keysym::Z if ctrl => {
                let done = if shift { self.doc.redo() } else { self.doc.undo() };
                Some(self.edited_if(done))
            }
            Keysym::y | Keysym::Y if ctrl => {
                let done = self.doc.redo();
                Some(self.edited_if(done))
            }
            Keysym::Escape => None,
            _ if ctrl || mods.alt || mods.logo => None,
            _ => {
                let text = text.filter(|t| typed(t))?;
                self.doc.put(text, Kind::Type);
                self.after_edit();
                Some(Did::Edited)
            }
        }
    }

    pub fn paste(&mut self, text: &str) {
        self.doc.put(&clean(text), Kind::Other);
        self.after_edit();
    }

    pub fn reveal(&mut self, range: Range<usize>) {
        self.doc.select(range.start, range.end);
        self.goal = None;
        self.follow = true;
    }
}

fn clean(text: &str) -> String {
    text.replace("\r\n", "\n").replace('\r', "\n").chars()
        .filter(|c| matches!(c, '\n' | '\t') || !(c.is_control() || matches!(c, '\u{202a}'..='\u{202e}' | '\u{2066}'..='\u{2069}')))
        .collect()
}

fn typed(text: &str) -> bool {
    !text.is_empty() && !text.chars().any(char::is_control)
}

fn build(fonts: &Fonts, line: &str, style: &Style, wrap: Option<f32>, space: f32, checker: Option<&Checker>) -> Para {
    let shown = line.replace('\t', " ");
    let mut layout = fonts.layout(&shown, style, wrap);
    if layout.lines.is_empty() {
        let blank = fonts.layout(" ", style, None);
        layout.height = blank.height;
        layout.lines = blank.lines;
        if let Some(l) = layout.lines.first_mut() {
            l.range = 0..0;
            l.shown_end = 0;
            l.glyphs.clear();
            l.width = 0.0;
        }
    }
    let tab = (space * TAB_SPACES).max(1.0);
    let rows: Vec<Row> = layout.lines.iter_mut().map(|l| row_of(line, l, space, tab)).collect();
    let height = layout.height.max(rows.last().map(|r| r.top + r.height).unwrap_or(0.0));
    let bad = match checker {
        Some(check) => words(line).filter(|r| !check(&line[r.clone()])).collect(),
        None => Vec::new(),
    };
    Para { key: 0, text: 0, height, layout, rows, bad }
}

fn row_of(line: &str, l: &mut Line, space: f32, tab: f32) -> Row {
    let mut at_x: HashMap<usize, f32> = HashMap::new();
    for (i, g) in l.glyphs.iter().enumerate() {
        if g.rtl {
            let right = l.glyphs.get(i + 1).map_or(l.x + l.width, |n| n.x);
            let e = at_x.entry(g.at).or_insert(right);
            *e = e.max(right);
        } else {
            let e = at_x.entry(g.at).or_insert(g.x);
            *e = e.min(g.x);
        }
    }
    let mut stops = Vec::new();
    let mut last = l.x;
    let mut trailing = l.x + l.width;
    for (i, _) in line[l.range.clone()].char_indices() {
        let b = l.range.start + i;
        let x = if b < l.shown_end {
            at_x.get(&b).copied().unwrap_or(last)
        } else {
            let x = trailing;
            trailing += space;
            x
        };
        last = x;
        stops.push((b, x));
    }
    let end_x = if l.range.end > l.shown_end || l.range.is_empty() { trailing } else { l.x + l.width };
    stops.push((l.range.end, end_x.max(last)));
    let mut shifts: Vec<(usize, f32)> = Vec::new();
    let mut shift = 0.0;
    for k in 0..stops.len() {
        stops[k].1 += shift;
        let b = stops[k].0;
        if b < l.range.end && line.as_bytes()[b] == b'\t' {
            let start = stops[k].1;
            let natural = stops.get(k + 1).map(|s| s.1 + shift).unwrap_or(start + space);
            let wanted = ((start + 0.5) / tab).floor() * tab + tab;
            shift += wanted - natural;
            shifts.push((b, shift));
        }
    }
    if !shifts.is_empty() {
        for g in &mut l.glyphs {
            if let Some(&(_, by)) = shifts.iter().rev().find(|(b, _)| *b < g.at) {
                g.x += by;
            }
        }
    }
    Row { start: l.range.start, end: l.range.end, top: l.top, height: l.height, stops }
}

pub fn words(line: &str) -> impl Iterator<Item = Range<usize>> + '_ {
    let mut at = 0;
    std::iter::from_fn(move || {
        let rest = &line[at..];
        let start = at + rest.find(|c: char| c.is_alphabetic())?;
        let len = line[start..].find(|c: char| !(c.is_alphabetic() || c == '\'' || c == '\u{2019}')).unwrap_or(line.len() - start);
        let mut end = start + len;
        while line[start..end].ends_with(['\'', '\u{2019}']) {
            end = line[start..end].char_indices().next_back().map(|(i, _)| start + i).unwrap_or(end);
        }
        at = start + len.max(1);
        let touches = line[..start].chars().next_back().is_some_and(|c| c.is_ascii_digit() || c == '_')
            || line[start + len..].chars().next().is_some_and(|c| c.is_ascii_digit() || c == '_');
        Some((!touches && end > start).then_some(start..end))
    })
    .flatten()
}

#[cfg(test)]
mod tests;
