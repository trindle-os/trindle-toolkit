//! A screen of cells painted as a terminal does: each cell has its words, colours and marks; backgrounds are filled in runs, words are laid out one run to a stretch of one look (never one to a cell), a wide character takes two cells with a spacer after it, and the cursor and the selection are drawn over. The program keeps a `Screen` and calls `paint_cells`.

use tiny_skia::PixmapMut;

use super::colour::Colour;
use super::draw::{fill_on, words_on};
use super::paint::Painter;
use super::text::Style;
use super::widget::{Pen, Rect};

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Cell {
    pub text: String,
    pub fg: Option<Colour>,
    pub bg: Option<Colour>,
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
    pub strike: bool,
    pub inverse: bool,
    pub spacer: bool,
}

impl Cell {
    pub fn new(text: impl Into<String>) -> Cell {
        Cell { text: text.into(), ..Cell::default() }
    }

    fn shown(&self) -> &str {
        if self.text.is_empty() { " " } else { &self.text }
    }

    fn same_look(&self, other: &Cell) -> bool {
        (self.fg, self.bold, self.italic, self.inverse, self.bg) == (other.fg, other.bold, other.italic, other.inverse, other.bg)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Screen {
    cols: usize,
    rows: usize,
    cells: Vec<Cell>,
}

impl Screen {
    pub fn new(cols: usize, rows: usize) -> Screen {
        Screen { cols, rows, cells: vec![Cell::default(); cols * rows] }
    }

    pub fn cols(&self) -> usize {
        self.cols
    }

    pub fn rows(&self) -> usize {
        self.rows
    }

    pub fn cell(&self, col: usize, row: usize) -> &Cell {
        assert!(col < self.cols, "column {col} is off a screen {} across", self.cols);
        &self.cells[row * self.cols + col]
    }

    pub fn cell_mut(&mut self, col: usize, row: usize) -> &mut Cell {
        assert!(col < self.cols, "column {col} is off a screen {} across", self.cols);
        &mut self.cells[row * self.cols + col]
    }

    pub fn put(&mut self, col: usize, row: usize, text: &str, look: &Cell) -> usize {
        let mut col = col.min(self.cols);
        if row >= self.rows {
            return col;
        }
        for c in text.chars() {
            let w = super::grid::cell_width(c);
            if w == 0 {
                if c.is_control() {
                    continue;
                }
                let mut at = col.saturating_sub(1);
                if at > 0 && self.cell(at, row).spacer {
                    at -= 1;
                }
                if col > 0 {
                    self.cell_mut(at, row).text.push(c);
                }
                continue;
            }
            if col + w > self.cols {
                break;
            }
            self.clear_halves(col, w, row, look);
            *self.cell_mut(col, row) = Cell { text: c.to_string(), spacer: false, ..look.clone() };
            if w == 2 {
                *self.cell_mut(col + 1, row) = Cell { text: String::new(), spacer: true, ..look.clone() };
            }
            col += w;
        }
        col
    }

    fn clear_halves(&mut self, col: usize, w: usize, row: usize, look: &Cell) {
        let blank = |cell: &Cell| Cell { text: String::new(), spacer: false, ..cell.clone() };
        if col > 0 && self.cell(col, row).spacer {
            *self.cell_mut(col - 1, row) = blank(look);
        }
        let last = col + w - 1;
        if last + 1 < self.cols && self.cell(last + 1, row).spacer {
            *self.cell_mut(last + 1, row) = blank(look);
        }
    }

    pub fn row_text(&self, row: usize) -> String {
        let s: String = (0..self.cols).map(|c| self.cell(c, row)).filter(|c| !c.spacer).map(Cell::shown).collect();
        s.trim_end().to_string()
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Shape {
    Block,
    Bar,
    Underline,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Caret {
    pub col: usize,
    pub row: usize,
    pub shape: Shape,
    pub focused: bool,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Selection {
    pub from: (usize, usize),
    pub to: (usize, usize),
    pub block: bool,
}

impl Selection {
    fn ordered(&self) -> ((usize, usize), (usize, usize)) {
        if (self.from.1, self.from.0) <= (self.to.1, self.to.0) { (self.from, self.to) } else { (self.to, self.from) }
    }

    pub fn holds(&self, col: usize, row: usize) -> bool {
        let (a, b) = self.ordered();
        if self.block {
            let (l, r) = (a.0.min(b.0), a.0.max(b.0));
            return row >= a.1 && row <= b.1 && col >= l && col <= r;
        }
        (row, col) >= (a.1, a.0) && (row, col) <= (b.1, b.0)
    }

    pub fn words(&self, screen: &Screen) -> String {
        if screen.rows == 0 {
            return String::new();
        }
        let (a, b) = self.ordered();
        let mut lines = Vec::new();
        for row in a.1..=b.1.min(screen.rows.saturating_sub(1)) {
            let line: String = (0..screen.cols).filter(|&c| self.holds(c, row)).map(|c| screen.cell(c, row)).filter(|c| !c.spacer)
                .map(Cell::shown).collect();
            lines.push(line.trim_end().to_string());
        }
        lines.join("\n")
    }
}

pub fn cell_metrics(pen: &Pen, size: f32) -> (Style, (f32, f32)) {
    let style = Style::monospace(size * pen.scale);
    let (w, h) = pen.fonts.cell_size(&style);
    (style, (w / pen.scale, h / pen.scale))
}

pub fn paint_cells(canvas: &mut PixmapMut, pen: &Pen, screen: &Screen, size: f32, caret: Option<Caret>, selection: Option<Selection>) {
    let mut p = Painter::default();
    paint_cells_on(&mut p, pen, screen, size, caret, selection);
    p.play(canvas);
}

pub fn paint_cells_on<'a>(p: &mut Painter<'a>, pen: &Pen<'a>, screen: &Screen, size: f32, caret: Option<Caret>, selection: Option<Selection>) {
    let (style, (cw, ch)) = cell_metrics(pen, size);
    for row in 0..screen.rows {
        paint_backgrounds(p, pen, screen, row, (cw, ch), selection);
        paint_words(p, pen, screen, row, (cw, ch), &style, selection);
    }
    if let Some(c) = caret.filter(|c| c.col < screen.cols && c.row < screen.rows) {
        paint_caret(p, pen, screen, c, (cw, ch), &style);
    }
}

fn cell_colours(pen: &Pen, c: &Cell) -> (Colour, Option<Colour>) {
    let t = pen.theme;
    let (fg, bg) = (c.fg.unwrap_or(t.fg), c.bg);
    if c.inverse { (bg.unwrap_or(t.raised), Some(fg)) } else { (fg, bg) }
}

fn paint_backgrounds(p: &mut Painter, pen: &Pen, screen: &Screen, row: usize, (cw, ch): (f32, f32), selection: Option<Selection>) {
    let y = row as f32 * ch;
    let mut col = 0;
    while col < screen.cols {
        let (_, bg) = cell_colours(pen, screen.cell(col, row));
        let mut end = col + 1;
        while end < screen.cols && cell_colours(pen, screen.cell(end, row)).1 == bg {
            end += 1;
        }
        if let Some(bg) = bg {
            fill_on(p, pen, Rect { x: col as f32 * cw, y, w: (end - col) as f32 * cw, h: ch }, bg);
        }
        col = end;
    }
    let Some(sel) = selection else { return };
    let mut col = 0;
    while col < screen.cols {
        if sel.holds(col, row) {
            let mut end = col + 1;
            while end < screen.cols && sel.holds(end, row) {
                end += 1;
            }
            fill_on(p, pen, Rect { x: col as f32 * cw, y, w: (end - col) as f32 * cw, h: ch }, pen.theme.picked);
            col = end;
        } else {
            col += 1;
        }
    }
}

fn paint_words<'a>(p: &mut Painter<'a>, pen: &Pen<'a>, screen: &Screen, row: usize, (cw, ch): (f32, f32), style: &Style, selection: Option<Selection>) {
    let t = pen.theme;
    let k = pen.scale;
    let y = row as f32 * ch;
    let mut col = 0;
    while col < screen.cols {
        let first = screen.cell(col, row);
        let mut end = col + 1;
        let picked = |c: usize| selection.is_some_and(|s| s.holds(c, row));
        while end < screen.cols && screen.cell(end, row).same_look(first) && picked(end) == picked(col) {
            end += 1;
        }
        let (fg, _) = cell_colours(pen, first);
        let fg = if picked(col) { t.on_picked } else { fg };
        let text: String = (col..end).map(|c| screen.cell(c, row)).filter(|c| !c.spacer).map(Cell::shown).collect();
        if !text.trim().is_empty() {
            let look = Style { weight: if first.bold { 700 } else { style.weight }, italic: first.italic || style.italic, ..style.clone() };
            let laid = pen.fonts.layout(&text, &look, None).snapped(&text, cw * k);
            let top = (y + (ch - laid.height / k) / 2.0) * k;
            words_on(p, pen.fonts, laid, col as f32 * cw * k, top, fg);
        }
        for c in col..end {
            let cellk = screen.cell(c, row);
            let (ink, _) = cell_colours(pen, cellk);
            if cellk.underline {
                fill_on(p, pen, Rect { x: c as f32 * cw, y: y + ch - 1.5, w: cw, h: 1.0 }, ink);
            }
            if cellk.strike {
                fill_on(p, pen, Rect { x: c as f32 * cw, y: y + ch / 2.0, w: cw, h: 1.0 }, ink);
            }
        }
        col = end;
    }
}

fn paint_caret<'a>(p: &mut Painter<'a>, pen: &Pen<'a>, screen: &Screen, c: Caret, (cw, ch): (f32, f32), style: &Style) {
    let t = pen.theme;
    let k = pen.scale;
    let wide = c.col + 1 < screen.cols && screen.cell(c.col + 1, c.row).spacer;
    let w = if wide { 2.0 * cw } else { cw };
    let (x, y) = (c.col as f32 * cw, c.row as f32 * ch);
    let line = 2.0;
    match (c.shape, c.focused) {
        (Shape::Block, true) => {
            fill_on(p, pen, Rect { x, y, w, h: ch }, t.accent);
            let cell = screen.cell(c.col, c.row);
            if !cell.text.is_empty() {
                let laid = pen.fonts.layout(&cell.text, style, None).snapped(&cell.text, cw * k);
                let top = (y + (ch - laid.height / k) / 2.0) * k;
                words_on(p, pen.fonts, laid, x * k, top, t.accent_text);
            }
        }
        (Shape::Block, false) => {
            for r in [Rect { x, y, w, h: line }, Rect { x, y: y + ch - line, w, h: line }, Rect { x, y, w: line, h: ch }, Rect { x: x + w - line, y, w: line, h: ch }] {
                fill_on(p, pen, r, t.accent);
            }
        }
        (Shape::Bar, _) => fill_on(p, pen, Rect { x, y, w: line, h: ch }, t.accent),
        (Shape::Underline, _) => fill_on(p, pen, Rect { x, y: y + ch - line, w, h: line }, t.accent),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[should_panic(expected = "off a screen")]
    fn a_column_past_the_end_is_not_the_next_rows_start() {
        Screen::new(4, 3).cell(4, 0);
    }

    #[test]
    #[should_panic]
    fn a_row_past_the_end_panics() {
        Screen::new(4, 3).cell(0, 3);
    }

    #[test]
    fn a_wide_character_takes_two_cells_with_a_spacer_and_marks_join() {
        let mut s = Screen::new(8, 1);
        let end = s.put(0, 0, "a日e\u{301}b", &Cell::default());
        assert_eq!(end, 5);
        assert!(s.cell(2, 0).spacer && !s.cell(1, 0).spacer);
        assert_eq!(s.cell(3, 0).text, "e\u{301}");
        assert_eq!(s.row_text(0), "a日e\u{301}b");
        let mut w = Screen::new(4, 1);
        w.put(0, 0, "日\u{301}", &Cell::default());
        assert_eq!(w.row_text(0), "日\u{301}");
        let mut t = Screen::new(3, 1);
        assert_eq!(t.put(0, 0, "ab日", &Cell::default()), 2);
    }

    #[test]
    fn writing_over_half_of_a_wide_character_leaves_no_orphan_half() {
        let look = Cell::default();
        let mut s = Screen::new(6, 1);
        s.put(0, 0, "日本", &look);
        s.put(1, 0, "x", &look);
        assert!(!s.cell(1, 0).spacer && s.cell(0, 0).text.is_empty(), "the spacer was written over, so its wide half goes");
        assert_eq!(s.row_text(0), " x本");
        let mut t = Screen::new(6, 1);
        t.put(0, 0, "日本", &look);
        t.put(0, 0, "x", &look);
        assert!(!t.cell(1, 0).spacer, "the wide half was written over, so its spacer goes");
        assert_eq!(t.row_text(0), "x 本");
        let mut u = Screen::new(6, 1);
        u.put(0, 0, "日本", &look);
        u.put(1, 0, "語", &look);
        assert_eq!(u.row_text(0), " 語");
        assert!(!u.cell(3, 0).spacer && u.cell(2, 0).spacer, "a new wide character fits in the cells it took");
    }

    #[test]
    fn put_stays_inside_the_grid_and_skips_control_characters() {
        let mut s = Screen::new(3, 1);
        assert_eq!(s.put(0, 5, "x", &Cell::default()), 0);
        assert_eq!(s.put(9, 0, "x", &Cell::default()), 3);
        assert_eq!(s.row_text(0), "");
        s.put(0, 0, "a\x1bb", &Cell::default());
        assert_eq!(s.row_text(0), "ab");
        let empty = Screen::new(0, 0);
        assert_eq!(Selection { from: (0, 0), to: (3, 0), block: false }.words(&empty), "");
    }

    #[test]
    fn a_selection_is_a_stream_or_a_block_whichever_end_comes_first() {
        let stream = Selection { from: (5, 2), to: (3, 1), block: false };
        assert!(stream.holds(4, 1) && !stream.holds(2, 1) && stream.holds(0, 2) && !stream.holds(6, 2));
        let block = Selection { from: (5, 2), to: (3, 1), block: true };
        assert!(block.holds(4, 2) && !block.holds(2, 2) && !block.holds(4, 3));
    }

    #[test]
    fn selected_words_are_rows_trimmed_and_joined() {
        let mut s = Screen::new(6, 2);
        s.put(0, 0, "ab 日", &Cell::default());
        s.put(0, 1, "cd", &Cell::default());
        let all = Selection { from: (0, 0), to: (5, 1), block: false };
        assert_eq!(all.words(&s), "ab 日\ncd");
        let part = Selection { from: (1, 0), to: (1, 1), block: false };
        assert_eq!(part.words(&s), "b 日\ncd");
    }
}
