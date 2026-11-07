//! Rows that open and close under a parent, set in by their depth in the first column.

use super::super::colour::Colour;
use super::super::paint::Painter;
use super::super::widget::{Pen, Rect, SaidBranch, CHEVRON};
use super::{Table, PAD};

const CONTROL: f32 = 24.0;
const MOST_DEEP: usize = 64;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Branch {
    pub id: usize,
    pub depth: usize,
    pub open: Option<bool>,
    place: (u32, u32),
}

impl Branch {
    pub fn leaf(id: usize) -> Branch {
        Branch { id, depth: 0, open: None, place: (0, 1) }
    }

    pub fn closed(id: usize) -> Branch {
        Branch { open: Some(false), ..Branch::leaf(id) }
    }

    pub fn opened(id: usize) -> Branch {
        Branch { open: Some(true), ..Branch::leaf(id) }
    }

    pub fn depth(mut self, depth: usize) -> Branch {
        self.depth = depth;
        self
    }
}

fn place(rows: &mut [Branch]) {
    let mut count: Vec<u32> = Vec::new();
    for r in rows.iter_mut() {
        r.depth = r.depth.min(MOST_DEEP);
        count.truncate(r.depth + 1);
        count.resize(r.depth + 1, 0);
        r.place.0 = count[r.depth];
        count[r.depth] += 1;
    }
    let mut total: Vec<Option<u32>> = Vec::new();
    for r in rows.iter_mut().rev() {
        total.truncate(r.depth + 1);
        total.resize(r.depth + 1, None);
        r.place.1 = *total[r.depth].get_or_insert(r.place.0 + 1);
    }
}

impl Table {
    pub fn set_tree(&mut self, mut rows: Vec<Branch>) {
        place(&mut rows);
        self.rows = rows.len();
        self.tree = Some(rows);
        self.row_height = self.row_height.max(CONTROL);
        self.keep_inside();
    }

    pub fn is_tree(&self) -> bool {
        self.tree.is_some()
    }

    pub fn branch(&self, row: usize) -> Option<&Branch> {
        self.tree.as_ref()?.get(row)
    }

    pub fn parent_of(&self, row: usize) -> Option<usize> {
        let tree = self.tree.as_ref()?;
        let depth = tree.get(row)?.depth;
        (0..row).rev().find(|&i| tree[i].depth < depth)
    }

    pub fn is_open(&self, row: usize) -> Option<bool> {
        self.branch(row)?.open
    }

    pub fn open_row(&mut self, row: usize, mut kids: Vec<Branch>) -> bool {
        let Some(tree) = &mut self.tree else { return false };
        let Some(&parent) = tree.get(row).filter(|b| b.open == Some(false) && b.depth < MOST_DEEP) else { return false };
        if kids.is_empty() {
            tree[row].open = None;
            return true;
        }
        for k in &mut kids {
            k.depth += parent.depth + 1;
        }
        place(&mut kids);
        let n = kids.len();
        tree[row].open = Some(true);
        tree.splice(row + 1..row + 1, kids);
        let line = self.line_of(row) + 1;
        self.rows += n;
        self.shift_rows(row + 1, n as isize, row);
        if (line as f32) < self.first.floor() {
            self.first += n as f32;
        }
        true
    }

    pub fn close_row(&mut self, row: usize) -> bool {
        let Some(tree) = &mut self.tree else { return false };
        let Some(&parent) = tree.get(row).filter(|b| b.open == Some(true)) else { return false };
        let end = row + 1 + tree[row + 1..].iter().take_while(|c| c.depth > parent.depth).count();
        tree[row].open = Some(false);
        tree.drain(row + 1..end);
        let n = end - row - 1;
        self.rows -= n;
        self.shift_rows(row + 1, -(n as isize), row);
        self.keep_inside();
        true
    }

    fn shift_rows(&mut self, at: usize, by: isize, to: usize) {
        let gone = if by < 0 { at..at + by.unsigned_abs() } else { at..at };
        let moved = |r: usize| if r >= gone.end { r.wrapping_add_signed(by) } else if r >= gone.start { to } else { r };
        self.chosen = self.chosen.map(moved);
        self.anchor = self.anchor.map(moved);
        if let Some(e) = &mut self.editing {
            if gone.contains(&e.row) {
                self.editing = None;
            } else {
                e.row = moved(e.row);
            }
        }
        let from = self.picked.partition_point(|&r| r < gone.start);
        let kept = self.picked.partition_point(|&r| r < gone.end);
        self.picked.drain(from..kept);
        for r in &mut self.picked[from..] {
            *r = r.wrapping_add_signed(by);
        }
        for g in &mut self.groups {
            g.0 = if g.0 >= gone.end { g.0.wrapping_add_signed(by) } else if g.0 >= gone.start { at } else { g.0 };
        }
        let starts: Vec<usize> = self.groups.iter().map(|g| g.0).collect();
        let mut i = 0;
        self.groups.retain(|_| {
            i += 1;
            starts.get(i).is_none_or(|&next| next != starts[i - 1])
        });
    }

    pub fn control_rect(&self, row: usize) -> Option<Rect> {
        let b = self.branch(row).filter(|b| b.open.is_some())?;
        let r = self.row_rect(row);
        Some(Rect { x: PAD + self.indent(b.depth), w: CONTROL, ..r })
    }

    fn indent(&self, depth: usize) -> f32 {
        super::super::theme::sheet().tree_indent * depth as f32
    }

    pub fn lead(&self, row: usize) -> f32 {
        self.branch(row).map_or(0.0, |b| self.indent(b.depth) + CONTROL)
    }

    pub(super) fn said_branch(&self, row: usize) -> Option<SaidBranch> {
        let b = self.branch(row)?;
        Some(SaidBranch { level: b.depth + 1, open: b.open, place: b.place.0 as usize, of: b.place.1 as usize })
    }

    pub(super) fn toggle_row(&mut self, row: usize) -> Option<super::Touch> {
        match self.is_open(row)? {
            true => self.close_row(row).then_some(super::Touch::Closed(row)),
            false => Some(super::Touch::Open(row)),
        }
    }
}

pub(super) fn paint_control(p: &mut Painter, pen: &Pen, table: &Table, row: usize, ink: Colour) {
    let Some(c) = table.control_rect(row) else { return };
    let k = pen.scale;
    let name = if table.is_open(row) == Some(true) { "pan-down-symbolic" } else { "pan-end-symbolic" };
    let size = CHEVRON;
    let (x, y) = ((c.x + (c.w - size) / 2.0) * k, (c.y + (c.h - size) / 2.0) * k);
    pen.icons.draw_on(p, name, x.round() as i32, y.round() as i32, (size * k).round() as u32, 1, ink);
}
