//! A small menu that opens over a window, at a point or beside a rectangle.

use std::cell::Cell;
use std::rc::Rc;
use std::time::{Duration, Instant};

use crate::paint::Painter;
use tiny_skia::{FillRule, PixmapMut};

use super::input::Keysym;
use super::text::Style;
use super::theme;
use super::ui::rounded;
use super::widget::{drawing, layers, placed, Length, Node, Pen, SaidRow, Pointer, Rect};

pub fn sub_pause() -> Duration {
    theme::sheet().menu_sub_pause
}
const ARROW: &str = "go-next-symbolic";
fn rule() -> f32 {
    theme::sheet().menu_group_gap
}
fn edge() -> f32 {
    theme::sheet().menu_edge
}
fn least_row() -> f32 {
    theme::sheet().menu_row_least
}

#[derive(Clone, Debug, PartialEq)]
pub struct Row {
    pub icon: String,
    pub words: String,
    pub keys: String,
    pub rule: bool,
    pub sub: Vec<Row>,
    pub sub_width: f32,
    pub check: Option<bool>,
    pub radio: bool,
}

pub fn row(icon: impl Into<String>, words: impl Into<String>) -> Row {
    Row { icon: icon.into(), words: words.into(), keys: String::new(), rule: false, sub: Vec::new(), sub_width: 0.0, check: None, radio: false }
}

impl Row {
    pub fn keys(mut self, keys: impl Into<String>) -> Self {
        self.keys = keys.into();
        self
    }

    pub fn rule(mut self) -> Self {
        self.rule = true;
        self
    }

    pub fn check(mut self, on: bool) -> Self {
        self.check = Some(on);
        self
    }

    pub fn radio(mut self, on: bool) -> Self {
        self.check = Some(on);
        self.radio = true;
        self
    }

    pub fn sub(mut self, rows: Vec<Row>, width: f32) -> Self {
        self.sub = rows;
        self.sub_width = width;
        self
    }

    pub fn said(&self) -> String {
        match (self.sub.len(), self.keys.is_empty()) {
            (0, true) => self.words.clone(),
            (0, false) => crate::t!("kit-row-keys", words = self.words.as_str(), keys = self.keys.as_str()),
            (n, _) => crate::t!("kit-row-sub", words = self.words.as_str(), n = n),
        }
    }
}

static PAUSES: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Beside {
    pub open: Option<usize>,
    pub hover: Option<usize>,
    pub due: Option<(u32, usize)>,
    from: Option<(i32, i32)>,
    aimed: Option<Instant>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Place {
    At(f32, f32),
}

#[derive(Clone, Debug, PartialEq)]
pub struct Popup {
    pub rows: Vec<Row>,
    pub place: Place,
    pub hover: Option<usize>,
    pub width: f32,
    pub most: Option<f32>,
    pub beside: Beside,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PopupMsg {
    Pick(usize),
    Hover(Option<usize>),
    Point { row: Option<usize>, x: i32, y: i32, toward: bool },
    SubHover(Option<usize>),
    SubPick(usize),
    Into,
    Out,
    Back,
    Due(u32),
    Move(i32),
    Letter(char),
    PickHovered,
    Close,
    Stay,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Done {
    Open,
    Picked(usize),
    PickedIn(usize, usize),
    Closed,
}

impl Popup {
    pub fn new(rows: Vec<Row>, place: Place) -> Self {
        Popup { rows, place, hover: None, width: theme::sheet().menu_width, most: None, beside: Beside::default() }
    }

    pub fn lit(mut self) -> Self {
        self.hover = (!self.rows.is_empty()).then_some(0);
        self
    }

    pub fn width(mut self, w: f32) -> Self {
        self.width = w;
        self
    }

    pub fn fits(mut self, room: f32) -> Self {
        self.most = Some((room - 2.0 * edge()).max(0.0));
        self
    }

    pub fn row_height(&self) -> f32 {
        tall_of(&self.rows, self.most)
    }

    fn has_sub(&self, i: usize) -> bool {
        self.rows.get(i).is_some_and(|r| !r.sub.is_empty())
    }

    pub fn sub_rows(&self) -> &[Row] {
        self.beside.open.and_then(|o| self.rows.get(o)).map_or(&[], |r| r.sub.as_slice())
    }

    fn sub_width(&self) -> f32 {
        let own = self.beside.open.and_then(|o| self.rows.get(o)).map_or(0.0, |r| r.sub_width);
        if own > 0.0 { own } else { self.width }
    }

    fn open_sub(&mut self, i: usize, lit: bool) {
        self.hover = Some(i);
        self.beside = Beside { open: Some(i), hover: lit.then_some(0), due: None, aimed: None, ..self.beside };
    }

    fn shut_sub(&mut self) {
        if let Some(o) = self.beside.open {
            self.hover = Some(o);
        }
        self.beside = Beside { open: None, hover: None, due: None, aimed: None, ..self.beside };
    }

    fn wait_for(&mut self, row: usize) {
        if self.beside.due.is_some_and(|(_, r)| r == row) {
            return;
        }
        self.beside.due = Some((PAUSES.fetch_add(1, std::sync::atomic::Ordering::Relaxed).wrapping_add(1), row));
    }

    pub fn said(&self) -> String {
        let inner = self.beside.hover.and_then(|j| self.sub_rows().get(j));
        inner.or_else(|| self.hover.and_then(|i| self.rows.get(i))).map(Row::said).unwrap_or_default()
    }

    pub fn moved(&self, by: i32) -> Option<usize> {
        let n = self.rows.len() as i32;
        if n == 0 {
            return None;
        }
        let from = self.hover.map_or(if by > 0 { -1 } else { n }, |h| h as i32);
        Some((from + by).rem_euclid(n) as usize)
    }

    pub fn update(&mut self, m: PopupMsg) -> Done {
        self.update_at(m, Instant::now())
    }

    pub fn update_at(&mut self, m: PopupMsg, now: Instant) -> Done {
        let n = self.rows.len();
        match m {
            PopupMsg::Pick(i) if self.has_sub(i) => {
                self.open_sub(i, false);
                Done::Open
            }
            PopupMsg::Pick(i) if i < n => Done::Picked(i),
            PopupMsg::Pick(_) | PopupMsg::Stay => Done::Open,
            PopupMsg::Hover(h) => {
                self.hover = h.filter(|&i| i < n).or(self.beside.open);
                self.beside.due = None;
                Done::Open
            }
            PopupMsg::Point { row, x, y, toward } => self.point(row, (x, y), toward, now),
            PopupMsg::Due(k) => self.due(k, now),
            PopupMsg::SubHover(h) => {
                self.beside.hover = h.filter(|&j| j < self.sub_rows().len());
                if self.beside.hover.is_some() {
                    self.hover = self.beside.open;
                    self.beside.due = None;
                    self.beside.aimed = None;
                }
                Done::Open
            }
            PopupMsg::SubPick(j) => match self.beside.open {
                Some(o) if j < self.sub_rows().len() => Done::PickedIn(o, j),
                _ => Done::Open,
            },
            PopupMsg::Move(by) => self.step(by),
            PopupMsg::Letter(c) => self.type_ahead(c),
            PopupMsg::Into => {
                match (self.beside.open, self.hover) {
                    (Some(o), _) if self.beside.hover.is_none() => self.open_sub(o, true),
                    (None, Some(h)) if self.has_sub(h) => self.open_sub(h, true),
                    _ => {}
                }
                Done::Open
            }
            PopupMsg::Out => {
                self.shut_sub();
                Done::Open
            }
            PopupMsg::Back if self.beside.open.is_some() => {
                self.shut_sub();
                Done::Open
            }
            PopupMsg::PickHovered => match (self.beside.open, self.beside.hover, self.hover) {
                (Some(o), Some(j), _) => Done::PickedIn(o, j),
                (_, _, Some(h)) if self.has_sub(h) => {
                    self.open_sub(h, true);
                    Done::Open
                }
                (_, _, h) => h.map_or(Done::Open, Done::Picked),
            },
            PopupMsg::Back | PopupMsg::Close => Done::Closed,
        }
    }

    fn point(&mut self, row: Option<usize>, at: (i32, i32), toward: bool, now: Instant) -> Done {
        let row = row.filter(|&i| i < self.rows.len());
        self.beside.from = Some(at);
        self.beside.hover = None;
        match (row, self.beside.open) {
            (Some(r), Some(o)) if toward && r != o => {
                self.beside.aimed = Some(now);
                self.hover = Some(o);
                self.wait_for(r);
            }
            (Some(r), open) => {
                self.beside.aimed = None;
                self.hover = Some(r);
                if open == Some(r) {
                    self.beside.due = None;
                } else if open.is_some() || self.has_sub(r) {
                    self.wait_for(r);
                } else {
                    self.beside.due = None;
                }
            }
            (None, open) => {
                self.hover = open;
                self.beside.due = None;
            }
        }
        Done::Open
    }

    fn due(&mut self, k: u32, now: Instant) -> Done {
        let Some((_, row)) = self.beside.due.filter(|&(number, _)| number == k) else { return Done::Open };
        self.beside.due = None;
        if self.beside.aimed.is_some_and(|t| now.saturating_duration_since(t) < sub_pause()) {
            self.wait_for(row);
        } else if self.has_sub(row) {
            self.open_sub(row, false);
        } else {
            self.shut_sub();
            self.hover = Some(row).filter(|&i| i < self.rows.len());
        }
        Done::Open
    }

    fn step(&mut self, by: i32) -> Done {
        self.beside.due = None;
        match (self.beside.open, self.beside.hover) {
            (Some(_), Some(j)) => {
                let rows = self.sub_rows().len() as i32;
                self.beside.hover = (rows > 0).then(|| (j as i32 + by).rem_euclid(rows) as usize);
            }
            _ => {
                let to = self.moved(by);
                self.shut_sub();
                self.hover = to;
            }
        }
        Done::Open
    }

    fn type_ahead(&mut self, c: char) -> Done {
        let starts = |r: &Row| r.words.chars().find(|ch| ch.is_alphanumeric()).is_some_and(|f| f.to_lowercase().eq(c.to_lowercase()));
        let in_sub = self.beside.open.is_some() && self.beside.hover.is_some();
        let (rows, from) = if in_sub { (self.sub_rows(), self.beside.hover) } else { (self.rows.as_slice(), self.hover) };
        let n = rows.len();
        let at = (1..=n).map(|k| (from.unwrap_or(n - 1) + k) % n.max(1)).find(|&i| starts(&rows[i]));
        if let Some(i) = at {
            if in_sub {
                self.beside.hover = Some(i);
            } else {
                self.shut_sub();
                self.hover = Some(i);
            }
            self.beside.due = None;
        }
        Done::Open
    }

    pub fn height(&self) -> f32 {
        height_of(&self.rows, self.row_height())
    }

    fn sub_size(&self) -> ((f32, f32), f32) {
        let tall = tall_of(self.sub_rows(), self.most);
        ((self.sub_width(), height_of(self.sub_rows(), tall)), tall)
    }

    pub fn spots(&self, room: (f32, f32)) -> (Rect, Option<Rect>) {
        let (w, h) = (self.width, self.height());
        let (x, y) = spot(self.place, (w, h), room);
        let menu = Rect { x, y, w, h };
        let sub = self.beside.open.filter(|&o| self.has_sub(o)).map(|o| {
            let (size, _) = self.sub_size();
            let rules: Vec<bool> = self.rows.iter().map(|r| r.rule).collect();
            let (x, y) = sub_spot(menu, top_of(&rules, o, self.row_height()), size, room);
            Rect { x, y, w: size.0, h: size.1 }
        });
        (menu, sub)
    }

    pub fn over<M: 'static>(&self, all: Node<M>, wrap: impl Fn(PopupMsg) -> M + Clone + 'static) -> Node<M> {
        let w1 = wrap.clone();
        let catcher = catcher(false, move || w1(PopupMsg::Close));
        let (h, tall) = (self.height(), self.row_height());
        let (rows, hover, open) = (self.rows.clone(), self.hover, self.beside.open.filter(|&o| self.has_sub(o)));
        let rules: Vec<bool> = self.rows.iter().map(|r| r.rule).collect();
        let landed: Rc<Cell<Option<Rect>>> = Rc::new(Cell::new(None));
        let (aim, from, w2) = (landed.clone(), self.beside.from, wrap.clone());
        let lit_here = if self.beside.hover.is_some() { None } else { self.hover };
        let told = menu_told(&self.rows, lit_here, tall);
        let drawn = drawing((self.width, h), "", move |canvas, pen| paint_rows(canvas, pen, &rows, hover, open, tall))
            .menu_rows(told)
            .called(crate::t!("kit-menu"))
            .named(true)
            .width(Length::Fixed(self.width))
            .height(Length::Fixed(h))
            .on_pointer(move |p| wrap(match p {
                Pointer::Down(x, y) | Pointer::Move(x, y) | Pointer::Hover(x, y) => PopupMsg::Point {
                    row: row_of(&rules, y, tall), x: x.round() as i32, y: y.round() as i32,
                    toward: open.is_some() && toward(from, (x, y), aim.get(), tall),
                },
                Pointer::Up(_, y) => row_of(&rules, y, tall).map_or(PopupMsg::Stay, PopupMsg::Pick),
                Pointer::Leave => PopupMsg::Hover(None),
                Pointer::Menu(..) | Pointer::Middle(..) | Pointer::RightDown(..) | Pointer::RightMove(..) | Pointer::RightUp(..) => PopupMsg::Stay,
            }))
            .on_hover()
            .on_keys(|_| None)
            .floats();
        let place = self.place;
        let mut all = vec![all, catcher, placed(drawn, move |size, room| spot(place, size, room))];
        if open.is_some() {
            let (size, tall) = self.sub_size();
            let (rows, lit, me) = (self.sub_rows().to_vec(), self.beside.hover, self.clone());
            let rules: Vec<bool> = rows.iter().map(|r| r.rule).collect();
            let told = menu_told(&rows, lit, tall);
            let sub = drawing(size, "", move |canvas, pen| paint_rows(canvas, pen, &rows, lit, None, tall))
                .menu_rows(told)
                .called(crate::t!("kit-menu"))
                .named(true)
                .width(Length::Fixed(size.0))
                .height(Length::Fixed(size.1))
                .on_pointer(move |p| w2(match p {
                    Pointer::Down(_, y) | Pointer::Move(_, y) | Pointer::Hover(_, y) => PopupMsg::SubHover(row_of(&rules, y, tall)),
                    Pointer::Up(_, y) => row_of(&rules, y, tall).map_or(PopupMsg::Stay, PopupMsg::SubPick),
                    Pointer::Leave => PopupMsg::SubHover(None),
                    Pointer::Menu(..) | Pointer::Middle(..) | Pointer::RightDown(..) | Pointer::RightMove(..) | Pointer::RightUp(..) => PopupMsg::Stay,
                }))
                .on_hover()
                .on_keys(|_| None)
                .floats();
            all.push(placed(sub, move |_, room| {
                let (menu, sub) = me.spots(room);
                let at = sub.unwrap_or(menu);
                landed.set(Some(Rect { x: at.x - menu.x, y: at.y - menu.y, ..at }));
                (at.x, at.y)
            }));
        }
        layers(all)
    }
}

fn menu_told(rows: &[Row], lit: Option<usize>, tall: f32) -> Vec<SaidRow> {
    let rules: Vec<bool> = rows.iter().map(|r| r.rule).collect();
    rows.iter().enumerate().map(|(i, r)| SaidRow { words: r.said(), top: top_of(&rules, i, tall), height: tall, lit: lit == Some(i), check: r.check, radio: r.radio }).collect()
}

fn tall_of(rows: &[Row], most: Option<f32>) -> f32 {
    let usual = theme::sheet().menu_row;
    let Some(most) = most else { return usual };
    let rules = rows.iter().filter(|r| r.rule).count() as f32 * rule();
    let each = ((most - 2.0 * theme::sheet().menu_padding - rules) / rows.len().max(1) as f32).floor();
    each.clamp(least_row().min(usual), usual)
}

fn height_of(rows: &[Row], tall: f32) -> f32 {
    2.0 * theme::sheet().menu_padding + rows.iter().map(|r| tall + if r.rule { rule() } else { 0.0 }).sum::<f32>()
}

fn top_of(rules: &[bool], i: usize, tall: f32) -> f32 {
    theme::sheet().menu_padding + rules.iter().take(i + 1).filter(|r| **r).count() as f32 * rule() + i as f32 * tall
}

pub fn sub_spot(menu: Rect, top: f32, (w, h): (f32, f32), (aw, ah): (f32, f32)) -> (f32, f32) {
    let lap = theme::sheet().menu_sub_overlap;
    let (right, left) = (menu.x + menu.w - lap, menu.x + lap - w);
    let x = if right + w + edge() <= aw {
        right
    } else if left >= edge() {
        left
    } else {
        (aw - w - edge()).max(edge())
    };
    (x, (menu.y + top - theme::sheet().menu_padding).min(ah - h - edge()).max(edge()))
}

pub fn toward(from: Option<(i32, i32)>, to: (f32, f32), sub: Option<Rect>, slack: f32) -> bool {
    let (Some((fx, fy)), Some(sub)) = (from, sub) else { return false };
    let (fx, fy) = (fx as f32, fy as f32);
    let right = sub.x + sub.w / 2.0 > fx;
    let edge = if right { sub.x } else { sub.x + sub.w };
    let (run, left_to_go) = if right { (to.0 - fx, edge - fx) } else { (fx - to.0, fx - edge) };
    if run < 1.0 || left_to_go <= 0.0 {
        return false;
    }
    let meets = fy + (to.1 - fy) * left_to_go / run;
    meets >= sub.y - slack && meets <= sub.y + sub.h + slack
}

pub fn key(sym: Keysym) -> PopupMsg {
    match sym {
        Keysym::Up | Keysym::KP_Up => PopupMsg::Move(-1),
        Keysym::Down | Keysym::KP_Down => PopupMsg::Move(1),
        Keysym::Return | Keysym::KP_Enter | Keysym::space => PopupMsg::PickHovered,
        Keysym::Right | Keysym::KP_Right => PopupMsg::Into,
        Keysym::Left | Keysym::KP_Left => PopupMsg::Out,
        Keysym::Escape => PopupMsg::Back,
        Keysym::Menu => PopupMsg::Close,
        _ => PopupMsg::Stay,
    }
}

pub fn typed(text: Option<&str>, mods: crate::input::Mods) -> PopupMsg {
    let mut chars = text.unwrap_or_default().chars();
    match (chars.next(), chars.next()) {
        (Some(c), None) if c.is_alphanumeric() && !mods.ctrl && !mods.alt => PopupMsg::Letter(c),
        _ => PopupMsg::Stay,
    }
}

pub fn spot(place: Place, (w, h): (f32, f32), (aw, ah): (f32, f32)) -> (f32, f32) {
    let Place::At(x, y) = place;
    let y = if y + h + edge() > ah { (y - h).max(edge()) } else { y };
    (x.min(aw - w - edge()).max(edge()), y)
}

pub(crate) fn row_at(rules: &[bool], y: f32) -> Option<usize> {
    row_of(rules, y, theme::sheet().menu_row)
}

#[cfg(test)]
pub(crate) fn row_top(rules: &[bool], i: usize) -> f32 {
    let tall = theme::sheet().menu_row;
    let before = rules.iter().take(i + 1).filter(|&&r| r).count();
    theme::sheet().menu_padding + i as f32 * tall + before as f32 * rule()
}

fn row_of(rules: &[bool], y: f32, tall: f32) -> Option<usize> {
    let mut top = theme::sheet().menu_padding;
    for (i, &r) in rules.iter().enumerate() {
        if r {
            top += rule();
        }
        if y >= top && y < top + tall {
            return Some(i);
        }
        top += tall;
    }
    None
}

#[cfg(test)]
pub(crate) fn paint_menu(canvas: &mut PixmapMut, pen: &Pen, rows: &[Row], hover: Option<usize>) {
    paint_rows(canvas, pen, rows, hover, None, theme::sheet().menu_row);
}

pub(crate) fn paint_menu_on(p: &mut Painter, pen: &Pen, size: (f32, f32), rows: &[Row], hover: Option<usize>) {
    paint_rows_on(p, pen, size, rows, hover, None, theme::sheet().menu_row);
}

fn paint_rows(canvas: &mut PixmapMut, pen: &Pen, rows: &[Row], hover: Option<usize>, open: Option<usize>, tall: f32) {
    let mut p = Painter::default();
    paint_rows_on(&mut p, pen, (canvas.width() as f32, canvas.height() as f32), rows, hover, open, tall);
    p.play(canvas);
}

fn paint_rows_on(p: &mut Painter, pen: &Pen, (w, h): (f32, f32), rows: &[Row], hover: Option<usize>, open: Option<usize>, tall: f32) {
    let (t, s) = (pen.theme, pen.scale);
    let whole = Rect { x: 0.0, y: 0.0, w, h };
    if let Some(path) = rounded(whole, (theme::sheet().corner_of(theme::Part::Menu) * s).round()) {
        p.fill_path(path, FillRule::Winding, t.raised);
    }
    let plain = Style::new((theme::sheet().text.0 * s).round());
    let small = Style::new((theme::sheet().note.0 * s).round());
    let px = (theme::sheet().menu_icon * s).round();
    let mut top = theme::sheet().menu_padding * s;
    for (i, l) in rows.iter().enumerate() {
        if l.rule {
            top += rule() * s;
        }
        let line = Rect { x: theme::sheet().menu_padding * s, y: top, w: w - 2.0 * theme::sheet().menu_padding * s, h: tall * s };
        let back = if hover == Some(i) || open == Some(i) { Some(t.raised_more) } else if i % 2 == 1 { Some(t.band) } else { None };
        if let Some(back) = back {
            if let Some(path) = rounded(line, theme::sheet().corner_of(theme::Part::Row) * s) {
                p.fill_path(path, FillRule::Winding, back);
            }
        }
        let ink = t.fg;
        let (ix, iy) = (line.x + theme::sheet().menu_padding * s, line.y + (line.h - px) / 2.0);
        if let Some(on) = l.check {
            if on && l.radio {
                dot(p, Rect { x: ix, y: iy, w: px, h: px }, ink);
            } else if on {
                tick(p, Rect { x: ix, y: iy, w: px, h: px }, ink);
            }
        } else if !l.icon.is_empty() {
            let (x, y) = (ix.round() as i32, iy.round() as i32);
            if pen.icons.draw_on(p, &l.icon, x, y, px as u32, 1, ink).is_none() {
                pen.icons.draw_on(p, "text-x-generic-symbolic", x, y, px as u32, 1, ink);
            }
        }
        let pad = theme::sheet().menu_padding * s;
        let arrow = (theme::sheet().menu_sub_arrow * s).round();
        let keys = (l.sub.is_empty() && !l.keys.is_empty()).then(|| pen.fonts.layout(&l.keys, &small, None));
        let at_right = if !l.sub.is_empty() { arrow } else { keys.as_ref().map_or(0.0, |k| k.width) };
        let words_x = ix + px + 2.0 * pad;
        let room = (line.x + line.w - pad - at_right - if at_right > 0.0 { pad } else { 0.0 } - words_x).max(0.0);
        let mut words = pen.fonts.layout(&l.words, &plain, None);
        if words.width > room {
            words = words.cut(room, &pen.fonts.layout("\u{2026}", &plain, None));
        }
        pen.fonts.paint(p, &words, words_x, (line.y + (line.h - words.height) / 2.0).round(), ink);
        if !l.sub.is_empty() {
            let (x, y) = (line.x + line.w - pad - arrow, line.y + (line.h - arrow) / 2.0);
            pen.icons.draw_on(p, ARROW, x.round() as i32, y.round() as i32, arrow as u32, 1, t.muted);
        } else if let Some(keys) = keys {
            let x = line.x + line.w - pad - keys.width;
            pen.fonts.paint(p, &keys, x, (line.y + (line.h - keys.height) / 2.0).round(), t.muted);
        }
        top += tall * s;
    }
}

fn dot(p: &mut Painter, b: Rect, ink: crate::colour::Colour) {
    if let Some(path) = tiny_skia::PathBuilder::from_circle(b.x + b.w / 2.0, b.y + b.h / 2.0, b.w / 5.0) {
        p.fill_path(path, FillRule::Winding, ink);
    }
}

fn tick(p: &mut Painter, b: Rect, ink: crate::colour::Colour) {
    let at = |x: f32, y: f32| (b.x + x * b.w, b.y + y * b.h);
    let mut pb = tiny_skia::PathBuilder::new();
    let (a, m, c) = (at(0.18, 0.52), at(0.42, 0.76), at(0.84, 0.26));
    pb.move_to(a.0, a.1);
    pb.line_to(m.0, m.1);
    pb.line_to(c.0, c.1);
    if let Some(path) = pb.finish() {
        let line = tiny_skia::Stroke { width: (b.w / 8.0).max(1.0), line_cap: tiny_skia::LineCap::Round, line_join: tiny_skia::LineJoin::Round, ..tiny_skia::Stroke::default() };
        p.stroke_path(path, line, ink);
    }
}

fn catcher<M: 'static>(right: bool, close: impl Fn() -> M + 'static) -> Node<M> {
    drawing((0.0, 0.0), "", |_, _| {})
        .width(Length::Fill)
        .height(Length::Fill)
        .on_pointer_when(move |p| (matches!(p, Pointer::Down(..) | Pointer::Menu(..) | Pointer::Middle(..)) || right && matches!(p, Pointer::RightDown(..))).then(&close))
        .on_keys(|_| None)
        .unseen()
}

pub fn dismissing<M: Clone + 'static>(page: Node<M>, panel: Node<M>, close: M, _stay: M) -> Node<M> {
    layers(vec![page, catcher(true, move || close.clone()), panel])
}

pub fn under_the_i<M>(panel: Node<M>, at: Rect) -> Node<M> {
    placed(panel, move |(w, h), (aw, ah)| {
        let x = at.x.min(aw - w - 8.0).max(8.0);
        let below = at.y + at.h + 8.0;
        let y = if below + h > ah - 8.0 { (at.y - h - 8.0).max(8.0) } else { below };
        (x, y)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn three() -> Popup {
        Popup::new(vec![row("a", "Open"), row("b", "Copy").keys("Ctrl+C"), row("c", "Delete").rule()], Place::At(10.0, 10.0))
    }

    #[test]
    fn a_rows_top_is_inside_the_row_the_point_finds_there() {
        let rules = [false, false, true, false];
        for i in 0..rules.len() {
            assert_eq!(row_at(&rules, row_top(&rules, i) + 1.0), Some(i));
        }
    }

    #[test]
    fn the_lit_row_moves_round_and_return_chooses_it() {
        let mut p = three();
        assert_eq!(p.update(PopupMsg::PickHovered), Done::Open);
        assert_eq!(p.update(PopupMsg::Move(1)), Done::Open);
        assert_eq!(p.hover, Some(0));
        p.update(PopupMsg::Move(-1));
        assert_eq!(p.hover, Some(2));
        p.update(PopupMsg::Move(1));
        assert_eq!(p.hover, Some(0));
        assert_eq!(p.update(key(Keysym::Return)), Done::Picked(0));
        assert_eq!(p.update(key(Keysym::Escape)), Done::Closed);
        assert_eq!(p.update(key(Keysym::a)), Done::Open);
        assert_eq!(p.update(PopupMsg::Pick(9)), Done::Open);
        assert_eq!(p.update(PopupMsg::Hover(Some(9))), Done::Open);
        assert_eq!(p.hover, None);
        assert_eq!(three().lit().hover, Some(0));
        assert_eq!(Popup::new(Vec::new(), Place::At(0.0, 0.0)).moved(1), None);
    }

    #[test]
    fn a_letter_lights_the_next_row_that_starts_with_it_going_round_and_never_chooses() {
        let mut p = Popup::new(vec![row("", "Copy"), row("", "Paste"), row("", "Cut"), row("", "Select all")], Place::At(0.0, 0.0));
        assert_eq!(p.update(PopupMsg::Letter('c')), Done::Open);
        assert_eq!(p.hover, Some(0));
        p.update(PopupMsg::Letter('C'));
        assert_eq!(p.hover, Some(2));
        p.update(PopupMsg::Letter('c'));
        assert_eq!(p.hover, Some(0));
        p.update(PopupMsg::Letter('s'));
        assert_eq!(p.hover, Some(3));
        p.update(PopupMsg::Letter('z'));
        assert_eq!(p.hover, Some(3), "no row starts with it, so nothing moves");
        assert_eq!(Popup::new(Vec::new(), Place::At(0.0, 0.0)).update(PopupMsg::Letter('a')), Done::Open);
    }

    #[test]
    fn a_letter_goes_to_the_sub_menu_when_the_keyboard_is_in_it() {
        let mut p = nested().lit();
        p.update(key(Keysym::Down));
        p.update(key(Keysym::Right));
        assert_eq!((p.beside.open, p.beside.hover), (Some(1), Some(0)));
        p.update(PopupMsg::Letter('c'));
        assert_eq!((p.beside.open, p.beside.hover, p.hover), (Some(1), Some(2), Some(1)));
        p.update(PopupMsg::Letter('s'));
        assert_eq!(p.beside.hover, Some(0));
        p.update(key(Keysym::Left));
        p.update(PopupMsg::Letter('t'));
        assert_eq!((p.hover, p.beside.open), (Some(3), None));
    }

    #[test]
    fn typed_takes_one_plain_letter_or_figure_only() {
        use crate::input::Mods;
        let none = Mods::default();
        assert_eq!(typed(Some("a"), none), PopupMsg::Letter('a'));
        assert_eq!(typed(Some("7"), none), PopupMsg::Letter('7'));
        assert_eq!(typed(Some(" "), none), PopupMsg::Stay);
        assert_eq!(typed(Some("ab"), none), PopupMsg::Stay);
        assert_eq!(typed(None, none), PopupMsg::Stay);
        assert_eq!(typed(Some("a"), Mods { ctrl: true, ..none }), PopupMsg::Stay);
        assert_eq!(typed(Some("a"), Mods { alt: true, ..none }), PopupMsg::Stay);
    }

    #[test]
    fn a_radio_row_is_a_check_row_drawn_with_a_dot_and_not_a_tick() {
        let on = row("", "Small").radio(true);
        assert_eq!((on.check, on.radio), (Some(true), true));
        assert!(!row("", "Small").check(true).radio);
        let shot = |r: Row| {
            let mut pm = tiny_skia::Pixmap::new(200, 40).unwrap();
            let theme = crate::theme::Theme::new(true, "#fbc711");
            let fonts = crate::text::Fonts::new();
            let icons = crate::icon::Icons::new();
            paint_menu(&mut pm.as_mut(), &Pen { theme: &theme, fonts: &fonts, icons: &icons, scale: 1.0 }, &[r], None);
            pm.data().to_vec()
        };
        assert_ne!(shot(row("", "Small").radio(true)), shot(row("", "Small").check(true)));
        assert_ne!(shot(row("", "Small").radio(true)), shot(row("", "Small").radio(false)));
    }

    #[test]
    fn rows_are_found_under_the_pointer_past_rules() {
        let rules = [false, false, true];
        assert_eq!(row_at(&rules, 1.0), None);
        assert_eq!(row_at(&rules, theme::sheet().menu_padding + 1.0), Some(0));
        assert_eq!(row_at(&rules, theme::sheet().menu_padding + theme::sheet().menu_row + 1.0), Some(1));
        assert_eq!(row_at(&rules, theme::sheet().menu_padding + 2.0 * theme::sheet().menu_row + 2.0), None, "on the rule");
        assert_eq!(row_at(&rules, theme::sheet().menu_padding + 2.0 * theme::sheet().menu_row + rule() + 1.0), Some(2));
        assert_eq!(three().height(), 2.0 * theme::sheet().menu_padding + 3.0 * theme::sheet().menu_row + rule());
    }

    #[test]
    fn a_long_menu_is_fitted_into_a_short_window_by_less_tall_rows() {
        let rows: Vec<Row> = (0..22).map(|i| if i % 4 == 0 && i > 0 { row("a", format!("Row {i}")).rule() } else { row("a", format!("Row {i}")) }).collect();
        let rules: Vec<bool> = rows.iter().map(|r| r.rule).collect();
        let long = Popup::new(rows, Place::At(300.0, 200.0));
        assert!(long.height() > 760.0, "too long for the window as it comes");
        let fitted = long.clone().fits(760.0);
        assert!(fitted.height() <= 760.0 - 2.0 * edge(), "{}", fitted.height());
        assert!(fitted.row_height() < theme::sheet().menu_row && fitted.row_height() >= least_row());
        let (_, y) = spot(fitted.place, (fitted.width, fitted.height()), (1200.0, 760.0));
        assert!(y >= edge() && y + fitted.height() <= 760.0);
        assert_eq!(row_of(&rules, fitted.height() - theme::sheet().menu_padding - 1.0, fitted.row_height()), Some(21));
        assert_eq!(row_of(&rules, theme::sheet().menu_padding + 1.0, fitted.row_height()), Some(0));
        assert_eq!(three().fits(760.0).height(), three().height());
        assert_eq!(long.fits(100.0).row_height(), least_row());
    }

    #[test]
    fn a_menu_at_a_point_stays_inside_the_window() {
        let room = (800.0, 600.0);
        assert_eq!(spot(Place::At(100.0, 100.0), (200.0, 150.0), room), (100.0, 100.0));
        assert_eq!(spot(Place::At(100.0, 500.0), (200.0, 150.0), room), (100.0, 350.0));
        assert_eq!(spot(Place::At(700.0, 100.0), (200.0, 150.0), room), (800.0 - 200.0 - edge(), 100.0));
    }

    fn nested() -> Popup {
        let send = vec![row("a", "Send to"), row("b", "Share by email"), row("c", "Copy location").rule()];
        let tools = vec![row("a", "Open in a terminal").keys("F4"), row("b", "Find duplicates")];
        Popup::new(vec![row("a", "Open"), row("b", "Send and share").sub(send, 0.0), row("c", "Copy").keys("Ctrl+C"), row("d", "Tools").sub(tools, 240.0).rule()], Place::At(10.0, 10.0))
    }

    #[test]
    fn the_keys_open_a_sub_menu_move_in_it_and_shut_just_it() {
        let mut p = nested().lit();
        assert_eq!(p.update(key(Keysym::Right)), Done::Open);
        assert_eq!(p.beside.open, None);
        p.update(key(Keysym::Down));
        assert_eq!(p.said(), "Send and share, sub-menu, 3 lines");
        assert_eq!(p.update(key(Keysym::Right)), Done::Open);
        assert_eq!((p.beside.open, p.beside.hover, p.hover), (Some(1), Some(0), Some(1)));
        assert_eq!(p.said(), "Send to");
        p.update(key(Keysym::Up));
        assert_eq!((p.beside.hover, p.hover), (Some(2), Some(1)));
        p.update(key(Keysym::Down));
        p.update(key(Keysym::Down));
        assert_eq!(p.beside.hover, Some(1));
        assert_eq!(p.clone().update(key(Keysym::Return)), Done::PickedIn(1, 1));
        assert_eq!(p.clone().update(key(Keysym::space)), Done::PickedIn(1, 1));
        assert_eq!(p.update(key(Keysym::Left)), Done::Open);
        assert_eq!((p.beside.open, p.beside.hover, p.hover), (None, None, Some(1)));
        assert_eq!(p.update(key(Keysym::Left)), Done::Open);
        assert_eq!(p.update(key(Keysym::Return)), Done::Open);
        assert_eq!((p.beside.open, p.beside.hover), (Some(1), Some(0)));
        assert_eq!(p.update(key(Keysym::Escape)), Done::Open);
        assert_eq!((p.beside.open, p.hover), (None, Some(1)));
        assert_eq!(p.update(key(Keysym::Escape)), Done::Closed);
        p.update(key(Keysym::Right));
        assert_eq!(p.clone().update(key(Keysym::Menu)), Done::Closed);
        assert_eq!(p.update(PopupMsg::Close), Done::Closed);
        let mut p = nested().lit();
        p.update(PopupMsg::Move(2));
        assert_eq!(p.said(), "Copy, Ctrl+C");
        assert_eq!(p.update(key(Keysym::Return)), Done::Picked(2));
        assert_eq!(row("a", "One").sub(vec![row("b", "Only")], 0.0).said(), "One, sub-menu, 1 line");
        assert_eq!(nested().said(), "");
    }

    #[test]
    fn a_pause_begun_in_one_menu_opens_nothing_in_the_next() {
        let now = Instant::now();
        let at = |row: usize| PopupMsg::Point { row: Some(row), x: 20, y: 20 + 30 * row as i32, toward: false };
        let mut first = nested();
        first.update_at(at(1), now);
        let Some((old, 1)) = first.beside.due else { panic!("{:?}", first.beside.due) };
        let mut next = nested();
        next.update_at(at(3), now);
        let Some((number, 3)) = next.beside.due else { panic!("{:?}", next.beside.due) };
        assert_ne!(number, old, "two menus gave their pauses the same number");
        next.update_at(PopupMsg::Due(old), now);
        assert_eq!(next.beside.open, None, "the timer of a menu since closed opened a sub-menu early");
        next.update_at(PopupMsg::Due(number), now);
        assert_eq!(next.beside.open, Some(3), "its own pause opens it");
    }

    #[test]
    fn a_click_or_a_touch_opens_a_sub_menu_at_once_and_pointing_after_a_pause() {
        let now = Instant::now();
        let at = |row: usize| PopupMsg::Point { row: Some(row), x: 20, y: 20 + 30 * row as i32, toward: false };
        let mut p = nested();
        assert_eq!(p.update(PopupMsg::Pick(3)), Done::Open);
        assert_eq!((p.beside.open, p.beside.hover, p.hover, p.beside.due), (Some(3), None, Some(3), None));
        assert_eq!(p.update(PopupMsg::SubPick(1)), Done::PickedIn(3, 1));
        assert_eq!(p.update(PopupMsg::SubPick(7)), Done::Open);
        let mut p = nested();
        p.update_at(at(0), now);
        assert_eq!(p.beside.due, None, "a row with no sub-menu waits for nothing");
        p.update_at(at(1), now);
        let Some((number, 1)) = p.beside.due else { panic!("{:?}", p.beside.due) };
        assert_eq!((p.beside.open, p.hover), (None, Some(1)));
        p.update_at(at(1), now);
        assert_eq!(p.beside.due, Some((number, 1)));
        p.update_at(PopupMsg::Due(number + 9), now);
        assert_eq!(p.beside.open, None);
        p.update_at(PopupMsg::Due(number), now);
        assert_eq!((p.beside.open, p.beside.hover, p.beside.due), (Some(1), None, None));
        let mut q = nested();
        q.update_at(at(1), now);
        q.update_at(at(0), now);
        assert_eq!(q.beside.due, None);
        q.update_at(PopupMsg::Due(1), now);
        assert_eq!(q.beside.open, None);
        p.update_at(PopupMsg::Hover(None), now);
        assert_eq!(p.hover, Some(1), "the pointer left the menu for the sub-menu");
        p.update_at(PopupMsg::SubHover(Some(2)), now);
        assert_eq!((p.hover, p.beside.hover), (Some(1), Some(2)));
        assert_eq!(p.said(), "Copy location");
        p.update_at(PopupMsg::SubHover(Some(9)), now);
        assert_eq!(p.beside.hover, None);
        p.update_at(at(2), now);
        let Some((number, 2)) = p.beside.due else { panic!("{:?}", p.beside.due) };
        assert_eq!((p.beside.open, p.hover), (Some(1), Some(2)));
        let mut other = p.clone();
        p.update_at(PopupMsg::Due(number), now);
        assert_eq!((p.beside.open, p.hover), (None, Some(2)));
        other.update_at(at(3), now);
        let Some((number, 3)) = other.beside.due else { panic!("{:?}", other.beside.due) };
        other.update_at(PopupMsg::Due(number), now);
        assert_eq!((other.beside.open, other.hover), (Some(3), Some(3)));
        let mut back = nested();
        back.update(PopupMsg::Pick(1));
        back.update_at(at(2), now);
        back.update_at(at(1), now);
        assert_eq!((back.beside.due, back.beside.open), (None, Some(1)));
        back.update(key(Keysym::Down));
        assert_eq!((back.beside.open, back.hover), (None, Some(2)));
    }

    #[test]
    fn the_pointer_can_cross_other_rows_on_its_way_to_the_sub_menu() {
        let now = Instant::now();
        let pause = sub_pause();
        let mut p = nested();
        p.update(PopupMsg::Pick(1));
        p.update_at(PopupMsg::Point { row: Some(2), x: 120, y: 90, toward: true }, now);
        assert_eq!((p.beside.open, p.hover), (Some(1), Some(1)));
        let Some((first, 2)) = p.beside.due else { panic!("{:?}", p.beside.due) };
        p.update_at(PopupMsg::Point { row: Some(2), x: 200, y: 96, toward: true }, now + pause / 2);
        p.update_at(PopupMsg::Due(first), now + pause);
        assert_eq!(p.beside.open, Some(1), "still on its way");
        let Some((second, 2)) = p.beside.due else { panic!("{:?}", p.beside.due) };
        assert_ne!(first, second);
        let mut there = p.clone();
        there.update_at(PopupMsg::SubHover(Some(1)), now + pause);
        assert_eq!((there.beside.due, there.beside.open, there.hover), (None, Some(1), Some(1)));
        there.update_at(PopupMsg::Due(second), now + pause * 3);
        assert_eq!(there.beside.open, Some(1));
        p.update_at(PopupMsg::Due(second), now + pause * 2);
        assert_eq!((p.beside.open, p.hover), (None, Some(2)));

        let sub = Some(Rect { x: 268.0, y: 40.0, w: 272.0, h: 120.0 });
        let slack = theme::sheet().menu_row;
        assert!(toward(Some((100, 60)), (140.0, 70.0), sub, slack));
        assert!(toward(Some((100, 60)), (140.0, 90.0), sub, slack), "down across the rows, towards its foot");
        assert!(!toward(Some((100, 60)), (140.0, 160.0), sub, slack), "too steeply down: it would pass under");
        assert!(!toward(Some((100, 60)), (60.0, 62.0), sub, slack), "away from it");
        assert!(!toward(Some((100, 60)), (100.0, 90.0), sub, slack), "straight down");
        assert!(!toward(None, (140.0, 70.0), sub, slack) && !toward(Some((100, 60)), (140.0, 70.0), None, slack));
        let sub = Some(Rect { x: -268.0, y: 40.0, w: 272.0, h: 120.0 });
        assert!(toward(Some((100, 60)), (60.0, 70.0), sub, slack));
        assert!(!toward(Some((100, 60)), (140.0, 70.0), sub, slack));
    }

    #[test]
    fn a_sub_menu_stays_inside_the_window_at_every_corner() {
        let room = (900.0, 620.0);
        let inside = |r: Rect| r.x >= edge() && r.y >= edge() && r.x + r.w <= room.0 - edge() + 0.01 && r.y + r.h <= room.1 - edge() + 0.01;
        for (x, y) in [(0.0, 0.0), (890.0, 0.0), (0.0, 610.0), (890.0, 610.0), (450.0, 300.0), (500.0, 10.0), (10.0, 500.0)] {
            for open in [1, 3] {
                let mut p = Popup { place: Place::At(x, y), ..nested() }.fits(room.1);
                p.update(PopupMsg::Pick(open));
                let (menu, sub) = p.spots(room);
                let sub = sub.expect("an open sub-menu has a place");
                assert!(inside(sub), "{x} {y} {open}: {sub:?}");
                assert!(menu.x >= edge() && menu.x + menu.w <= room.0 - edge() + 0.01, "{x} {y}: {menu:?}");
                let lap = theme::sheet().menu_sub_overlap;
                let (right, left) = ((sub.x - (menu.x + menu.w - lap)).abs() < 0.01, (sub.x + sub.w - (menu.x + lap)).abs() < 0.01);
                assert!(right || left, "{x} {y} {open}: {menu:?} {sub:?}");
                assert_eq!(right, menu.x + menu.w - lap + sub.w + edge() <= room.0, "to the right whenever there is room: {x} {y}");
            }
        }
        let mut p = nested();
        p.update(PopupMsg::Pick(1));
        let (menu, sub) = p.spots(room);
        assert_eq!(sub.unwrap().y, menu.y + theme::sheet().menu_row);
        assert_eq!(sub.unwrap().w, menu.w, "as wide as its menu unless it asks");
        p.update(PopupMsg::Pick(3));
        assert_eq!(p.spots(room).1.unwrap().w, 240.0);
        let mut low = Popup { place: Place::At(100.0, 480.0), ..nested() };
        low.update(PopupMsg::Pick(3));
        let (menu, sub) = low.spots(room);
        let sub = sub.unwrap();
        assert!(sub.y + sub.h <= room.1 - edge() && sub.y >= menu.y, "{menu:?} {sub:?}");
        let tight = (300.0, 400.0);
        let mut p = Popup { place: Place::At(10.0, 10.0), ..nested() };
        p.update(PopupMsg::Pick(1));
        let sub = p.spots(tight).1.unwrap();
        assert!(sub.x >= edge() && sub.x + sub.w <= tight.0 - edge() + 0.01, "{sub:?}");
        assert_eq!(nested().spots(room).1, None);
    }

    fn a_sub_menu_arrow_is_painted_at_the_end_of_the_row() {
        use super::super::icon::Icons;
        use super::super::text::Fonts;
        use super::super::theme::Theme;
        let (t, fonts, icons) = (Theme::new(true, "#fbc711"), Fonts::new(), Icons::new());
        let pen = Pen { theme: &t, fonts: &fonts, icons: &icons, scale: 1.0 };
        let inked = |rows: &[Row]| {
            let p = Popup::new(rows.to_vec(), Place::At(0.0, 0.0));
            let mut pix = tiny_skia::Pixmap::new(p.width as u32, p.height() as u32).unwrap();
            paint_menu(&mut pix.as_mut(), &pen, &p.rows, None);
            let (w, pad, tall) = (p.width as u32, theme::sheet().menu_padding as u32, theme::sheet().menu_row as u32);
            let first = pix.pixel(w - 40, pad + 2).unwrap();
            (w - 40..w - pad).flat_map(|x| (pad..pad + tall).map(move |y| (x, y))).filter(|&(x, y)| pix.pixel(x, y).unwrap() != first).count()
        };
        assert_eq!(inked(&[row("a", "Tools")]), 0);
        assert!(
            icons.get(ARROW, theme::sheet().menu_sub_arrow as u32, 1, t.muted).is_some(),
            "this test checks nothing without an icon set that has {ARROW}: install Debian's adwaita-icon-theme"
        );
        assert!(inked(&[row("a", "Tools").sub(vec![row("b", "One"), row("c", "Two")], 0.0)]) > 0, "the arrow");
    }

        fn shown(p: &Popup, room: (f32, f32)) -> super::super::ui::Ui<PopupMsg> {
            let mut ui = super::super::ui::Ui::new(p.over(super::super::widget::label("under"), |m| m), super::super::theme::Theme::new(true, "#fbc711"));
            let mut pix = tiny_skia::Pixmap::new(room.0 as u32, room.1 as u32).unwrap();
            ui.paint(&mut pix.as_mut(), 1.0);
            ui
        }
        fn send(p: &mut Popup, room: (f32, f32), input: super::super::input::Input) -> Vec<Done> {
            let mut ui = shown(p, room);
            ui.input(&input, Instant::now()).messages.into_iter().map(|m| p.update(m)).collect()
        }
        fn click(p: &mut Popup, room: (f32, f32), (x, y): (f32, f32)) -> Vec<Done> {
        use super::super::input::{Button, Input};
            let mut ui = shown(p, room);
            let mut got = Vec::new();
            for input in [Input::PointerMove { x, y }, Input::Press { button: Button::Left, x, y }, Input::Release { button: Button::Left, x, y }] {
                got.extend(ui.input(&input, Instant::now()).messages);
            }
            got.into_iter().map(|m| p.update(m)).collect()
        }

    #[test]
    fn a_row_with_a_sub_menu_shows_an_arrow_and_is_pressed_through_the_window() {
        use super::super::input::Input;
        a_sub_menu_arrow_is_painted_at_the_end_of_the_row();
        let room = (900.0, 620.0);
        let mut p = Popup { place: Place::At(300.0, 200.0), ..nested() };
        let (menu, _) = p.spots(room);
        let rules: Vec<bool> = p.rows.iter().map(|r| r.rule).collect();
        let on = |i: usize| (menu.x + 60.0, menu.y + top_of(&rules, i, theme::sheet().menu_row) + theme::sheet().menu_row / 2.0);
        let (x, y) = on(1);
        send(&mut p, room, Input::PointerMove { x, y });
        assert_eq!((p.hover, p.beside.open), (Some(1), None));
        assert!(matches!(p.beside.due, Some((_, 1))), "pointing begins the pause: {:?}", p.beside.due);
        click(&mut p, room, (x, y));
        assert_eq!(p.beside.open, Some(1), "a click opens it at once");
        let sub = p.spots(room).1.unwrap();
        let (bx, by) = on(2);
        send(&mut p, room, Input::PointerMove { x: x + 40.0, y: y + 8.0 });
        send(&mut p, room, Input::PointerMove { x: bx + 110.0, y: by });
        assert_eq!((p.hover, p.beside.open), (Some(1), Some(1)), "on its way across the row below");
        let mut down = p.clone();
        send(&mut down, room, Input::PointerMove { x: bx + 110.0, y: by + 10.0 });
        assert_eq!(down.hover, Some(2));
        let (sx, sy) = (sub.x + 60.0, sub.y + theme::sheet().menu_padding + theme::sheet().menu_row * 1.5);
        send(&mut p, room, Input::PointerMove { x: sx, y: sy });
        assert_eq!((p.hover, p.beside.hover, p.beside.due), (Some(1), Some(1), None));
        assert_eq!(click(&mut p, room, (sx, sy)).last(), Some(&Done::PickedIn(1, 1)));
        let mut p = Popup { place: Place::At(300.0, 200.0), ..nested() };
        let (x, y) = on(3);
        let mut ui = shown(&p, room);
        let mut got = ui.input(&Input::TouchDown { id: 1, x, y }, Instant::now()).messages;
        got.extend(ui.input(&Input::TouchUp { id: 1 }, Instant::now()).messages);
        for m in got {
            p.update(m);
        }
        assert_eq!(p.beside.open, Some(3), "a touch opens it");
        let sub = p.spots(room).1.unwrap();
        let mut ui = shown(&p, room);
        let mut got = ui.input(&Input::TouchDown { id: 2, x: sub.x + 50.0, y: sub.y + theme::sheet().menu_padding + 5.0 }, Instant::now()).messages;
        got.extend(ui.input(&Input::TouchUp { id: 2 }, Instant::now()).messages);
        assert_eq!(got.into_iter().map(|m| p.update(m)).last(), Some(Done::PickedIn(3, 0)));
        let mut p = Popup { place: Place::At(300.0, 200.0), ..nested() };
        p.update(PopupMsg::Pick(1));
        assert!(click(&mut p, room, (20.0, 20.0)).contains(&Done::Closed));
        let mut p = nested().lit();
        p.update(key(Keysym::Down));
        let hears = |p: &Popup, words: &str| {
            let t = shown(p, room).access("Menu");
            t.nodes.iter().any(|(id, part)| *id == t.focus && part.label() == Some(words))
        };
        assert!(hears(&p, "Send and share, sub-menu, 3 lines"));
        p.update(key(Keysym::Right));
        assert!(hears(&p, "Send to") && !hears(&p, "Send and share, sub-menu, 3 lines"));
    }

    #[test]
    fn the_menu_paints_its_rows() {
        use super::super::icon::Icons;
        use super::super::text::Fonts;
        use super::super::theme::Theme;
        let (t, fonts, icons) = (Theme::new(true, "#fbc711"), Fonts::new(), Icons::new());
        let pen = Pen { theme: &t, fonts: &fonts, icons: &icons, scale: 1.0 };
        let p = three();
        let mut pix = tiny_skia::Pixmap::new(p.width as u32, p.height() as u32).unwrap();
        paint_menu(&mut pix.as_mut(), &pen, &p.rows, Some(1));
        let near = |want: tiny_skia::ColorU8| pix.pixels().iter().any(|c| {
            let c = c.demultiply();
            (c.red() as i32 - want.red() as i32).abs() < 30 && (c.green() as i32 - want.green() as i32).abs() < 30
                && (c.blue() as i32 - want.blue() as i32).abs() < 30 && c.alpha() > 200
        });
        assert!(near(t.fg.to_skia().to_color_u8()), "the words are drawn");
        assert!(!near(t.danger.to_skia().to_color_u8()), "no row is red");
    }
}
