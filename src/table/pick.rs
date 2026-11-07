//! Several rows chosen at once, and the band dragged across them to choose.

use tiny_skia::PixmapMut;

use super::super::draw::{edge, fill_on};
use super::super::paint::Painter;
use super::super::theme;
use super::super::widget::{Pen, Rect};
use super::{Table, Touch, SLOP};

impl Table {
    pub fn picked(&self) -> &[usize] {
        &self.picked
    }

    pub fn set_picked(&mut self, rows: impl IntoIterator<Item = usize>) {
        let mut rows: Vec<usize> = rows.into_iter().filter(|&r| r < self.rows).collect();
        rows.sort_unstable();
        rows.dedup();
        self.picked = rows;
    }

    pub fn is_chosen(&self, row: usize) -> bool {
        if self.multiple { self.picked.binary_search(&row).is_ok() } else { self.chosen == Some(row) }
    }

    pub fn pick_all(&mut self) {
        self.picked = (0..self.rows).collect();
    }

    pub(super) fn pick_one(&mut self, row: usize) {
        self.picked = vec![row];
        self.anchor = Some(row);
        self.chosen = Some(row);
    }

    pub(super) fn pick_to(&mut self, row: usize) {
        let from = self.anchor.or(self.chosen).unwrap_or(row);
        self.anchor = Some(from);
        self.picked = (from.min(row)..=from.max(row)).collect();
        self.chosen = Some(row);
    }

    pub(super) fn pick_toggle(&mut self, row: usize) {
        match self.picked.binary_search(&row) {
            Ok(at) => {
                self.picked.remove(at);
            }
            Err(at) => self.picked.insert(at, row),
        }
        self.anchor = Some(row);
        self.chosen = Some(row);
    }

    fn line_at(&self, y: f32) -> f32 {
        self.first + (y - self.head_height) / self.row_h()
    }

    pub(super) fn below_rows(&self, x: f32, y: f32, tall: f32) -> bool {
        y >= self.head_height && y < tall && (0.0..self.width()).contains(&x) && self.line_at(y).floor() >= self.lines() as f32
    }
}

#[derive(Clone, Debug)]
pub(super) struct Banding {
    from: (f32, f32),
    line: f32,
    to: (f32, f32),
    moved: bool,
    kept: Vec<usize>,
    row: Option<usize>,
}

impl Banding {
    pub(super) fn new(table: &Table, (x, y): (f32, f32), row: Option<usize>, keep: bool) -> Banding {
        Banding { from: (x, y), line: table.line_at(y), to: (x, y), moved: false, kept: if keep { table.picked.clone() } else { Vec::new() }, row }
    }

    pub(super) fn drag(&mut self, table: &mut Table, x: f32, y: f32) -> Option<Touch> {
        self.to = (x, y);
        if !self.moved && (x - self.from.0).abs() <= SLOP && (y - self.from.1).abs() <= SLOP {
            return None;
        }
        self.moved = true;
        let lines = table.lines();
        let now = table.line_at(y);
        let (top, foot) = (self.line.min(now).max(0.0), self.line.max(now));
        let first = (top.floor() as usize).min(lines);
        let last = (foot.ceil().max(top.floor() + 1.0) as usize).min(lines);
        let rows = table.rows_before(first)..table.rows_before(last);
        let low = self.kept.partition_point(|&r| r < rows.start);
        let high = self.kept.partition_point(|&r| r < rows.end);
        let mut picked = self.kept[..low].to_vec();
        picked.extend(rows);
        picked.extend_from_slice(&self.kept[high..]);
        if picked == table.picked {
            return None;
        }
        table.picked = picked;
        Some(Touch::Banded)
    }

    pub(super) fn let_go(self, table: &mut Table) -> Option<Touch> {
        if self.moved {
            return None;
        }
        let row = self.row?;
        table.pick_toggle(row);
        Some(Touch::Toggled(row))
    }

    pub(super) fn rect(&self, table: &Table, size: (f32, f32)) -> Option<Rect> {
        if !self.moved {
            return None;
        }
        let start = table.head_height + (self.line - table.first) * table.row_h();
        let (x0, x1) = (self.from.0.min(self.to.0).max(0.0), self.from.0.max(self.to.0).min(size.0));
        let (y0, y1) = (start.min(self.to.1).max(table.head_height), start.max(self.to.1).min(size.1));
        (x1 > x0 && y1 > y0).then_some(Rect { x: x0, y: y0, w: x1 - x0, h: y1 - y0 })
    }
}

pub fn paint_band(canvas: &mut PixmapMut, pen: &Pen, band: Option<Rect>) {
    let mut p = Painter::default();
    paint_band_on(&mut p, pen, band);
    p.play(canvas);
}

pub fn paint_band_on(p: &mut Painter, pen: &Pen, band: Option<Rect>) {
    let Some(area) = band else { return };
    fill_on(p, pen, area, theme::alpha(pen.theme.accent, theme::sheet().selection));
    edge(p, pen, area, 1.0, pen.theme.accent);
}
