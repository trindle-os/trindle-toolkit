//! A terminal's screen fed by a program: ANSI colours and marks, the cursor, line feeds that scroll into a scroll-back, painted with `cellgrid`.

use std::collections::VecDeque;

use tiny_skia::PixmapMut;

use super::cellgrid::{paint_cells_on, Caret, Cell, Screen, Shape};
use super::colour::Colour;
use super::grid::cell_width;
use super::paint::Painter;
use super::theme::{self, clear};
use super::widget::{Pen, Reading};

const READABLE: f32 = 4.5;

const MOST_OSC: usize = 4096;

#[derive(Clone, Debug, PartialEq)]
enum Parse {
    Ground,
    Esc,
    Csi(Vec<u16>, Option<u16>, bool),
    Osc(bool, usize),
    Set,
}

#[derive(Clone, Debug)]
pub struct Term {
    cols: usize,
    rows: usize,
    lines: VecDeque<Vec<Cell>>,
    pub cursor: (usize, usize),
    pub cursor_shown: bool,
    pub back: usize,
    pub most: usize,
    pub bell: bool,
    look: Cell,
    parse: Parse,
    partial: Vec<u8>,
}

impl Term {
    pub fn new(cols: usize, rows: usize) -> Term {
        let (cols, rows) = (cols.max(1), rows.max(1));
        Term { cols, rows, lines: (0..rows).map(|_| vec![Cell::default(); cols]).collect(), cursor: (0, 0), cursor_shown: true, back: 0, most: 10_000,
               bell: false, look: Cell::default(), parse: Parse::Ground, partial: Vec::new() }
    }

    pub fn size(&self) -> (usize, usize) {
        (self.cols, self.rows)
    }

    pub fn scrolled_off(&self) -> usize {
        self.lines.len() - self.rows
    }

    pub fn feed(&mut self, bytes: &[u8]) {
        self.back = 0;
        let mut all = std::mem::take(&mut self.partial);
        all.extend_from_slice(bytes);
        let mut rest = &all[..];
        while !rest.is_empty() {
            let (good, bad) = match std::str::from_utf8(rest) {
                Ok(s) => (s, None),
                Err(e) => (std::str::from_utf8(&rest[..e.valid_up_to()]).unwrap_or_default(), Some(e)),
            };
            good.chars().for_each(|c| self.take(c));
            let Some(e) = bad else { break };
            rest = &rest[e.valid_up_to()..];
            match e.error_len() {
                Some(n) => {
                    self.take(char::REPLACEMENT_CHARACTER);
                    rest = &rest[n..];
                }
                None => {
                    self.partial = rest.to_vec();
                    break;
                }
            }
        }
    }

    fn take(&mut self, c: char) {
        match std::mem::replace(&mut self.parse, Parse::Ground) {
            Parse::Ground => self.plain(c),
            Parse::Esc => match c {
                '[' => self.parse = Parse::Csi(Vec::new(), None, false),
                ']' => self.parse = Parse::Osc(false, 0),
                'c' => *self = Term { most: self.most, ..Term::new(self.cols, self.rows) },
                '(' | ')' => self.parse = Parse::Set,
                _ => {}
            },
            Parse::Set => {}
            Parse::Csi(mut nums, now, private) => match c {
                '0'..='9' => {
                    let n = now.unwrap_or(0).saturating_mul(10).saturating_add(c as u16 - b'0' as u16);
                    self.parse = Parse::Csi(nums, Some(n), private);
                }
                ';' | ':' => {
                    if nums.len() < 32 {
                        nums.push(now.unwrap_or(0));
                    }
                    self.parse = Parse::Csi(nums, None, private);
                }
                '?' | '>' | '=' => self.parse = Parse::Csi(nums, now, true),
                '\u{40}'..='\u{7e}' => {
                    nums.extend(now);
                    self.control(c, &nums, private);
                }
                _ => self.parse = Parse::Csi(nums, now, private),
            },
            Parse::Osc(esc, n) => match c {
                '\u{7}' => {}
                '\\' if esc => {}
                _ if n >= MOST_OSC => {}
                _ => self.parse = Parse::Osc(c == '\u{1b}', n + 1),
            },
        }
    }

    fn plain(&mut self, c: char) {
        match c {
            '\u{1b}' => self.parse = Parse::Esc,
            '\r' => self.cursor.0 = 0,
            '\n' | '\u{b}' | '\u{c}' => self.line_feed(),
            '\u{8}' => self.cursor.0 = self.cursor.0.saturating_sub(1),
            '\t' => self.cursor.0 = ((self.cursor.0 / 8 + 1) * 8).min(self.cols - 1),
            '\u{7}' => self.bell = true,
            c if c.is_control() => {}
            c => self.put(c),
        }
    }

    fn put(&mut self, c: char) {
        let w = cell_width(c);
        let row = self.top() + self.cursor.1;
        if w == 0 {
            let at = self.cursor.0.saturating_sub(1).min(self.lines[row].len() - 1);
            let at = if at > 0 && self.lines[row][at].spacer { at - 1 } else { at };
            self.lines[row][at].text.push(c);
            return;
        }
        if self.cursor.0 + w > self.cols {
            self.cursor.0 = 0;
            self.line_feed();
        }
        let row = self.top() + self.cursor.1;
        let col = self.cursor.0;
        self.lines[row][col] = Cell { text: c.to_string(), spacer: false, ..self.look.clone() };
        if w == 2 && col + 1 < self.cols {
            self.lines[row][col + 1] = Cell { text: String::new(), spacer: true, ..self.look.clone() };
        }
        self.cursor.0 += w;
    }

    fn top(&self) -> usize {
        self.lines.len() - self.rows
    }

    fn line_feed(&mut self) {
        if self.cursor.1 + 1 < self.rows {
            self.cursor.1 += 1;
            return;
        }
        self.lines.push_back(vec![Cell::default(); self.cols]);
        while self.scrolled_off() > self.most {
            self.lines.pop_front();
        }
    }

    fn control(&mut self, c: char, nums: &[u16], private: bool) {
        let n = |i: usize| nums.get(i).copied().filter(|&n| n > 0).unwrap_or(1) as usize;
        let (col, row) = self.cursor;
        let blank = Cell { bg: self.look.bg, ..Cell::default() };
        match c {
            'A' => self.cursor.1 = row.saturating_sub(n(0)),
            'B' => self.cursor.1 = (row + n(0)).min(self.rows - 1),
            'C' => self.cursor.0 = (col + n(0)).min(self.cols - 1),
            'D' => self.cursor.0 = col.saturating_sub(n(0)),
            'G' => self.cursor.0 = (n(0) - 1).min(self.cols - 1),
            'd' => self.cursor.1 = (n(0) - 1).min(self.rows - 1),
            'H' | 'f' => self.cursor = ((n(1) - 1).min(self.cols - 1), (n(0) - 1).min(self.rows - 1)),
            'J' | 'K' => {
                let top = self.top();
                let (from, to) = match (c, nums.first().copied().unwrap_or(0)) {
                    ('J', 0) => ((col, row), (self.cols, self.rows - 1)),
                    ('J', 1) => ((0, 0), (col + 1, row)),
                    ('J', _) => ((0, 0), (self.cols, self.rows - 1)),
                    (_, 0) => ((col, row), (self.cols, row)),
                    (_, 1) => ((0, row), (col + 1, row)),
                    _ => ((0, row), (self.cols, row)),
                };
                for r in from.1..=to.1 {
                    let (a, b) = (if r == from.1 { from.0 } else { 0 }, if r == to.1 { to.0 } else { self.cols });
                    self.lines[top + r][a.min(self.cols)..b.min(self.cols)].fill(blank.clone());
                }
            }
            'm' if !private => self.colours(nums),
            'h' | 'l' if private && nums.contains(&25) => self.cursor_shown = c == 'h',
            _ => {}
        }
    }

    fn colours(&mut self, nums: &[u16]) {
        let term = theme::sheet().term;
        let mut i = 0;
        if nums.is_empty() {
            self.look = Cell::default();
        }
        while i < nums.len() {
            let l = &mut self.look;
            match nums[i] {
                0 => *l = Cell::default(),
                1 => l.bold = true,
                3 => l.italic = true,
                4 => l.underline = true,
                7 => l.inverse = true,
                9 => l.strike = true,
                22 => l.bold = false,
                23 => l.italic = false,
                24 => l.underline = false,
                27 => l.inverse = false,
                29 => l.strike = false,
                n @ 30..=37 => l.fg = Some(term[n as usize - 30]),
                n @ 90..=97 => l.fg = Some(term[n as usize - 82]),
                n @ 40..=47 => l.bg = Some(term[n as usize - 40]),
                n @ 100..=107 => l.bg = Some(term[n as usize - 92]),
                39 => l.fg = None,
                49 => l.bg = None,
                n @ (38 | 48) => {
                    let (colour, used) = match nums.get(i + 1) {
                        Some(5) => (nums.get(i + 2).map(|&k| indexed(k, &term)), 2),
                        Some(2) if nums.len() > i + 4 => {
                            let v = |j: usize| nums[i + j].min(255) as f32 / 255.0;
                            (Some(Colour::rgb(v(2), v(3), v(4))), 4)
                        }
                        _ => (None, 0),
                    };
                    if n == 38 { l.fg = colour } else { l.bg = colour }
                    i += used;
                }
                _ => {}
            }
            i += 1;
        }
    }

    pub fn scroll(&mut self, lines: f32) {
        self.back = (self.back as f32 - lines).round().clamp(0.0, self.scrolled_off() as f32) as usize;
    }

    pub fn resize(&mut self, cols: usize, rows: usize) {
        let (cols, rows) = (cols.max(1), rows.max(1));
        let line = self.top() + self.cursor.1;
        for l in &mut self.lines {
            l.resize(cols, Cell::default());
        }
        let gone = self.rows.saturating_sub(self.cursor.1 + 1).min(self.rows.saturating_sub(rows));
        self.lines.truncate(self.lines.len() - gone);
        while self.lines.len() < rows {
            self.lines.push_back(vec![Cell::default(); cols]);
        }
        (self.cols, self.rows, self.back) = (cols, rows, 0);
        self.cursor = (self.cursor.0.min(cols - 1), line.saturating_sub(self.top()).min(rows - 1));
    }

    pub fn lines_shown(&self) -> Vec<String> {
        let first = self.top() - self.back;
        (first..first + self.rows).map(|i| {
            let s: String = self.lines[i].iter().filter(|c| !c.spacer).map(|c| if c.text.is_empty() { " " } else { &c.text }).collect();
            s.trim_end().to_string()
        }).collect()
    }

    pub fn screen(&self, ground: Colour, words: Colour) -> Screen {
        let mut screen = Screen::new(self.cols, self.rows);
        let first = self.top() - self.back;
        for row in 0..self.rows {
            for (col, cell) in self.lines[first + row].iter().enumerate() {
                let fg = cell.fg.unwrap_or(words);
                let (ink, under) = if cell.inverse { (cell.bg.unwrap_or(ground), Some(fg)) } else { (fg, cell.bg) };
                let fg = Some(clear(ink, &[under.unwrap_or(ground)], READABLE));
                *screen.cell_mut(col, row) = Cell { fg, bg: under, inverse: false, ..cell.clone() };
            }
        }
        screen
    }

    pub fn caret(&self, focused: bool) -> Option<Caret> {
        (self.cursor_shown && self.back == 0).then_some(Caret { col: self.cursor.0, row: self.cursor.1, shape: Shape::Block, focused })
    }

    pub fn reading(&self) -> Reading {
        let caret = self.caret(true).map(|c| (c.row, c.col));
        Reading { lines: self.lines_shown(), caret, selection: None, editable: false }
    }
}

fn indexed(k: u16, term: &[Colour; 16]) -> Colour {
    match k {
        0..=15 => term[k as usize],
        16..=231 => {
            let k = k - 16;
            let level = |v: u16| if v == 0 { 0.0 } else { (55.0 + 40.0 * v as f32) / 255.0 };
            Colour::rgb(level(k / 36), level(k / 6 % 6), level(k % 6))
        }
        _ => {
            let v = (8.0 + 10.0 * (k.min(255) - 232) as f32) / 255.0;
            Colour::rgb(v, v, v)
        }
    }
}

pub fn paint_term(canvas: &mut PixmapMut, pen: &Pen, term: &Term, size: f32, focused: bool) {
    let mut p = Painter::default();
    paint_term_on(&mut p, pen, term, size, focused);
    p.play(canvas);
}

pub fn paint_term_on<'a>(p: &mut Painter<'a>, pen: &Pen<'a>, term: &Term, size: f32, focused: bool) {
    let screen = term.screen(pen.theme.bg, pen.theme.fg);
    paint_cells_on(p, pen, &screen, size, term.caret(focused), None);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fed(cols: usize, rows: usize, s: &str) -> Term {
        let mut t = Term::new(cols, rows);
        t.feed(s.as_bytes());
        t
    }

    #[test]
    fn words_and_line_ends_land_where_a_terminal_puts_them() {
        let t = fed(10, 3, "ab\r\ncd\tx\r\nlong line here");
        assert_eq!(t.lines_shown(), ["cd      x", "long line", "here"], "a full line wraps and the first scrolls off");
        assert_eq!(t.scrolled_off(), 1);
        assert_eq!(t.cursor, (4, 2));
    }

    #[test]
    fn colours_and_marks_come_from_the_sheet_and_reset() {
        let term = theme::sheet().term;
        let t = fed(20, 2, "\x1b[1;31mred\x1b[0m \x1b[92;44mgo\x1b[38;2;10;20;30mx\x1b[38;5;196my");
        let line = &t.lines[0];
        assert!(line[0].bold && line[0].fg == Some(term[1]));
        assert_eq!((line[3].fg, line[3].bold), (None, false));
        assert_eq!((line[4].fg, line[4].bg), (Some(term[10]), Some(term[4])));
        assert_eq!(line[6].fg, Some(Colour::rgb(10.0 / 255.0, 20.0 / 255.0, 30.0 / 255.0)));
        assert_eq!(line[7].fg, Some(Colour::rgb(1.0, 0.0, 0.0)));
    }

    #[test]
    fn the_cursor_moves_and_erasing_clears() {
        let mut t = fed(10, 3, "hello\r\nworld");
        t.feed(b"\x1b[1;3H\x1b[K");
        assert_eq!(t.lines_shown()[0], "he");
        t.feed(b"\x1b[2J\x1b[2;2Hx\x1b[A\x1b[2Cy\x1b[?25l");
        assert_eq!(t.lines_shown(), ["    y", " x", ""]);
        assert!(!t.cursor_shown && t.caret(true).is_none());
        t.feed(b"\x1b]0;a title\x07z\x1b(B\x07");
        assert_eq!(t.lines_shown()[0], "    yz", "a title and a character set are passed over");
        assert!(t.bell);
    }

    #[test]
    fn an_endless_operating_system_command_is_given_up() {
        let t = fed(10, 2, &format!("\x1b]{}ok", "x".repeat(MOST_OSC + 1)));
        assert_eq!(t.lines_shown()[0], "ok");
    }

    #[test]
    fn a_character_split_between_feeds_and_a_wide_one_are_whole() {
        let mut t = Term::new(10, 2);
        let bytes = "é漢x".as_bytes();
        t.feed(&bytes[..1]);
        t.feed(&bytes[1..]);
        assert_eq!(t.lines_shown()[0], "é漢x");
        assert!(t.lines[0][2].spacer);
        t.feed(&[0xff, b'!']);
        assert_eq!(t.lines_shown()[0], "é漢x\u{fffd}!");
    }

    #[test]
    fn the_scroll_back_is_kept_to_its_most_and_scrolled_through() {
        let mut t = Term::new(10, 2);
        t.most = 5;
        for i in 0..20 {
            t.feed(format!("{i}\r\n").as_bytes());
        }
        assert_eq!(t.scrolled_off(), 5);
        t.scroll(-3.0);
        assert_eq!((t.back, t.lines_shown()[0].as_str()), (3, "16"));
        assert!(t.caret(true).is_none(), "no cursor in the scroll-back");
        t.scroll(-100.0);
        assert_eq!(t.back, 5);
        t.feed(b"x");
        assert_eq!(t.back, 0, "output brings the live screen back");
    }

    #[test]
    fn resizing_keeps_the_cursor_on_its_line() {
        let mut t = fed(10, 4, "a\r\nb\r\nc");
        t.resize(5, 2);
        assert_eq!(t.lines_shown(), ["b", "c"]);
        assert_eq!(t.cursor, (1, 1));
        t.resize(5, 4);
        assert_eq!(t.lines_shown(), ["a", "b", "c", ""], "the line kept above comes back");
        assert_eq!(t.cursor, (1, 2));
    }

    #[test]
    fn every_colour_drawn_can_be_read_on_its_cell() {
        let t = fed(40, 1, "\x1b[30mA\x1b[90mB\x1b[34;44mC\x1b[7mD\x1b[0;38;5;232mE");
        for (ground, words) in [(Colour::rgb(0.06, 0.08, 0.12), Colour::rgb(0.9, 0.9, 0.9)), (Colour::rgb(0.97, 0.97, 0.97), Colour::rgb(0.1, 0.1, 0.1))] {
            let s = t.screen(ground, words);
            for col in 0..5 {
                let c = s.cell(col, 0);
                let (fg, bg) = (c.fg.unwrap_or(words), c.bg.unwrap_or(ground));
                assert!(theme::contrast(fg, bg) >= READABLE - 0.01, "cell {col}: {}", theme::contrast(fg, bg));
            }
        }
    }

    #[test]
    fn a_reader_hears_the_lines_and_the_cursor_and_the_screen_is_painted() {
        let t = fed(20, 2, "$ ls\r\nnotes.txt");
        let r = t.reading();
        assert_eq!((r.lines, r.caret, r.editable), (vec!["$ ls".to_string(), "notes.txt".into()], Some((1, 9)), false));
        let th = theme::Theme::new(false, "#fbc711");
        let (fonts, icons) = (super::super::text::Fonts::new(), super::super::icon::Icons::new());
        let pen = Pen { theme: &th, fonts: &fonts, icons: &icons, scale: 1.0 };
        let mut data = vec![0u8; 300 * 60 * 4];
        paint_term(&mut PixmapMut::from_bytes(&mut data, 300, 60).unwrap(), &pen, &t, 14.0, true);
        assert!(data.chunks(4).any(|p| p[3] > 0));
    }
}
