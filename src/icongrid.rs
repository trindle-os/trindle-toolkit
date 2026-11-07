//! A grid of icons a program paints itself, as a folder shows files: named tiles, several chosen at once, moved through by keys.

use std::collections::BTreeSet;

use tiny_skia::PixmapMut;

use super::draw::{around, edge, fill_on, words_on};
use super::paint::Painter;
use super::input::{Input, Keysym, Mods};
use super::theme;
use super::widget::{Pen, Pointer, Rect, SaidCells, SaidTable, Words};

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Selection {
    pub chosen: BTreeSet<usize>,
    pub focus: Option<usize>,
    anchor: Option<usize>,
}

impl Selection {
    pub fn press(&mut self, i: usize, mods: Mods) {
        if mods.shift {
            self.run_to(i, mods.ctrl);
        } else if mods.ctrl {
            self.toggle(i);
        } else {
            self.only(i);
        }
        self.focus = Some(i);
    }

    pub fn is_chosen(&self, i: usize) -> bool {
        self.chosen.contains(&i)
    }

    pub fn all(&mut self, count: usize) {
        self.chosen = (0..count).collect();
    }

    pub fn clear(&mut self) {
        self.chosen.clear();
        self.anchor = None;
    }

    pub fn band(&mut self, kept: &BTreeSet<usize>, items: impl IntoIterator<Item = usize>, mods: Mods) {
        self.chosen = if mods.ctrl || mods.shift { kept.clone() } else { BTreeSet::new() };
        self.chosen.extend(items);
    }

    pub fn key(&mut self, input: &Input, columns: usize, count: usize) -> bool {
        let Input::Key { sym, mods, .. } = input else { return false };
        if count == 0 || mods.alt {
            return false;
        }
        let (at, cols) = (self.focus.unwrap_or(0).min(count - 1), columns.max(1));
        let to = match *sym {
            Keysym::Left => at.checked_sub(1),
            Keysym::Right => Some(at + 1),
            Keysym::Up => at.checked_sub(cols),
            Keysym::Down => Some(at + cols),
            Keysym::Home => Some(0),
            Keysym::End => Some(count - 1),
            Keysym::a | Keysym::A if mods.ctrl => {
                self.all(count);
                return true;
            }
            Keysym::space => {
                if mods.ctrl { self.toggle(at) } else { self.only(at) }
                self.focus = Some(at);
                return true;
            }
            _ => return false,
        };
        let first = self.focus.is_none() && !matches!(*sym, Keysym::Home | Keysym::End);
        let to = if first { 0 } else { to.filter(|&t| t < count).unwrap_or(at) };
        if mods.shift {
            self.run_to(to, mods.ctrl);
        } else if !mods.ctrl {
            self.only(to);
        }
        self.focus = Some(to);
        true
    }

    fn only(&mut self, i: usize) {
        self.chosen = BTreeSet::from([i]);
        self.anchor = Some(i);
    }

    fn toggle(&mut self, i: usize) {
        if !self.chosen.remove(&i) {
            self.chosen.insert(i);
        }
        self.anchor = Some(i);
    }

    fn run_to(&mut self, i: usize, keep: bool) {
        let from = self.anchor.unwrap_or(i);
        if !keep {
            self.chosen.clear();
        }
        self.chosen.extend(from.min(i)..=from.max(i));
        self.anchor = Some(from);
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct IconGrid {
    pub count: usize,
    pub selection: Selection,
    pub top: f32,
    band: Option<Band>,
}

#[derive(Clone, Debug, PartialEq)]
struct Band {
    from: (f32, f32),
    to: (f32, f32),
    kept: BTreeSet<usize>,
    mods: Mods,
}

impl IconGrid {
    pub fn new(count: usize) -> Self {
        IconGrid { count, ..Self::default() }
    }

    pub fn columns(&self, wide: f32) -> usize {
        let (tile, gap) = step();
        (((wide + gap) / (tile + gap)).floor() as usize).max(1)
    }

    pub fn tile_rect(&self, i: usize, wide: f32) -> Rect {
        let (tile, gap) = step();
        let cols = self.columns(wide);
        Rect { x: (i % cols) as f32 * (tile + gap), y: (i / cols) as f32 * (tile + gap) - self.top, w: tile, h: tile }
    }

    pub fn height(&self, wide: f32) -> f32 {
        let (tile, gap) = step();
        let rows = self.count.div_ceil(self.columns(wide));
        (rows as f32 * (tile + gap) - gap).max(0.0)
    }

    pub fn at(&self, x: f32, y: f32, wide: f32) -> Option<usize> {
        let (tile, gap) = step();
        let (col, row) = ((x / (tile + gap)).floor(), ((y + self.top) / (tile + gap)).floor());
        if x < 0.0 || col as usize >= self.columns(wide) || row < 0.0 {
            return None;
        }
        let i = row as usize * self.columns(wide) + col as usize;
        (i < self.count && self.tile_rect(i, wide).contains(x, y)).then_some(i)
    }

    pub fn scroll(&mut self, by: f32, size: (f32, f32)) {
        self.top = (self.top + by).clamp(0.0, (self.height(size.0) - size.1).max(0.0));
    }

    pub fn reveal(&mut self, i: usize, size: (f32, f32)) {
        let r = self.tile_rect(i, size.0);
        if r.y < 0.0 {
            self.scroll(r.y, size);
        } else if r.y + r.h > size.1 {
            self.scroll(r.y + r.h - size.1, size);
        }
    }

    pub fn pointer(&mut self, event: Pointer, mods: Mods, size: (f32, f32)) -> bool {
        let was = (self.selection.clone(), self.band.clone());
        match event {
            Pointer::Down(x, y) => match self.at(x, y, size.0) {
                Some(i) => self.selection.press(i, mods),
                None => {
                    if !mods.ctrl && !mods.shift {
                        self.selection.clear();
                    }
                    let at = (x, y + self.top);
                    self.band = Some(Band { from: at, to: at, kept: self.selection.chosen.clone(), mods });
                }
            },
            Pointer::Move(x, y) => {
                let Some(band) = &mut self.band else { return false };
                band.to = (x, y + self.top);
                let (kept, mods) = (band.kept.clone(), band.mods);
                let r = self.band_rect().unwrap_or_default();
                let hit: Vec<usize> = (0..self.count).filter(|&i| touches(&self.tile_rect(i, size.0), &r)).collect();
                self.selection.band(&kept, hit, mods);
            }
            Pointer::Up(..) | Pointer::Leave => self.band = None,
            _ => {}
        }
        (self.selection.clone(), self.band.clone()) != was
    }

    pub fn band_rect(&self) -> Option<Rect> {
        let b = self.band.as_ref()?;
        let (x, y) = (b.from.0.min(b.to.0), b.from.1.min(b.to.1));
        Some(Rect { x, y: y - self.top, w: (b.from.0 - b.to.0).abs(), h: (b.from.1 - b.to.1).abs() })
    }

    pub fn key(&mut self, input: &Input, size: (f32, f32)) -> bool {
        let used = self.selection.key(input, self.columns(size.0), self.count);
        if let Some(i) = self.selection.focus.filter(|_| used) {
            self.reveal(i, size);
        }
        used
    }

    pub fn said(&self, size: (f32, f32), name: impl Fn(usize) -> String) -> SaidTable {
        let rows = (0..self.count).map(|i| (i, self.tile_rect(i, size.0)))
            .filter(|(_, r)| r.y + r.h > 0.0 && r.y < size.1)
            .map(|(i, r)| SaidCells { index: i, cells: vec![name(i)], top: r.y, height: r.h, chosen: self.selection.is_chosen(i), group: None, branch: None })
            .collect();
        SaidTable { headings: vec![crate::t!("kit-name").to_string()], columns: vec![(0.0, size.0)], sort: None, total: self.count, head_height: 0.0, rows, column: None, many: false, tree: false, cursor: None }
    }
}

fn touches(a: &Rect, b: &Rect) -> bool {
    a.x < b.x + b.w && b.x < a.x + a.w && a.y < b.y + b.h && b.y < a.y + a.h
}

fn step() -> (f32, f32) {
    let k = theme::sheet();
    (k.icon_tile, k.gap)
}

pub fn paint_icons(canvas: &mut PixmapMut, pen: &Pen, grid: &IconGrid, hover: Option<usize>, keys: bool, tile: impl Fn(usize) -> (String, String)) {
    let mut p = Painter::default();
    paint_icons_on(&mut p, pen, (canvas.width(), canvas.height()), grid, hover, keys, tile);
    p.play(canvas);
}

pub fn paint_icons_on<'a>(p: &mut Painter<'a>, pen: &Pen<'a>, size: (u32, u32), grid: &IconGrid, hover: Option<usize>, keys: bool, tile: impl Fn(usize) -> (String, String)) {
    let (k, t, s) = (pen.scale, pen.theme, theme::sheet());
    let size = (size.0 as f32 / k, size.1 as f32 / k);
    let style = Words::Plain.style(k);
    let dots = pen.fonts.layout("\u{2026}", &style, None);
    for i in 0..grid.count {
        let r = grid.tile_rect(i, size.0);
        if r.y + r.h <= 0.0 || r.y >= size.1 {
            continue;
        }
        let chosen = grid.selection.is_chosen(i);
        if chosen {
            fill_on(p, pen, r, t.picked);
        } else if hover == Some(i) {
            fill_on(p, pen, r, t.hover);
        }
        let (icon, words) = tile(i);
        let px = s.icon_tile_icon * k;
        let (ix, iy) = (r.x * k + (r.w * k - px) / 2.0, r.y * k + s.gap * k);
        let ink = if chosen { t.on_picked } else { t.fg };
        let (icons, at) = (pen.icons, (ix.round() as i32, iy.round() as i32, px.round() as u32));
        p.pixels(around(at.0 as f32, at.1 as f32, at.2 as f32, at.2 as f32, 0.0), move |c| {
            icons.draw(c, &icon, at.0, at.1, at.2, 1, ink);
        });
        let room = (r.w - 2.0 * s.gap) * k;
        let full = pen.fonts.layout(&words, &style, None);
        let line = if full.width > room { full.cut(room, &dots) } else { full };
        let x = r.x * k + (r.w * k - line.width) / 2.0;
        words_on(p, pen.fonts, line, x, iy + px + s.gap * k, ink);
        if keys && grid.selection.focus == Some(i) {
            edge(p, pen, r, s.focus_width, t.accent);
        }
    }
    if let Some(band) = grid.band_rect() {
        fill_on(p, pen, band, theme::alpha(t.accent, s.selection));
        edge(p, pen, band, 1.0, t.accent);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn keyed(sym: Keysym, mods: Mods) -> Input {
        Input::Key { sym, text: None, mods, repeat: false }
    }

    const CTRL: Mods = Mods { ctrl: true, shift: false, alt: false, logo: false };
    const SHIFT: Mods = Mods { ctrl: false, shift: true, alt: false, logo: false };

    fn chosen(s: &Selection) -> Vec<usize> {
        s.chosen.iter().copied().collect()
    }

    #[test]
    fn a_press_chooses_one_ctrl_adds_or_takes_away_and_shift_chooses_the_run() {
        let mut s = Selection::default();
        s.press(2, Mods::default());
        s.press(5, CTRL);
        assert_eq!(chosen(&s), [2, 5]);
        s.press(2, CTRL);
        assert_eq!(chosen(&s), [5]);
        s.press(8, SHIFT);
        assert_eq!((chosen(&s), s.focus), (vec![2, 3, 4, 5, 6, 7, 8], Some(8)));
        s.press(0, Mods::default());
        assert_eq!(chosen(&s), [0]);
    }

    #[test]
    fn arrows_move_by_one_across_and_by_a_row_down_and_stop_at_the_ends() {
        let mut s = Selection::default();
        assert!(s.key(&keyed(Keysym::Right, Mods::default()), 4, 10));
        assert_eq!(s.focus, Some(0), "the first key lands on the first item");
        s.key(&keyed(Keysym::Down, Mods::default()), 4, 10);
        s.key(&keyed(Keysym::Right, Mods::default()), 4, 10);
        assert_eq!((s.focus, chosen(&s)), (Some(5), vec![5]));
        s.key(&keyed(Keysym::Down, Mods::default()), 4, 10);
        assert_eq!(s.focus, Some(9));
        s.key(&keyed(Keysym::Down, Mods::default()), 4, 10);
        assert_eq!(s.focus, Some(9), "no row below");
        s.key(&keyed(Keysym::Up, SHIFT), 4, 10);
        assert_eq!(chosen(&s), [5, 6, 7, 8, 9]);
        s.key(&keyed(Keysym::Home, CTRL), 4, 10);
        assert_eq!((s.focus, chosen(&s).len()), (Some(0), 5), "Ctrl only moves");
        s.key(&keyed(Keysym::space, CTRL), 4, 10);
        assert!(s.is_chosen(0));
        assert!(s.key(&keyed(Keysym::a, CTRL), 4, 10));
        assert_eq!(chosen(&s).len(), 10);
        assert!(!s.key(&keyed(Keysym::x, Mods::default()), 4, 10));
        assert!(!s.key(&keyed(Keysym::Down, Mods::default()), 4, 0));
    }

    #[test]
    fn tiles_lie_in_rows_and_are_found_by_point_and_scrolled_into_sight() {
        let (tile, gap) = step();
        let mut g = IconGrid::new(10);
        let wide = 3.0 * tile + 2.0 * gap + 1.0;
        assert_eq!(g.columns(wide), 3);
        assert_eq!(g.tile_rect(4, wide), Rect { x: tile + gap, y: tile + gap, w: tile, h: tile });
        assert_eq!(g.at(tile + gap + 1.0, tile + gap + 1.0, wide), Some(4));
        assert_eq!(g.at(tile + 1.0, 1.0, wide), None, "in the gap");
        assert_eq!(g.at(1.0, 4.0 * (tile + gap), wide), None, "past the last row");
        let size = (wide, tile * 2.0);
        assert!(g.key(&keyed(Keysym::End, Mods::default()), size));
        assert_eq!(g.top, g.height(wide) - size.1);
        assert!(g.pointer(Pointer::Down(tile + 1.0, 1.0), Mods::default(), size));
        assert!(g.selection.chosen.is_empty(), "a press between tiles lets all go");
    }

    #[test]
    fn a_band_dragged_from_between_tiles_chooses_all_it_touches() {
        let (tile, gap) = step();
        let wide = 4.0 * (tile + gap);
        let size = (wide, 1000.0);
        let mut g = IconGrid::new(12);
        g.selection.press(11, Mods::default());
        assert!(g.pointer(Pointer::Down(tile + gap / 2.0, 2.0), Mods::default(), size));
        assert!(g.selection.chosen.is_empty());
        g.pointer(Pointer::Move(2.0 * (tile + gap) + 4.0, tile + gap + 4.0), Mods::default(), size);
        assert_eq!(chosen(&g.selection), [1, 2, 5, 6]);
        assert!(g.band_rect().is_some());
        g.pointer(Pointer::Move(2.0 * (tile + gap) + 4.0, 4.0), Mods::default(), size);
        assert_eq!(chosen(&g.selection), [1, 2]);
        g.pointer(Pointer::Up(0.0, 0.0), Mods::default(), size);
        assert_eq!((g.band_rect(), chosen(&g.selection)), (None, vec![1, 2]));
        g.pointer(Pointer::Down(tile + gap / 2.0, 2.0 * (tile + gap) + 2.0), CTRL, size);
        g.pointer(Pointer::Move(tile + gap + 4.0, 2.0 * (tile + gap) + 8.0), CTRL, size);
        assert_eq!(chosen(&g.selection), [1, 2, 9]);
    }
}
