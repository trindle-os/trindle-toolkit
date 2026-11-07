//! A bar of tabs a program paints itself, with close boxes, reordering and a menu for those out of reach.

use tiny_skia::{FillRule, LineCap, PathBuilder, PixmapMut, Stroke};

use super::colour::Colour;
use super::draw::{fill_on, text_in_on};
use super::paint::Painter;
use super::input::Cursor;
use super::theme::sheet;
use super::widget::{Pen, Pointer, Rect, SaidTab, SaidTabs, Words};

pub(crate) fn more() -> f32 {
    sheet().tab_more
}

pub(crate) fn pad() -> f32 {
    sheet().tab_pad
}

pub(crate) fn close_box(r: Rect) -> Rect {
    let close = sheet().tab_close;
    Rect { x: r.x + r.w - close - 4.0, y: r.y + (r.h - close) / 2.0, w: close, h: close }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Tab {
    pub title: String,
    pub count: Option<usize>,
    pub closable: bool,
}

impl Tab {
    pub fn new(title: impl Into<String>) -> Tab {
        Tab { title: title.into(), count: None, closable: true }
    }

    pub fn count(mut self, n: usize) -> Tab {
        self.count = Some(n);
        self
    }

    pub fn fixed(mut self) -> Tab {
        self.closable = false;
        self
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Laid {
    pub shown: Vec<(usize, Rect)>,
    pub more: Option<Rect>,
    pub hidden: Vec<usize>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Tabs {
    pub tabs: Vec<Tab>,
    pub chosen: usize,
    pub first: usize,
    pub height: f32,
    pub least: f32,
    pub most: f32,
}

fn reach(first: usize, chosen: usize, fits: usize, n: usize) -> usize {
    let start = first.min(n - fits);
    if chosen < start {
        chosen
    } else if chosen >= start + fits {
        chosen + 1 - fits
    } else {
        start
    }
}

impl Tabs {
    pub fn new(tabs: Vec<Tab>) -> Tabs {
        let s = sheet();
        Tabs { tabs, chosen: 0, first: 0, height: s.tab_height, least: s.tab_least, most: s.tab_most }
    }

    pub fn off_bar(&self, y: f32) -> bool {
        !(-sheet().tab_tear..=self.height + sheet().tab_tear).contains(&y)
    }

    pub fn len(&self) -> usize {
        self.tabs.len()
    }

    pub fn is_empty(&self) -> bool {
        self.tabs.is_empty()
    }

    fn fits(&self, width: f32) -> usize {
        (((width - more()) / self.least).floor().max(1.0)) as usize
    }

    pub fn layout(&self, width: f32) -> Laid {
        let n = self.tabs.len();
        if n == 0 {
            return Laid { shown: Vec::new(), more: None, hidden: Vec::new() };
        }
        if n as f32 * self.least <= width {
            let each = (width / n as f32).min(self.most);
            let shown = (0..n).map(|i| (i, Rect { x: i as f32 * each, y: 0.0, w: each, h: self.height })).collect();
            return Laid { shown, more: None, hidden: Vec::new() };
        }
        let fits = self.fits(width).min(n);
        let chosen = self.chosen.min(n - 1);
        let start = reach(self.first, chosen, fits, n);
        let each = ((width - more()) / fits as f32).clamp(0.0, self.most);
        let shown = (start..start + fits).map(|i| (i, Rect { x: (i - start) as f32 * each, y: 0.0, w: each, h: self.height })).collect();
        let hidden: Vec<usize> = (0..start).chain(start + fits..n).collect();
        let more = (!hidden.is_empty()).then_some(Rect { x: width - more(), y: 0.0, w: more(), h: self.height });
        Laid { shown, more, hidden }
    }

    pub fn reveal(&mut self, width: f32) {
        let n = self.tabs.len();
        if n == 0 || n as f32 * self.least <= width {
            self.first = 0;
            return;
        }
        let fits = self.fits(width).min(n);
        self.chosen = self.chosen.min(n - 1);
        self.first = reach(self.first, self.chosen, fits, n);
    }

    pub fn close_rect(&self, i: usize, r: Rect) -> Option<Rect> {
        self.tabs.get(i)?.closable.then(|| close_box(r))
    }

    pub fn tab_at(&self, width: f32, x: f32, y: f32) -> Option<usize> {
        self.layout(width).shown.iter().find(|(_, r)| r.contains(x, y)).map(|&(i, _)| i)
    }

    pub fn choose(&mut self, i: usize, width: f32) {
        if i < self.tabs.len() {
            self.chosen = i;
            self.reveal(width);
        }
    }

    pub fn step(&mut self, by: isize, width: f32) -> Option<usize> {
        if self.tabs.is_empty() {
            return None;
        }
        let to = (self.chosen as isize + by).clamp(0, self.tabs.len() as isize - 1) as usize;
        self.choose(to, width);
        Some(to)
    }

    pub fn shift(&mut self, by: isize, width: f32) -> Option<Touch> {
        let from = self.chosen;
        let to = from.checked_add_signed(by).filter(|&to| to < self.tabs.len())?;
        self.reorder(from, to);
        self.reveal(width);
        Some(Touch::Moved(from, to))
    }

    pub fn open(&mut self, tab: Tab, width: f32) -> usize {
        self.tabs.push(tab);
        let at = self.tabs.len() - 1;
        self.choose(at, width);
        at
    }

    pub fn close(&mut self, i: usize) -> Option<usize> {
        if i >= self.tabs.len() {
            return (!self.tabs.is_empty()).then_some(self.chosen);
        }
        self.tabs.remove(i);
        if self.tabs.is_empty() {
            self.chosen = 0;
            self.first = 0;
            return None;
        }
        if i < self.chosen {
            self.chosen -= 1;
        }
        self.chosen = self.chosen.min(self.tabs.len() - 1);
        self.first = self.first.min(self.tabs.len() - 1);
        Some(self.chosen)
    }

    pub fn reorder(&mut self, from: usize, to: usize) {
        let n = self.tabs.len();
        if from >= n || to >= n || from == to {
            return;
        }
        let tab = self.tabs.remove(from);
        self.tabs.insert(to, tab);
        self.chosen = if self.chosen == from {
            to
        } else {
            let mut c = self.chosen;
            if from < c {
                c -= 1;
            }
            if to <= c {
                c += 1;
            }
            c
        };
    }

    pub fn scroll(&mut self, by: isize, width: f32) {
        let n = self.tabs.len();
        if n as f32 * self.least <= width {
            return;
        }
        let most = n - self.fits(width).min(n);
        self.first = (self.first as isize + by).clamp(0, most as isize) as usize;
    }

    pub fn said(&self, width: f32) -> SaidTabs {
        let laid = self.layout(width);
        SaidTabs {
            total: self.tabs.len(),
            tabs: laid.shown.iter().map(|&(i, r)| SaidTab {
                index: i,
                words: match self.tabs[i].count {
                    Some(n) => crate::t!("kit-tab-count", title = self.tabs[i].title.as_str(), count = n),
                    None => self.tabs[i].title.clone(),
                },
                chosen: i == self.chosen,
                closable: self.tabs[i].closable,
                x: r.x,
                width: r.w,
            }).collect(),
            height: self.height,
            more: laid.more.is_some(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Touch {
    Chose(usize),
    Closed(usize),
    Dragging(usize, usize),
    Moved(usize, usize),
    More,
    Tearing(usize),
    TornOff(usize),
}

#[derive(Clone, Copy, Debug)]
enum Down {
    Tab { index: usize, at: (f32, f32), dragging: Option<usize> },
    Close(usize),
    More,
}

#[derive(Clone, Debug, Default)]
pub struct Gesture {
    down: Option<Down>,
    at: Option<(f32, f32)>,
}

impl Gesture {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn pointer(&mut self, tabs: &mut Tabs, event: Pointer, width: f32) -> Option<Touch> {
        let laid = tabs.layout(width);
        match event {
            Pointer::Down(x, y) => {
                self.at = Some((x, y));
                self.press(tabs, &laid, (x, y), width)
            }
            Pointer::Move(x, y) => {
                self.at = Some((x, y));
                let Some(Down::Tab { index, at, dragging }) = &mut self.down else { return None };
                let started = dragging.is_some() || (x - at.0).hypot(y - at.1) >= sheet().tab_slop;
                if !started {
                    return None;
                }
                let to = Self::place(&laid, x, tabs.len(), *index);
                *dragging = Some(to);
                if tabs.off_bar(y) {
                    return Some(Touch::Tearing(*index));
                }
                Some(Touch::Dragging(*index, to))
            }
            Pointer::Up(x, y) => {
                self.at = Some((x, y));
                match self.down.take()? {
                    Down::Close(i) => laid.shown.iter().find(|&&(n, r)| n == i && tabs.close_rect(n, r).is_some_and(|c| c.contains(x, y))).map(|_| Touch::Closed(i)),
                    Down::More => laid.more.is_some_and(|r| r.contains(x, y)).then_some(Touch::More),
                    Down::Tab { index, dragging: Some(_), .. } if tabs.off_bar(y) => Some(Touch::TornOff(index)),
                    Down::Tab { index, dragging: Some(_), .. } => {
                        let to = Self::place(&laid, x, tabs.len(), index);
                        if to == index {
                            return None;
                        }
                        tabs.reorder(index, to);
                        tabs.reveal(width);
                        Some(Touch::Moved(index, to))
                    }
                    Down::Tab { .. } => None,
                }
            }
            Pointer::Middle(x, y) => {
                let i = laid.shown.iter().find(|(_, r)| r.contains(x, y)).map(|&(i, _)| i)?;
                tabs.tabs[i].closable.then_some(Touch::Closed(i))
            }
            Pointer::Hover(x, y) => {
                self.at = Some((x, y));
                None
            }
            Pointer::Leave => {
                self.at = None;
                None
            }
            _ => None,
        }
    }

    fn press(&mut self, tabs: &mut Tabs, laid: &Laid, (x, y): (f32, f32), width: f32) -> Option<Touch> {
        if laid.more.is_some_and(|r| r.contains(x, y)) {
            self.down = Some(Down::More);
            return None;
        }
        let &(i, r) = laid.shown.iter().find(|(_, r)| r.contains(x, y))?;
        if tabs.close_rect(i, r).is_some_and(|c| c.contains(x, y)) {
            self.down = Some(Down::Close(i));
            return None;
        }
        tabs.choose(i, width);
        self.down = Some(Down::Tab { index: i, at: (x, y), dragging: None });
        Some(Touch::Chose(i))
    }

    fn place(laid: &Laid, x: f32, n: usize, from: usize) -> usize {
        let (Some(&(first, _)), Some(&(last, _))) = (laid.shown.first(), laid.shown.last()) else { return from };
        let before = laid.shown.iter().filter(|(_, r)| r.x + r.w / 2.0 < x).count();
        (first + before).min(last).min(n - 1)
    }

    pub fn dragging(&self) -> Option<(usize, usize)> {
        match self.down? {
            Down::Tab { index, dragging: Some(to), .. } => Some((index, to)),
            _ => None,
        }
    }

    pub fn hovering(&self, tabs: &Tabs, width: f32) -> Option<usize> {
        let (x, y) = self.at?;
        tabs.tab_at(width, x, y)
    }

    pub fn cursor(&self, tabs: &Tabs, width: f32, x: f32, y: f32) -> Cursor {
        let laid = tabs.layout(width);
        let over_close = laid.shown.iter().any(|&(i, r)| tabs.close_rect(i, r).is_some_and(|c| c.contains(x, y)));
        if over_close || laid.more.is_some_and(|r| r.contains(x, y)) {
            Cursor::Hand
        } else {
            Cursor::Arrow
        }
    }
}

fn stroke(p: &mut Painter, pen: &Pen, from: (f32, f32), to: (f32, f32), colour: Colour, wide: f32) {
    let k = pen.scale;
    let mut pb = PathBuilder::new();
    pb.move_to(from.0 * k, from.1 * k);
    pb.line_to(to.0 * k, to.1 * k);
    let Some(path) = pb.finish() else { return };
    p.stroke_path(path, Stroke { width: wide * k, line_cap: LineCap::Round, ..Stroke::default() }, colour);
}

pub fn paint_tabs(canvas: &mut PixmapMut, pen: &Pen, tabs: &Tabs, hover: Option<usize>, dragging: Option<(usize, usize)>) {
    let mut p = Painter::default();
    paint_tabs_on(&mut p, pen, canvas.width(), tabs, hover, dragging);
    p.play(canvas);
}

pub fn paint_tabs_on<'a>(p: &mut Painter<'a>, pen: &Pen<'a>, wide: u32, tabs: &Tabs, hover: Option<usize>, dragging: Option<(usize, usize)>) {
    let k = pen.scale;
    let width = wide as f32 / k;
    let t = pen.theme;
    let style = Words::Plain.style(k);
    let laid = tabs.layout(width);
    let soft = |p: &mut Painter, r: Rect, colour: Colour, joined: bool| {
        let px = Rect { x: r.x * k, y: r.y * k, w: r.w * k, h: r.h * k };
        super::ui::fill_on(p, px, super::theme::corner_px(super::theme::Part::Tab, px.h, k), colour);
        if joined {
            fill_on(p, pen, Rect { y: r.y + r.h / 2.0, h: r.h / 2.0, ..r }, colour);
        }
    };
    for &(i, r) in &laid.shown {
        let chosen = i == tabs.chosen;
        if chosen {
            soft(p, r, t.card(1), true);
        } else if hover == Some(i) {
            soft(p, r, t.hover, false);
        }
        let close = tabs.close_rect(i, r);
        let tab = &tabs.tabs[i];
        let counted = if tab.count.is_some() { sheet().tab_count } else { 0.0 };
        let right = close.map_or(r.x + r.w - pad(), |c| c.x - 2.0);
        let ink = if chosen { t.fg } else { t.muted };
        let room = Rect { x: r.x + pad(), w: (right - counted - r.x - pad()).max(0.0), ..r };
        text_in_on(p, pen, &tab.title, &style, room, ink, false);
        if let Some(n) = tab.count {
            text_in_on(p, pen, &n.to_string(), &Words::Note.style(k), Rect { x: right - sheet().tab_count, w: sheet().tab_count, ..r }, t.muted, true);
        }
        if let Some(c) = close {
            let (cx, cy) = (c.x + c.w / 2.0, c.y + c.h / 2.0);
            let lit = hover == Some(i) || chosen;
            let ink = if lit { t.fg } else { t.muted };
            stroke(p, pen, (cx - 4.0, cy - 4.0), (cx + 4.0, cy + 4.0), ink, 1.5);
            stroke(p, pen, (cx - 4.0, cy + 4.0), (cx + 4.0, cy - 4.0), ink, 1.5);
        }
    }
    if let Some(m) = laid.more {
        let (cx, cy) = (m.x + m.w / 2.0, m.y + m.h / 2.0);
        let mut pb = PathBuilder::new();
        pb.move_to((cx - 5.0) * k, (cy - 2.5) * k);
        pb.line_to((cx + 5.0) * k, (cy - 2.5) * k);
        pb.line_to(cx * k, (cy + 3.0) * k);
        pb.close();
        if let Some(path) = pb.finish() {
            p.fill_path(path, FillRule::Winding, t.fg);
        }
    }
    if let Some((from, to)) = dragging.filter(|(f, to)| f != to) {
        if let Some(&(_, r)) = laid.shown.iter().find(|&&(i, _)| i == to) {
            let x = if to > from { r.x + r.w } else { r.x };
            fill_on(p, pen, Rect { x: x - 1.0, y: r.y + 4.0, w: 2.0, h: r.h - 8.0 }, t.accent);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tabs(n: usize) -> Tabs {
        Tabs::new((0..n).map(|i| Tab::new(format!("Tab {i}"))).collect())
    }

    fn at(g: &mut Gesture, t: &mut Tabs, e: Pointer, width: f32) -> Option<Touch> {
        g.pointer(t, e, width)
    }

    #[test]
    fn tabs_share_the_bar_up_to_the_widest() {
        let t = tabs(3);
        let l = t.layout(600.0);
        assert_eq!(l.shown.len(), 3);
        assert_eq!(l.shown[1].1, Rect { x: 200.0, y: 0.0, w: 200.0, h: 40.0 });
        assert!(l.more.is_none() && l.hidden.is_empty());
        let wide = t.layout(1200.0);
        assert_eq!(wide.shown[0].1.w, 240.0);
    }

    #[test]
    fn a_bar_narrower_than_the_button_draws_no_negative_tab() {
        let t = tabs(5);
        for w in [0.0, 10.0, 30.0] {
            let l = t.layout(w);
            assert!(l.shown.iter().all(|(_, r)| r.w >= 0.0), "{w}");
        }
    }

    #[test]
    fn tabs_that_do_not_fit_leave_a_button_and_a_hidden_list() {
        let mut t = tabs(10);
        let l = t.layout(500.0);
        let fits = l.shown.len();
        assert_eq!(fits, 4);
        assert!(l.more.is_some());
        assert_eq!(l.hidden.len(), 6);
        t.choose(9, 500.0);
        let l = t.layout(500.0);
        assert_eq!(l.shown.last().unwrap().0, 9);
        assert_eq!(t.first, 6);
        t.choose(2, 500.0);
        assert_eq!(t.first, 2);
    }

    #[test]
    fn a_press_chooses_and_the_close_box_closes() {
        let (mut t, mut g) = (tabs(3), Gesture::new());
        assert_eq!(at(&mut g, &mut t, Pointer::Down(250.0, 10.0), 600.0), Some(Touch::Chose(1)));
        assert_eq!(t.chosen, 1);
        at(&mut g, &mut t, Pointer::Up(250.0, 10.0), 600.0);
        let c = t.close_rect(1, t.layout(600.0).shown[1].1).unwrap();
        at(&mut g, &mut t, Pointer::Down(c.x + 5.0, c.y + 5.0), 600.0);
        assert_eq!(at(&mut g, &mut t, Pointer::Up(c.x + 5.0, c.y + 5.0), 600.0), Some(Touch::Closed(1)));
        at(&mut g, &mut t, Pointer::Down(c.x + 5.0, c.y + 5.0), 600.0);
        assert_eq!(at(&mut g, &mut t, Pointer::Up(20.0, 10.0), 600.0), None);
    }

    #[test]
    fn a_middle_click_closes_a_tab_that_can_be_closed() {
        let mut t = tabs(3);
        t.tabs[0] = Tab::new("Home").fixed();
        let mut g = Gesture::new();
        assert_eq!(at(&mut g, &mut t, Pointer::Middle(10.0, 10.0), 600.0), None);
        assert_eq!(at(&mut g, &mut t, Pointer::Middle(250.0, 10.0), 600.0), Some(Touch::Closed(1)));
        assert!(t.close_rect(0, t.layout(600.0).shown[0].1).is_none());
    }

    #[test]
    fn a_tab_dragged_well_off_the_bar_is_torn_off_and_the_bar_is_left_alone() {
        let (mut t, mut g) = (tabs(4), Gesture::new());
        at(&mut g, &mut t, Pointer::Down(250.0, 10.0), 600.0);
        assert!(matches!(at(&mut g, &mut t, Pointer::Move(250.0, 70.0), 600.0), Some(Touch::Dragging(1, _))), "a little below the bar is still a reorder");
        assert_eq!(at(&mut g, &mut t, Pointer::Move(300.0, 200.0), 600.0), Some(Touch::Tearing(1)));
        assert_eq!(at(&mut g, &mut t, Pointer::Up(300.0, 200.0), 600.0), Some(Touch::TornOff(1)));
        assert_eq!(t.tabs.iter().map(|t| t.title.as_str()).collect::<Vec<_>>(), ["Tab 0", "Tab 1", "Tab 2", "Tab 3"]);
        at(&mut g, &mut t, Pointer::Down(250.0, 10.0), 600.0);
        at(&mut g, &mut t, Pointer::Move(250.0, 200.0), 600.0);
        let home = t.layout(600.0).shown[1].1;
        assert_eq!(at(&mut g, &mut t, Pointer::Up(home.x + home.w / 2.0, 20.0), 600.0), None, "brought back to its place on the bar, it stays");
        assert!(t.off_bar(-60.0) && !t.off_bar(-40.0));
    }

    #[test]
    fn dragging_a_tab_reorders_and_the_chosen_goes_with_it() {
        let (mut t, mut g) = (tabs(4), Gesture::new());
        at(&mut g, &mut t, Pointer::Down(50.0, 10.0), 600.0);
        assert_eq!(at(&mut g, &mut t, Pointer::Move(52.0, 10.0), 600.0), None);
        assert_eq!(at(&mut g, &mut t, Pointer::Move(340.0, 10.0), 600.0), Some(Touch::Dragging(0, 2)));
        assert_eq!(g.dragging(), Some((0, 2)));
        assert_eq!(at(&mut g, &mut t, Pointer::Up(340.0, 10.0), 600.0), Some(Touch::Moved(0, 2)));
        assert_eq!(t.tabs.iter().map(|x| x.title.as_str()).collect::<Vec<_>>(), ["Tab 1", "Tab 2", "Tab 0", "Tab 3"]);
        assert_eq!(t.chosen, 2);
        assert_eq!(g.dragging(), None);
    }

    #[test]
    fn a_key_shifts_the_chosen_tab_one_place() {
        let mut t = tabs(3);
        assert_eq!(t.shift(-1, 600.0), None);
        assert_eq!(t.shift(1, 600.0), Some(Touch::Moved(0, 1)));
        assert_eq!((t.chosen, t.tabs[1].title.as_str()), (1, "Tab 0"));
        assert_eq!(t.shift(1, 600.0), Some(Touch::Moved(1, 2)));
        assert_eq!(t.shift(1, 600.0), None);
    }

    #[test]
    fn a_tab_let_go_where_it_was_does_not_move() {
        let (mut t, mut g) = (tabs(3), Gesture::new());
        at(&mut g, &mut t, Pointer::Down(250.0, 10.0), 600.0);
        at(&mut g, &mut t, Pointer::Move(300.0, 20.0), 600.0);
        assert_eq!(at(&mut g, &mut t, Pointer::Up(300.0, 20.0), 600.0), None);
    }

    #[test]
    fn reordering_keeps_the_chosen_tab_on_its_own() {
        let mut t = tabs(5);
        t.chosen = 3;
        t.reorder(0, 4);
        assert_eq!(t.tabs[t.chosen].title, "Tab 3");
        t.reorder(4, 0);
        assert_eq!(t.tabs[t.chosen].title, "Tab 3");
        t.reorder(3, 1);
        assert_eq!(t.tabs[t.chosen].title, "Tab 3");
    }

    #[test]
    fn closing_picks_the_neighbour_and_the_last_tab_leaves_none() {
        let mut t = tabs(3);
        t.chosen = 1;
        assert_eq!(t.close(1), Some(1));
        assert_eq!(t.tabs[1].title, "Tab 2");
        assert_eq!(t.close(1), Some(0));
        t.chosen = 0;
        t.open(Tab::new("New"), 600.0);
        assert_eq!(t.chosen, 1);
        t.close(0);
        assert_eq!(t.tabs[t.chosen].title, "New");
        assert_eq!(t.close(0), None);
    }

    #[test]
    fn the_more_button_asks_for_a_menu() {
        let (mut t, mut g) = (tabs(10), Gesture::new());
        let more = t.layout(500.0).more.unwrap();
        at(&mut g, &mut t, Pointer::Down(more.x + 5.0, 10.0), 500.0);
        assert_eq!(at(&mut g, &mut t, Pointer::Up(more.x + 5.0, 10.0), 500.0), Some(Touch::More));
    }

    #[test]
    fn the_keys_step_along_without_wrapping() {
        let mut t = tabs(3);
        assert_eq!(t.step(-1, 600.0), Some(0));
        assert_eq!(t.step(1, 600.0), Some(1));
        assert_eq!(t.step(5, 600.0), Some(2));
    }

    #[test]
    fn the_wheel_moves_the_tabs_in_reach() {
        let mut t = tabs(10);
        t.scroll(3, 500.0);
        assert_eq!(t.first, 3);
        t.scroll(100, 500.0);
        assert_eq!(t.first, 6);
        t.scroll(-100, 500.0);
        assert_eq!(t.first, 0);
    }

    #[test]
    fn a_reader_hears_each_tab_with_its_count() {
        let mut t = tabs(3);
        t.tabs[1] = Tab::new("Inbox").count(4);
        t.chosen = 1;
        let said = t.said(600.0);
        assert_eq!(said.total, 3);
        assert_eq!(said.tabs[1].words, "Inbox, 4");
        assert!(said.tabs[1].chosen && !said.tabs[0].chosen);
    }

    #[test]
    fn the_count_is_said_in_the_words_of_the_language() {
        let mut t = tabs(2);
        t.tabs[1] = Tab::new("Inbox").count(4);
        let catalogue = crate::lang::Catalogue::new(
            crate::lang::Bundle::from_text("xx", "kit-tab-count = { $count } in { $title }"),
            crate::lang::Bundle::from_files("en", crate::lang::KIT_ENGLISH),
        );
        let words = crate::lang::with_catalogue(catalogue, || t.said(600.0).tabs[1].words.clone());
        assert_eq!(words, "4 in Inbox");
    }
}
