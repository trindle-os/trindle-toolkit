//! A table a program paints itself: sortable, resizable headings that move by drag or keys, chosen rows (one, or several with a band), rows that open under a parent, group headings, cells typed in, and only the rows in sight painted.

mod cells;
mod keys;
mod pick;
mod tree;

pub use cells::{CellEdit, Edited};
pub use pick::{paint_band, paint_band_on};
pub use tree::Branch;

use std::ops::Range;
use std::time::Instant;

use tiny_skia::PixmapMut;

use super::colour::Colour;
use super::draw::{edge, fill_on, text_in_on};
use super::input::{Cursor, Mods};
use super::paint::Painter;
use super::text::Fonts;
use super::widget::{Pen, Pointer, Rect, SaidCells, SaidTable, Words};
use pick::Banding;

const PAD: f32 = 8.0;
const GRAB: f32 = 12.0;
const MOST_MEASURED: usize = 5000;
const BAR: f32 = 6.0;
const SLOP: f32 = 6.0;

#[derive(Clone, Debug, PartialEq)]
pub struct Column {
    pub title: String,
    pub width: f32,
    pub min: f32,
    pub sortable: bool,
    pub end: bool,
    pub editable: bool,
    pub id: usize,
}

impl Column {
    pub fn new(title: impl Into<String>, width: f32) -> Column {
        Column { title: title.into(), width, min: 48.0, sortable: false, end: false, editable: false, id: 0 }
    }

    pub fn editable(mut self) -> Column {
        self.editable = true;
        self
    }

    pub fn min(mut self, min: f32) -> Column {
        self.min = min;
        self.width = self.width.max(min);
        self
    }

    pub fn sortable(mut self) -> Column {
        self.sortable = true;
        self
    }

    pub fn end(mut self) -> Column {
        self.end = true;
        self
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Table {
    pub columns: Vec<Column>,
    pub sort: Option<(usize, bool)>,
    pub chosen: Option<usize>,
    pub rows: usize,
    pub row_height: f32,
    pub head_height: f32,
    pub first: f32,
    pub groups: Vec<(usize, String)>,
    pub editing: Option<CellEdit>,
    pub column: Option<usize>,
    pub multiple: bool,
    picked: Vec<usize>,
    anchor: Option<usize>,
    tree: Option<Vec<Branch>>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Line {
    Row(usize),
    Group(usize),
}

impl Table {
    pub fn new(mut columns: Vec<Column>) -> Table {
        for (i, c) in columns.iter_mut().enumerate() {
            c.id = i;
        }
        let (row_height, head_height) = crate::theme::sheet().table;
        Table { columns, sort: None, chosen: None, rows: 0, row_height, head_height, first: 0.0, groups: Vec::new(), editing: None, column: None, multiple: false, picked: Vec::new(), anchor: None, tree: None }
    }

    pub fn set_rows(&mut self, rows: usize) {
        self.tree = None;
        self.rows = rows;
        self.keep_inside();
    }

    fn keep_inside(&mut self) {
        let rows = self.rows;
        self.chosen = self.chosen.filter(|&c| c < rows);
        self.anchor = self.anchor.filter(|&c| c < rows);
        self.picked.truncate(self.picked.partition_point(|&r| r < rows));
        self.groups.retain(|g| g.0 < rows);
        self.editing = self.editing.take().filter(|e| e.row < rows);
        self.first = self.first.clamp(0.0, self.lines().saturating_sub(1) as f32);
    }

    pub fn set_groups(&mut self, mut groups: Vec<(usize, String)>) {
        groups.sort_by_key(|g| g.0);
        groups.dedup_by_key(|g| g.0);
        groups.retain(|g| g.0 < self.rows);
        self.groups = groups;
        self.first = self.first.clamp(0.0, self.lines().saturating_sub(1) as f32);
    }

    pub fn lines(&self) -> usize {
        self.rows + self.groups.len()
    }

    pub fn line_of(&self, row: usize) -> usize {
        row + self.groups.partition_point(|g| g.0 <= row)
    }

    fn group_line(&self, g: usize) -> usize {
        self.groups[g].0 + g
    }

    pub fn at_line(&self, line: usize) -> Line {
        let k = self.groups_before(line + 1);
        if k > 0 && self.group_line(k - 1) == line {
            Line::Group(k - 1)
        } else {
            Line::Row(line - k)
        }
    }

    pub fn group_of(&self, row: usize) -> Option<usize> {
        self.groups.partition_point(|g| g.0 <= row).checked_sub(1)
    }

    fn rows_before(&self, line: usize) -> usize {
        line - self.groups_before(line)
    }

    fn groups_before(&self, line: usize) -> usize {
        let (mut lo, mut hi) = (0, self.groups.len());
        while lo < hi {
            let mid = (lo + hi) / 2;
            if self.group_line(mid) < line {
                lo = mid + 1;
            } else {
                hi = mid;
            }
        }
        lo
    }

    pub fn width(&self) -> f32 {
        self.columns.iter().map(|c| c.width).sum()
    }

    pub fn column_x(&self, i: usize) -> f32 {
        self.columns.iter().take(i).map(|c| c.width).sum()
    }

    pub fn heading_rect(&self, i: usize) -> Rect {
        Rect { x: self.column_x(i), y: 0.0, w: self.columns.get(i).map_or(0.0, |c| c.width), h: self.head_height }
    }

    pub fn row_rect(&self, i: usize) -> Rect {
        self.line_rect(self.line_of(i))
    }

    pub fn line_rect(&self, line: usize) -> Rect {
        Rect { x: 0.0, y: self.head_height + (line as f32 - self.first) * self.row_h(), w: self.width(), h: self.row_h() }
    }

    pub fn cell_rect(&self, row: usize, column: usize) -> Rect {
        let r = self.row_rect(row);
        Rect { x: self.column_x(column), w: self.columns.get(column).map_or(0.0, |c| c.width), ..r }
    }

    fn row_h(&self) -> f32 {
        self.row_height.max(1.0)
    }

    pub fn rows_in(&self, tall: f32) -> usize {
        (((tall - self.head_height) / self.row_h()).floor().max(0.0)) as usize
    }

    pub fn visible(&self, tall: f32) -> Range<usize> {
        let lines = self.visible_lines(tall);
        self.rows_before(lines.start)..self.rows_before(lines.end)
    }

    pub fn visible_lines(&self, tall: f32) -> Range<usize> {
        let first = self.first.floor().max(0.0) as usize;
        let shown = ((tall - self.head_height) / self.row_h() + self.first.fract()).ceil().max(0.0) as usize;
        let lines = self.lines();
        first.min(lines)..first.saturating_add(shown).min(lines)
    }

    fn bar_and_fit(&self, wide: f32, tall: f32) -> Option<(Rect, usize)> {
        let fit = self.rows_in(tall).max(1);
        if self.lines() <= fit {
            return None;
        }
        let room = (tall - self.head_height).max(0.0);
        let thumb = (room * fit as f32 / self.lines() as f32).max(24.0).min(room);
        let top = self.head_height + (room - thumb) * (self.first / (self.lines() - fit) as f32).min(1.0);
        Some((Rect { x: wide - BAR - 2.0, y: top, w: BAR, h: thumb }, fit))
    }

    fn drag_bar(&mut self, y: f32, off: f32, tall: f32, fit: usize) -> bool {
        let room = (tall - self.head_height).max(0.0);
        let thumb = (room * fit as f32 / self.lines() as f32).max(24.0).min(room);
        let share = ((y - off - self.head_height) / (room - thumb).max(1.0)).clamp(0.0, 1.0);
        let first = share * self.lines().saturating_sub(fit) as f32;
        let moved = first != self.first;
        self.first = first;
        moved
    }

    pub fn bar(&self, wide: f32, tall: f32) -> Option<Rect> {
        self.bar_and_fit(wide, tall).map(|b| b.0)
    }

    pub fn row_at(&self, x: f32, y: f32, tall: f32) -> Option<usize> {
        if y < self.head_height || x < 0.0 || x >= self.width() || y >= tall {
            return None;
        }
        let line = (self.first + (y - self.head_height) / self.row_h()).floor() as usize;
        match self.at_line(line) {
            Line::Row(row) if line < self.lines() => Some(row),
            _ => None,
        }
    }

    pub fn cell_at(&self, x: f32, y: f32, tall: f32) -> Option<(usize, usize)> {
        let row = self.row_at(x, y, tall)?;
        let column = (0..self.columns.len()).find(|&i| x < self.column_x(i) + self.columns[i].width)?;
        Some((row, column))
    }

    pub fn heading_at(&self, x: f32, y: f32) -> Option<usize> {
        (0..self.columns.len()).find(|&i| self.heading_rect(i).contains(x, y))
    }

    pub fn edge_at(&self, x: f32, y: f32) -> Option<usize> {
        if !(0.0..self.head_height).contains(&y) {
            return None;
        }
        (0..self.columns.len()).find(|&i| (self.column_x(i) + self.columns[i].width - x).abs() <= GRAB)
    }

    pub fn resize(&mut self, i: usize, width: f32) {
        if let Some(c) = self.columns.get_mut(i) {
            c.width = width.max(c.min);
        }
    }

    pub fn fit(&mut self, i: usize, widest: f32) {
        self.resize(i, widest + 2.0 * PAD);
    }

    pub fn fit_to(&mut self, i: usize, fonts: &Fonts, scale: f32, cell: impl Fn(usize, usize) -> String) {
        let (plain, head) = (Words::Plain.style(scale), Words::Column.style(scale));
        let mut widest = fonts.layout(self.columns.get(i).map_or("", |c| &c.title), &head, None).width / scale;
        if self.columns.get(i).is_some_and(|c| c.sortable) {
            widest += PAD + 10.0;
        }
        for row in 0..self.rows.min(MOST_MEASURED) {
            widest = widest.max(fonts.layout(&cell(row, i), &plain, None).width / scale);
        }
        self.fit(i, widest);
    }

    pub fn sort_by(&mut self, i: usize) -> Option<(usize, bool)> {
        if !self.columns.get(i)?.sortable {
            return None;
        }
        let up = match self.sort {
            Some((c, up)) if c == i => !up,
            _ => true,
        };
        self.sort = Some((i, up));
        self.sort
    }

    pub fn move_column(&mut self, from: usize, to: usize) {
        if from >= self.columns.len() || to >= self.columns.len() {
            return;
        }
        let ids = |t: &Table, at: usize| t.columns[at].id;
        let sorted = self.sort.map(|(c, up)| (ids(self, c), up));
        let typing = self.editing.as_ref().map(|e| ids(self, e.column));
        let keys = self.column.filter(|&c| c < self.columns.len()).map(|c| ids(self, c));
        let column = self.columns.remove(from);
        self.columns.insert(to, column);
        let place = |id: usize, cols: &[Column]| cols.iter().position(|c| c.id == id).unwrap_or(0);
        self.sort = sorted.map(|(id, up)| (place(id, &self.columns), up));
        self.column = keys.map(|id| place(id, &self.columns));
        if let (Some(e), Some(id)) = (self.editing.as_mut(), typing) {
            e.column = place(id, &self.columns);
        }
    }

    pub fn column_place(&self, x: f32, from: usize) -> usize {
        (0..self.columns.len()).filter(|&i| i != from && self.column_x(i) + self.columns[i].width / 2.0 < x).count()
    }

    pub fn step_column(&mut self, by: isize) -> Option<usize> {
        let last = self.columns.len().checked_sub(1)?;
        let at = match self.column {
            Some(c) => (c.min(last) as isize).saturating_add(by).clamp(0, last as isize) as usize,
            None => 0,
        };
        self.column = Some(at);
        self.column
    }

    pub fn shift_column(&mut self, by: isize) -> Option<(usize, usize)> {
        let last = self.columns.len().checked_sub(1)?;
        let from = self.column?.min(last);
        let to = (from as isize).saturating_add(by).clamp(0, last as isize) as usize;
        if to == from {
            return None;
        }
        self.move_column(from, to);
        Some((from, to))
    }

    pub fn scroll(&mut self, lines: f32) {
        self.first = (self.first + lines).clamp(0.0, self.lines().saturating_sub(1) as f32);
    }

    pub fn reveal(&mut self, i: usize, tall: f32) {
        let fit = self.rows_in(tall).max(1) as f32;
        let line = self.line_of(i);
        let top = if self.groups.binary_search_by_key(&i, |g| g.0).is_ok() { line - 1 } else { line } as f32;
        let line = line as f32;
        if top < self.first {
            self.first = top;
        } else if line + 1.0 > self.first + fit {
            self.first = line + 1.0 - fit;
        }
        self.first = self.first.clamp(0.0, self.lines().saturating_sub(1) as f32);
    }

    pub fn step(&mut self, by: isize, tall: f32) -> Option<usize> {
        self.step_with(by, tall, false)
    }

    fn step_with(&mut self, by: isize, tall: f32, extend: bool) -> Option<usize> {
        if self.rows == 0 {
            return None;
        }
        let at = match self.chosen {
            Some(c) => (c as isize + by).clamp(0, self.rows as isize - 1) as usize,
            None => 0,
        };
        Some(self.go_to(at, tall, extend))
    }

    fn go_to(&mut self, at: usize, tall: f32, extend: bool) -> usize {
        if self.multiple {
            if extend {
                self.pick_to(at);
            } else {
                self.pick_one(at);
            }
        }
        self.chosen = Some(at);
        self.reveal(at, tall);
        at
    }

    pub fn home(&mut self, tall: f32) -> Option<usize> {
        self.step(isize::MIN / 2, tall)
    }

    pub fn end(&mut self, tall: f32) -> Option<usize> {
        self.chosen.get_or_insert(0);
        self.step(isize::MAX / 2, tall)
    }

    pub fn said(&self, tall: f32, cell: impl Fn(usize, usize) -> String) -> SaidTable {
        let rows = self.visible(tall).map(|row| SaidCells {
            index: row,
            cells: (0..self.columns.len()).map(|c| cell(row, c)).collect(),
            top: self.row_rect(row).y,
            height: self.row_h(),
            chosen: self.is_chosen(row),
            group: self.group_of(row).map(|g| self.groups[g].1.clone()),
            branch: self.said_branch(row),
        }).collect();
        SaidTable {
            headings: self.columns.iter().map(|c| c.title.clone()).collect(),
            columns: (0..self.columns.len()).map(|i| (self.column_x(i), self.columns[i].width)).collect(),
            sort: self.sort,
            total: self.rows,
            head_height: self.head_height,
            rows,
            column: self.column,
            many: self.multiple,
            tree: self.tree.is_some(),
            cursor: self.chosen,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Touch {
    Chose(usize),
    Opened(usize),
    Toggled(usize),
    Extended(usize),
    Sorted(usize, bool),
    Resizing(usize),
    Resized(usize),
    Fit(usize),
    Scrolling,
    Moving(usize, usize),
    Moved(usize, usize),
    Edit(usize, usize),
    LeftEdit,
    Open(usize),
    Closed(usize),
    Picked,
    Banded,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Hit {
    Edge(usize),
    Row(usize),
    Cell(usize, usize),
}

#[derive(Clone, Copy, Debug)]
enum Down {
    Edge(usize, f32),
    Head(usize, f32, Option<usize>),
    Row,
    Bar(f32),
    Band,
}

#[derive(Clone, Debug, Default)]
pub struct Gesture {
    down: Option<Down>,
    clicked: Option<(Hit, Instant)>,
    at: Option<(f32, f32)>,
    band: Option<Banding>,
}

impl Gesture {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn pointer(&mut self, table: &mut Table, event: Pointer, now: Instant, size: (f32, f32)) -> Option<Touch> {
        self.pointer_mods(table, event, now, Mods::default(), size)
    }

    pub fn pointer_mods(&mut self, table: &mut Table, event: Pointer, now: Instant, mods: Mods, size: (f32, f32)) -> Option<Touch> {
        let tall = size.1;
        match event {
            Pointer::Down(x, y) => {
                self.at = Some((x, y));
                self.press(table, (x, y), now, mods, size)
            }
            Pointer::Move(x, y) => {
                self.at = Some((x, y));
                match self.down? {
                    Down::Edge(c, off) => {
                        let left = table.column_x(c);
                        table.resize(c, x - off - left);
                        Some(Touch::Resizing(c))
                    }
                    Down::Bar(off) => {
                        let fit = table.rows_in(tall).max(1);
                        table.drag_bar(y, off, tall, fit).then_some(Touch::Scrolling)
                    }
                    Down::Head(h, from, place) => {
                        if place.is_none() && (x - from).abs() < SLOP {
                            return None;
                        }
                        let to = table.column_place(x, h);
                        self.down = Some(Down::Head(h, from, Some(to)));
                        Some(Touch::Moving(h, to))
                    }
                    Down::Band => self.band.as_mut()?.drag(table, x, y),
                    _ => None,
                }
            }
            Pointer::Up(x, y) => {
                self.at = Some((x, y));
                match self.down.take()? {
                    Down::Edge(c, _) => Some(Touch::Resized(c)),
                    Down::Head(h, _, Some(_)) => {
                        let to = table.column_place(x, h);
                        table.move_column(h, to);
                        (to != h).then_some(Touch::Moved(h, to))
                    }
                    Down::Head(h, ..) if table.heading_at(x, y) == Some(h) => table.sort_by(h).map(|(c, up)| Touch::Sorted(c, up)),
                    Down::Band => self.band.take().and_then(|b| b.let_go(table)),
                    _ => None,
                }
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

    fn press(&mut self, table: &mut Table, (x, y): (f32, f32), now: Instant, mods: Mods, size: (f32, f32)) -> Option<Touch> {
        let tall = size.1;
        if let Some((bar, fit)) = table.bar_and_fit(size.0, tall).filter(|(b, _)| x >= b.x - GRAB + BAR && x <= size.0) {
            let off = if y >= bar.y && y <= bar.y + bar.h { y - bar.y } else { bar.h / 2.0 };
            self.down = Some(Down::Bar(off));
            return table.drag_bar(y, off, size.1, fit).then_some(Touch::Scrolling);
        }
        if let Some(e) = &table.editing {
            if !table.cell_rect(e.row, e.column).contains(x, y) {
                self.down = None;
                return Some(Touch::LeftEdit);
            }
            return None;
        }
        let double = self.clicked.take().filter(|(_, t)| now.duration_since(*t) <= super::host::host().double_click());
        if let Some(c) = table.edge_at(x, y) {
            let off = x - (table.column_x(c) + table.columns[c].width);
            if double.is_some_and(|(was, _)| was == Hit::Edge(c)) {
                self.down = None;
                return Some(Touch::Fit(c));
            }
            self.down = Some(Down::Edge(c, off));
            self.clicked = Some((Hit::Edge(c), now));
            return None;
        }
        if let Some(h) = table.heading_at(x, y) {
            self.down = Some(Down::Head(h, x, None));
            return None;
        }
        let Some((row, column)) = table.cell_at(x, y, tall) else {
            if table.multiple && table.below_rows(x, y, tall) {
                self.begin_band(table, (x, y), None, mods.ctrl);
            }
            return None;
        };
        self.press_row(table, (x, y), (row, column), mods, now, double)
    }

    fn press_row(&mut self, table: &mut Table, (x, y): (f32, f32), (row, column): (usize, usize), mods: Mods, now: Instant, double: Option<(Hit, Instant)>) -> Option<Touch> {
        self.down = Some(Down::Row);
        if column == 0 && table.control_rect(row).is_some_and(|r| r.contains(x, y)) {
            return table.toggle_row(row);
        }
        let again = |hit| double.is_some_and(|(was, _)| was == hit);
        let editable = table.columns[column].editable && !mods.ctrl && !mods.shift;
        if editable && again(Hit::Cell(row, column)) {
            self.clicked = None;
            return Some(Touch::Edit(row, column));
        }
        let opened = again(Hit::Row(row));
        self.clicked = Some((if editable { Hit::Cell(row, column) } else { Hit::Row(row) }, now));
        if table.column.is_some() {
            table.column = Some(column);
        }
        if table.multiple {
            if mods.ctrl {
                self.begin_band(table, (x, y), Some(row), true);
                return None;
            }
            if mods.shift {
                table.pick_to(row);
                return Some(Touch::Extended(row));
            }
            table.pick_one(row);
        } else {
            if mods.ctrl {
                return Some(Touch::Toggled(row));
            }
            if mods.shift {
                return Some(Touch::Extended(row));
            }
            table.chosen = Some(row);
        }
        Some(if opened { table.toggle_row(row).unwrap_or(Touch::Opened(row)) } else { Touch::Chose(row) })
    }

    fn begin_band(&mut self, table: &Table, (x, y): (f32, f32), row: Option<usize>, keep: bool) {
        self.band = Some(Banding::new(table, (x, y), row, keep));
        self.down = Some(Down::Band);
    }

    pub fn band(&self, table: &Table, size: (f32, f32)) -> Option<Rect> {
        self.band.as_ref()?.rect(table, size)
    }

    pub fn moving(&self) -> Option<(usize, usize)> {
        match self.down? {
            Down::Head(h, _, Some(to)) => Some((h, to)),
            _ => None,
        }
    }

    pub fn hovering(&self, table: &Table, tall: f32) -> Option<usize> {
        let (x, y) = self.at?;
        table.row_at(x, y, tall)
    }

    pub fn cursor(&self, table: &Table, x: f32, y: f32) -> Cursor {
        if matches!(self.down, Some(Down::Edge(..))) || table.edge_at(x, y).is_some() {
            Cursor::ColResize
        } else if table.editing.as_ref().is_some_and(|e| table.cell_rect(e.row, e.column).contains(x, y)) {
            Cursor::Text
        } else if table.heading_at(x, y).is_some_and(|i| table.columns[i].sortable) {
            Cursor::Hand
        } else {
            Cursor::Arrow
        }
    }
}

fn paint_arrow(p: &mut Painter, pen: &Pen, x: f32, y: f32, up: bool, colour: Colour) {
    let k = pen.scale;
    let mut pb = tiny_skia::PathBuilder::new();
    let (w, h) = (8.0 * k, 5.0 * k);
    let (x, y) = (x * k, y * k);
    if up {
        pb.move_to(x, y + h / 2.0);
        pb.line_to(x + w, y + h / 2.0);
        pb.line_to(x + w / 2.0, y - h / 2.0);
    } else {
        pb.move_to(x, y - h / 2.0);
        pb.line_to(x + w, y - h / 2.0);
        pb.line_to(x + w / 2.0, y + h / 2.0);
    }
    pb.close();
    if let Some(path) = pb.finish() {
        p.fill_path(path, tiny_skia::FillRule::Winding, colour);
    }
}

pub fn paint_table(canvas: &mut PixmapMut, pen: &Pen, table: &Table, hover: Option<usize>, cell: impl Fn(usize, usize) -> String) {
    let mut p = Painter::default();
    paint_table_on(&mut p, pen, (canvas.width(), canvas.height()), table, hover, cell);
    p.play(canvas);
}

fn paint_headings<'a>(p: &mut Painter<'a>, pen: &Pen<'a>, table: &Table, head: &super::text::Style) {
    let t = pen.theme;
    for (i, col) in table.columns.iter().enumerate() {
        let r = table.heading_rect(i);
        let sorted = table.sort.filter(|&(c, _)| c == i);
        let room = Rect { x: r.x + PAD, w: (r.w - 2.0 * PAD - if sorted.is_some() { 14.0 } else { 0.0 }).max(0.0), ..r };
        text_in_on(p, pen, &col.title, head, room, if sorted.is_some() { t.fg } else { t.muted }, col.end);
        if let Some((_, up)) = sorted {
            paint_arrow(p, pen, r.x + r.w - PAD - 8.0, r.y + r.h / 2.0, up, t.accent);
        }
    }
}

pub fn paint_table_on<'a>(p: &mut Painter<'a>, pen: &Pen<'a>, size: (u32, u32), table: &Table, hover: Option<usize>, cell: impl Fn(usize, usize) -> String) {
    let k = pen.scale;
    let tall = size.1 as f32 / k;
    let wide = (size.0 as f32 / k).max(table.width());
    let (plain, head, group) = (Words::Plain.style(k), Words::Column.style(k), Words::Section.style(k));
    let t = pen.theme;
    paint_headings(p, pen, table, &head);
    fill_on(p, pen, Rect { x: 0.0, y: table.head_height - 1.0, w: wide, h: 1.0 }, t.control_edge);
    for line in table.visible_lines(tall) {
        let r = table.line_rect(line);
        if r.y + r.h <= table.head_height {
            continue;
        }
        let row = match table.at_line(line) {
            Line::Row(row) => row,
            Line::Group(g) => {
                fill_on(p, pen, Rect { w: wide, ..r }, t.band);
                text_in_on(p, pen, &table.groups[g].1, &group, Rect { x: r.x + PAD, w: (wide - 2.0 * PAD).max(0.0), ..r }, t.fg, false);
                continue;
            }
        };
        let chosen = table.is_chosen(row);
        if chosen {
            fill_on(p, pen, Rect { w: wide, ..r }, t.picked);
        } else if hover == Some(row) {
            fill_on(p, pen, Rect { w: wide, ..r }, t.hover);
        }
        if table.multiple && table.chosen == Some(row) && !chosen {
            edge(p, pen, Rect { w: wide, ..r }, super::theme::sheet().focus_width, t.accent);
        }
        let ink = if chosen { t.on_picked } else { t.fg };
        for (i, col) in table.columns.iter().enumerate() {
            let c = table.cell_rect(row, i);
            let lead = if i == 0 { table.lead(row) } else { 0.0 };
            if i == 0 {
                tree::paint_control(p, pen, table, row, ink);
            }
            let room = Rect { x: c.x + PAD + lead, w: (c.w - 2.0 * PAD - lead).max(0.0), ..c };
            match &table.editing {
                Some(e) if (e.row, e.column) == (row, i) => cells::paint_edit(p, pen, e, c, room, &plain),
                _ => text_in_on(p, pen, &cell(row, i), &plain, room, ink, col.end),
            }
        }
    }
    if let Some(c) = table.column.filter(|&c| c < table.columns.len() && table.editing.is_none()) {
        let r = match table.chosen {
            Some(row) => table.cell_rect(row, c),
            None => table.heading_rect(c),
        };
        if r.y + r.h > table.head_height || table.chosen.is_none() {
            edge(p, pen, r, super::theme::sheet().focus_width, t.accent);
        }
    }
    if let Some(bar) = table.bar(size.0 as f32 / k, tall) {
        fill_on(p, pen, bar, t.control_edge);
    }
}

pub fn paint_column_drop(canvas: &mut PixmapMut, pen: &Pen, table: &Table, moving: Option<(usize, usize)>) {
    let mut p = Painter::default();
    paint_column_drop_on(&mut p, pen, table, moving);
    p.play(canvas);
}

pub fn paint_column_drop_on(p: &mut Painter, pen: &Pen, table: &Table, moving: Option<(usize, usize)>) {
    let Some((from, to)) = moving else { return };
    let x = if to > from { table.column_x(to) + table.columns.get(to).map_or(0.0, |c| c.width) } else { table.column_x(to) };
    fill_on(p, pen, Rect { x: (x - 1.0).max(0.0), y: 0.0, w: 2.0, h: table.head_height }, pen.theme.accent);
}

#[cfg(test)]
mod tests;

#[cfg(test)]
mod tree_pick_tests;
