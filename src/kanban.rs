//! A board of cards in columns a program paints itself; cards move by drag, within a column and across, or by keys.

use tiny_skia::PixmapMut;

use super::draw::{fill_on, text_in_on};
use super::paint::Painter;
use super::input::Cursor;
use super::theme::{corner_px, Part};
use super::ui::fill_on as rounded_fill_on;
use super::widget::{Pen, Pointer, Rect, SaidCells, SaidTable, Words};

fn pad() -> f32 {
    crate::theme::gaps().medium
}
fn gap() -> f32 {
    crate::theme::gaps().large
}
const SLOP: f32 = 6.0;

#[derive(Clone, Debug, PartialEq)]
pub struct Card {
    pub title: String,
    pub id: usize,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Lane {
    pub title: String,
    pub cards: Vec<Card>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Board {
    pub lanes: Vec<Lane>,
    pub chosen: Option<(usize, usize)>,
    pub lane_width: f32,
    pub card_height: f32,
    pub head_height: f32,
    pub down: f32,
}

impl Board {
    pub fn new(lanes: Vec<Lane>) -> Board {
        let (lane_width, card_height, head_height) = crate::theme::sheet().lane;
        Board { lanes, chosen: None, lane_width, card_height, head_height, down: 0.0 }
    }

    pub fn lane_rect(&self, lane: usize, tall: f32) -> Rect {
        Rect { x: lane as f32 * (self.lane_width + gap()), y: 0.0, w: self.lane_width, h: tall }
    }

    pub fn card_rect(&self, lane: usize, place: usize) -> Rect {
        let step = self.card_height + pad();
        Rect { x: lane as f32 * (self.lane_width + gap()) + pad(), y: self.head_height + place as f32 * step - self.down, w: self.lane_width - 2.0 * pad(), h: self.card_height }
    }

    fn lane_at(&self, x: f32) -> usize {
        ((x / (self.lane_width + gap())).floor().max(0.0) as usize).min(self.lanes.len().saturating_sub(1))
    }

    pub fn card_at(&self, x: f32, y: f32) -> Option<(usize, usize)> {
        if y < self.head_height {
            return None;
        }
        let lane = self.lane_at(x);
        (0..self.lanes.get(lane)?.cards.len()).find(|&c| self.card_rect(lane, c).contains(x, y)).map(|c| (lane, c))
    }

    pub fn place(&self, x: f32, y: f32, from: (usize, usize)) -> (usize, usize) {
        let lane = self.lane_at(x);
        let cards = self.lanes.get(lane).map_or(0, |l| l.cards.len());
        let above = (0..cards).filter(|&c| (lane, c) != from && { let r = self.card_rect(lane, c); r.y + r.h / 2.0 < y }).count();
        (lane, above)
    }

    pub fn move_card(&mut self, from: (usize, usize), to: (usize, usize)) -> bool {
        if to.0 >= self.lanes.len() || from.1 >= self.lanes.get(from.0).map_or(0, |l| l.cards.len()) {
            return false;
        }
        let card = self.lanes[from.0].cards.remove(from.1);
        let at = to.1.min(self.lanes[to.0].cards.len());
        self.lanes[to.0].cards.insert(at, card);
        if self.chosen == Some(from) {
            self.chosen = Some((to.0, at));
        }
        true
    }

    pub fn step(&mut self, across: isize, by: isize, tall: f32) -> Option<(usize, usize)> {
        self.drop_stale_choice();
        let (lane, place) = match self.chosen {
            Some((l, c)) => {
                let l = self.near_lane(l, across)?;
                let len = self.lanes[l].cards.len() as isize;
                (l, (c as isize + by).clamp(0, len - 1) as usize)
            }
            None => (self.lanes.iter().position(|l| !l.cards.is_empty())?, 0),
        };
        self.chosen = Some((lane, place));
        self.reveal(lane, place, tall);
        self.chosen
    }

    fn drop_stale_choice(&mut self) {
        if self.chosen.is_some_and(|(l, c)| self.lanes.get(l).is_none_or(|l| c >= l.cards.len())) {
            self.chosen = None;
        }
    }

    fn near_lane(&self, lane: usize, across: isize) -> Option<usize> {
        if across == 0 {
            return Some(lane);
        }
        let mut l = lane as isize;
        loop {
            l += across.signum();
            match self.lanes.get(usize::try_from(l).ok()?) {
                Some(n) if !n.cards.is_empty() => return Some(l as usize),
                Some(_) => continue,
                None => return Some(lane),
            }
        }
    }

    pub fn shift(&mut self, across: isize, by: isize, tall: f32) -> Option<Touch> {
        self.drop_stale_choice();
        let from = self.chosen?;
        let lane = (from.0 as isize + across).clamp(0, self.lanes.len() as isize - 1) as usize;
        let to = if lane == from.0 {
            (lane, (from.1 as isize + by).clamp(0, self.lanes[lane].cards.len() as isize - 1) as usize)
        } else {
            (lane, from.1.min(self.lanes[lane].cards.len()))
        };
        if to == from || !self.move_card(from, to) {
            return None;
        }
        self.reveal(to.0, to.1, tall);
        Some(Touch::Moved(from, to))
    }

    pub fn reveal(&mut self, lane: usize, place: usize, tall: f32) {
        let r = self.card_rect(lane, place);
        if r.y < self.head_height {
            self.down -= self.head_height - r.y;
        } else if r.y + r.h + pad() > tall {
            self.down += r.y + r.h + pad() - tall;
        }
        self.down = self.down.max(0.0);
    }

    pub fn scroll(&mut self, lines: f32, tall: f32) {
        let most = self.lanes.iter().map(|l| l.cards.len()).max().unwrap_or(0) as f32 * (self.card_height + pad()) + self.head_height + pad() - tall;
        self.down = (self.down + lines * 24.0).clamp(0.0, most.max(0.0));
    }

    pub fn said(&self) -> SaidTable {
        let most = self.lanes.iter().map(|l| l.cards.len()).max().unwrap_or(0);
        SaidTable {
            headings: self.lanes.iter().map(|l| l.title.clone()).collect(),
            columns: (0..self.lanes.len()).map(|l| { let r = self.lane_rect(l, 0.0); (r.x, r.w) }).collect(),
            sort: None,
            total: most,
            head_height: self.head_height,
            rows: (0..most).map(|c| SaidCells {
                index: c,
                cells: self.lanes.iter().map(|l| l.cards.get(c).map_or_else(String::new, |k| k.title.clone())).collect(),
                top: self.card_rect(0, c).y,
                height: self.card_height,
                chosen: self.chosen.is_some_and(|(_, at)| at == c),
                group: None,
                branch: None,
            }).collect(),
            column: self.chosen.map(|(lane, _)| lane),
            many: false,
            tree: false,
            cursor: None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Touch {
    Chose((usize, usize)),
    Dragging((usize, usize), (usize, usize)),
    Moved((usize, usize), (usize, usize)),
}

#[derive(Clone, Debug, Default)]
pub struct Gesture {
    down: Option<((usize, usize), (f32, f32), Option<(usize, usize)>)>,
    at: Option<(f32, f32)>,
}

impl Gesture {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn pointer(&mut self, board: &mut Board, event: Pointer) -> Option<Touch> {
        match event {
            Pointer::Down(x, y) => {
                self.at = Some((x, y));
                let card = board.card_at(x, y)?;
                board.chosen = Some(card);
                self.down = Some((card, (x, y), None));
                Some(Touch::Chose(card))
            }
            Pointer::Move(x, y) => {
                self.at = Some((x, y));
                let (from, at, dragging) = self.down.as_mut()?;
                if dragging.is_none() && (x - at.0).hypot(y - at.1) < SLOP {
                    return None;
                }
                let to = board.place(x, y, *from);
                *dragging = Some(to);
                Some(Touch::Dragging(*from, to))
            }
            Pointer::Up(x, y) => {
                self.at = Some((x, y));
                let (from, _, Some(_)) = self.down.take()? else { return None };
                let to = board.place(x, y, from);
                (to != from && board.move_card(from, to)).then_some(Touch::Moved(from, to))
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

    pub fn dragging(&self) -> Option<((usize, usize), (usize, usize), (f32, f32))> {
        let ((from, start, Some(to)), Some(now)) = (self.down?, self.at) else { return None };
        Some((from, to, (now.0 - start.0, now.1 - start.1)))
    }

    pub fn hovering(&self, board: &Board) -> Option<(usize, usize)> {
        let (x, y) = self.at?;
        board.card_at(x, y)
    }

    pub fn cursor(&self, board: &Board, x: f32, y: f32) -> Cursor {
        if self.down.is_some_and(|d| d.2.is_some()) {
            Cursor::Move
        } else if board.card_at(x, y).is_some() {
            Cursor::Hand
        } else {
            Cursor::Arrow
        }
    }
}

pub fn paint_board(canvas: &mut PixmapMut, pen: &Pen, board: &Board, hover: Option<(usize, usize)>, dragging: Option<((usize, usize), (usize, usize), (f32, f32))>) {
    let mut p = Painter::default();
    paint_board_on(&mut p, pen, canvas.height(), board, hover, dragging);
    p.play(canvas);
}

pub fn paint_board_on<'a>(p: &mut Painter<'a>, pen: &Pen<'a>, tall: u32, board: &Board, hover: Option<(usize, usize)>, dragging: Option<((usize, usize), (usize, usize), (f32, f32))>) {
    let (t, k) = (pen.theme, pen.scale);
    let tall = tall as f32 / k;
    let (plain, head) = (Words::Plain.style(k), Words::Heading.style(k));
    let px = |r: Rect| Rect { x: r.x * k, y: r.y * k, w: r.w * k, h: r.h * k };
    let card = |p: &mut Painter<'a>, r: Rect, words: &str, back, ink| {
        rounded_fill_on(p, px(r), corner_px(Part::Box, r.h * k, k), back);
        text_in_on(p, pen, words, &plain, Rect { x: r.x + pad(), w: (r.w - 2.0 * pad()).max(0.0), ..r }, ink, false);
    };
    for (l, lane) in board.lanes.iter().enumerate() {
        let r = board.lane_rect(l, tall);
        rounded_fill_on(p, px(r), corner_px(Part::Card, r.w.min(r.h) * k, k), t.panel);
        text_in_on(p, pen, &lane.title, &head, Rect { x: r.x + pad(), y: 0.0, w: r.w - 2.0 * pad(), h: board.head_height }, t.fg, false);
        for (c, item) in lane.cards.iter().enumerate() {
            let r = board.card_rect(l, c);
            if r.y < board.head_height || r.y > tall || dragging.is_some_and(|d| d.0 == (l, c)) {
                continue;
            }
            let (back, ink) = if board.chosen == Some((l, c)) {
                (t.picked, t.on_picked)
            } else if hover == Some((l, c)) {
                (t.hover, t.fg)
            } else {
                (t.raised, t.fg)
            };
            card(p, r, &item.title, back, ink);
        }
    }
    let Some((from, to, (dx, dy))) = dragging else { return };
    let Some(item) = board.lanes.get(from.0).and_then(|l| l.cards.get(from.1)) else { return };
    let shown = if to.0 == from.0 && to.1 >= from.1 { to.1 + 1 } else { to.1 };
    let r = board.card_rect(to.0, shown);
    fill_on(p, pen, Rect { y: r.y - pad() / 2.0 - 1.0, h: 2.0, ..r }, t.accent);
    let r = board.card_rect(from.0, from.1);
    card(p, Rect { x: r.x + dx, y: r.y + dy, ..r }, &item.title, t.raised_more, t.fg);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn board() -> Board {
        let lane = |title: &str, n: usize, base: usize| Lane { title: title.into(), cards: (0..n).map(|i| Card { title: format!("{title} {i}"), id: base + i }).collect() };
        Board::new(vec![lane("To do", 3, 0), lane("Doing", 1, 10), lane("Done", 0, 20)])
    }

    fn ids(b: &Board, lane: usize) -> Vec<usize> {
        b.lanes[lane].cards.iter().map(|c| c.id).collect()
    }

    fn mid(b: &Board, lane: usize, place: usize) -> (f32, f32) {
        let r = b.card_rect(lane, place);
        (r.x + r.w / 2.0, r.y + r.h / 2.0)
    }

    #[test]
    fn cards_lie_in_columns_apart_and_are_found_by_point() {
        let b = board();
        let (a, c) = (b.card_rect(0, 0), b.card_rect(0, 1));
        assert!(c.y >= a.y + a.h + pad(), "cards never touch");
        assert!(b.card_rect(1, 0).x >= a.x + a.w + gap(), "columns never touch");
        let (x, y) = mid(&b, 1, 0);
        assert_eq!(b.card_at(x, y), Some((1, 0)));
        assert_eq!(b.card_at(x, 10.0), None, "the column's name is no card");
        assert_eq!(b.card_at(mid(&b, 2, 0).0, 100.0), None, "an empty column");
    }

    #[test]
    fn a_card_dragged_across_columns_moves_there() {
        let (mut b, mut g) = (board(), Gesture::new());
        let (x, y) = mid(&b, 0, 1);
        assert_eq!(g.pointer(&mut b, Pointer::Down(x, y)), Some(Touch::Chose((0, 1))));
        assert_eq!(g.pointer(&mut b, Pointer::Move(x + 2.0, y)), None, "a slip is not a drag");
        let (tx, _) = mid(&b, 2, 0);
        assert_eq!(g.pointer(&mut b, Pointer::Move(tx, 60.0)), Some(Touch::Dragging((0, 1), (2, 0))));
        assert!(g.dragging().is_some());
        assert_eq!(g.cursor(&b, tx, 60.0), Cursor::Move);
        assert_eq!(g.pointer(&mut b, Pointer::Up(tx, 60.0)), Some(Touch::Moved((0, 1), (2, 0))));
        assert_eq!((ids(&b, 0), ids(&b, 2)), (vec![0, 2], vec![1]));
        assert_eq!(b.chosen, Some((2, 0)), "the chosen card goes with it");
        assert_eq!(g.dragging(), None);
    }

    #[test]
    fn a_card_dragged_down_its_own_column_takes_the_place_below() {
        let (mut b, mut g) = (board(), Gesture::new());
        let (x, y) = mid(&b, 0, 0);
        g.pointer(&mut b, Pointer::Down(x, y));
        let below = b.card_rect(0, 2).y + b.card_height;
        g.pointer(&mut b, Pointer::Move(x, below));
        assert_eq!(g.pointer(&mut b, Pointer::Up(x, below)), Some(Touch::Moved((0, 0), (0, 2))));
        assert_eq!(ids(&b, 0), [1, 2, 0]);
    }

    #[test]
    fn let_go_where_it_was_nothing_moves() {
        let (mut b, mut g) = (board(), Gesture::new());
        let (x, y) = mid(&b, 0, 1);
        g.pointer(&mut b, Pointer::Down(x, y));
        g.pointer(&mut b, Pointer::Move(x, y + 10.0));
        assert_eq!(g.pointer(&mut b, Pointer::Up(x, y + 10.0)), None);
        assert_eq!(ids(&b, 0), [0, 1, 2]);
    }

    #[test]
    fn the_keys_choose_and_move_a_card_as_a_drag_would() {
        let mut b = board();
        assert_eq!(b.step(0, 1, 400.0), Some((0, 0)));
        assert_eq!(b.step(0, 1, 400.0), Some((0, 1)));
        assert_eq!(b.step(1, 0, 400.0), Some((1, 0)), "across to the nearest height");
        assert_eq!(b.step(1, 0, 400.0), Some((1, 0)), "an empty column is passed over, and the end stays");
        assert_eq!(b.shift(1, 0, 400.0), Some(Touch::Moved((1, 0), (2, 0))));
        assert_eq!(ids(&b, 2), [10]);
        b.chosen = Some((0, 0));
        assert_eq!(b.shift(0, 1, 400.0), Some(Touch::Moved((0, 0), (0, 1))));
        assert_eq!(ids(&b, 0), [1, 0, 2]);
        assert_eq!(b.shift(-1, 0, 400.0), None, "no column to the left");
    }

    #[test]
    fn a_choice_left_behind_by_a_removed_card_is_dropped_not_a_crash() {
        let mut b = board();
        b.chosen = Some((0, 99));
        assert_eq!(b.shift(1, 0, 400.0), None);
        assert_eq!(b.chosen, None);
        b.chosen = Some((9, 0));
        assert!(b.step(0, 1, 400.0).is_some());
        let mut empty = Board::new(vec![]);
        empty.chosen = Some((0, 0));
        assert_eq!((empty.step(1, 0, 400.0), empty.shift(0, 1, 400.0)), (None, None));
    }

    #[test]
    fn scrolling_reaches_the_longest_column_and_no_further() {
        let mut b = board();
        b.lanes[0].cards.extend((3..40).map(|i| Card { title: format!("{i}"), id: i }));
        b.scroll(1000.0, 400.0);
        let last = b.card_rect(0, 39);
        assert!(last.y + last.h <= 400.0 && last.y + last.h > 400.0 - b.card_height, "{last:?}");
        b.scroll(-1000.0, 400.0);
        assert_eq!(b.down, 0.0);
        b.reveal(0, 30, 400.0);
        assert!(b.card_rect(0, 30).y + b.card_height <= 400.0);
    }

    #[test]
    fn a_reader_hears_a_table_of_columns_and_cards() {
        let mut b = board();
        b.chosen = Some((1, 0));
        let said = b.said();
        assert_eq!(said.headings, ["To do", "Doing", "Done"]);
        assert_eq!(said.total, 3);
        assert_eq!(said.rows[0].cells, ["To do 0", "Doing 0", ""]);
        assert!(said.rows[0].chosen && !said.rows[1].chosen);
    }

    #[test]
    fn the_board_and_a_dragged_card_are_painted() {
        let theme = super::super::theme::Theme::new(false, "#fbc711");
        let (fonts, icons) = (super::super::text::Fonts::new(), super::super::icon::Icons::new());
        let pen = Pen { theme: &theme, fonts: &fonts, icons: &icons, scale: 1.0 };
        let mut b = board();
        b.chosen = Some((0, 0));
        let shot = |dragging| {
            let mut data = vec![0u8; 900 * 300 * 4];
            paint_board(&mut PixmapMut::from_bytes(&mut data, 900, 300).unwrap(), &pen, &b, None, dragging);
            data
        };
        let still = shot(None);
        assert!(still.chunks(4).any(|p| p[3] > 0));
        assert_ne!(still, shot(Some(((0, 1), (2, 0), (500.0, 0.0)))), "the drag shows");
    }
}
