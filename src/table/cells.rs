//! Typing in one cell of a table: the text, its caret and the keys that edit it.

use super::super::draw::{fill_on, words_on};
use super::super::paint::Painter;
use super::super::edit::Edit;
use super::super::input::{Keysym, Mods};
use super::super::text::Style;
use super::super::widget::{Pen, Rect};
use super::Table;

#[derive(Clone, Debug)]
pub struct CellEdit {
    pub row: usize,
    pub column: usize,
    edit: Edit,
}

impl PartialEq for CellEdit {
    fn eq(&self, other: &Self) -> bool {
        (self.row, self.column, &self.edit.text, self.edit.caret, self.edit.anchor)
            == (other.row, other.column, &other.edit.text, other.edit.caret, other.edit.anchor)
    }
}

impl CellEdit {
    pub fn text(&self) -> &str {
        &self.edit.text
    }

    pub fn caret(&self) -> usize {
        self.edit.caret
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Edited {
    Typing,
    Kept { row: usize, column: usize, text: String },
    Dropped,
    Copy(String),
    Paste,
}

impl Table {
    pub fn begin_edit(&mut self, row: usize, column: usize, words: &str) -> bool {
        if row >= self.rows || !self.columns.get(column).is_some_and(|c| c.editable) {
            return false;
        }
        let mut edit = Edit::new(words);
        edit.anchor = 0;
        self.editing = Some(CellEdit { row, column, edit });
        self.chosen = Some(row);
        true
    }

    pub fn end_edit(&mut self) -> Option<(usize, usize, String)> {
        self.editing.take().map(|mut e| (e.row, e.column, std::mem::take(&mut e.edit.text)))
    }

    pub fn paste(&mut self, words: &str) -> bool {
        let line = crate::ui::outside::one_line(words);
        self.editing.as_mut().is_some_and(|c| c.edit.insert(&line))
    }

    pub fn first_editable(&self) -> Option<usize> {
        self.columns.iter().position(|c| c.editable)
    }

    pub fn edit_column(&self) -> Option<usize> {
        self.column.filter(|&c| self.columns.get(c).is_some_and(|c| c.editable)).or_else(|| self.first_editable())
    }

    pub fn edit_key(&mut self, sym: Keysym, text: Option<&str>, mods: Mods) -> Option<Edited> {
        let e = &mut self.editing.as_mut()?.edit;
        let jump = mods.ctrl;
        match sym {
            Keysym::Escape => {
                self.editing = None;
                return Some(Edited::Dropped);
            }
            Keysym::Return | Keysym::KP_Enter | Keysym::Tab => {
                let (row, column, text) = self.end_edit()?;
                return Some(Edited::Kept { row, column, text });
            }
            Keysym::BackSpace => {
                let to = if jump { e.prev_word(e.caret) } else { e.prev(e.caret) };
                e.delete_to(to);
            }
            Keysym::Delete => {
                let to = if jump { e.next_word(e.caret) } else { e.next(e.caret) };
                e.delete_to(to);
            }
            Keysym::Left => {
                let to = if jump { e.prev_word(e.caret) } else { e.prev(e.caret) };
                e.move_to(to, mods.shift);
            }
            Keysym::Right => {
                let to = if jump { e.next_word(e.caret) } else { e.next(e.caret) };
                e.move_to(to, mods.shift);
            }
            Keysym::Home => e.move_to(0, mods.shift),
            Keysym::End => e.move_to(e.text.len(), mods.shift),
            Keysym::a | Keysym::A if mods.ctrl => e.select(0..e.text.len()),
            Keysym::c | Keysym::C | Keysym::Insert | Keysym::KP_Insert if mods.ctrl => {
                return Some(match e.selected() {
                    "" => Edited::Typing,
                    words => Edited::Copy(words.to_string()),
                });
            }
            Keysym::x | Keysym::X if mods.ctrl => return Some(cut_selected(e)),
            Keysym::v | Keysym::V if mods.ctrl => return Some(Edited::Paste),
            Keysym::Insert | Keysym::KP_Insert if mods.shift => return Some(Edited::Paste),
            Keysym::z | Keysym::Z if mods.ctrl && !mods.shift => {
                e.undo();
            }
            Keysym::z | Keysym::Z | Keysym::y | Keysym::Y if mods.ctrl => {
                e.redo();
            }
            _ => {
                if let Some(t) = text.filter(|t| !mods.ctrl && !mods.alt && !t.chars().any(char::is_control)) {
                    e.insert(t);
                }
            }
        }
        Some(Edited::Typing)
    }
}

pub(super) fn paint_edit<'a>(p: &mut Painter<'a>, pen: &Pen<'a>, e: &CellEdit, cell: Rect, room: Rect, style: &Style) {
    let (t, k) = (pen.theme, pen.scale);
    let boxed = Rect { x: cell.x + 2.0, y: cell.y + 2.0, w: (cell.w - 4.0).max(0.0), h: (cell.h - 4.0).max(0.0) };
    fill_on(p, pen, boxed, t.field_edge);
    fill_on(p, pen, Rect { x: boxed.x + 1.0, y: boxed.y + 1.0, w: (boxed.w - 2.0).max(0.0), h: (boxed.h - 2.0).max(0.0) }, t.field);
    let text = &e.edit.text;
    let width = |s: &str| pen.fonts.layout(s, style, None).width / k;
    let mut start = 0;
    while start < e.edit.caret && width(&text[start..e.edit.caret]) > room.w {
        start = e.edit.next(start);
    }
    let shown = &text[start..];
    let mut end = shown.len();
    while end > 0 && width(&shown[..end]) > room.w {
        end = e.edit.prev(start + end) - start;
    }
    let shown = &shown[..end];
    let at = |i: usize| room.x + width(&text[start..i.clamp(start, start + end)]);
    let sel = e.edit.selection();
    if !sel.is_empty() {
        let (a, b) = (at(sel.start), at(sel.end));
        fill_on(p, pen, Rect { x: a, y: boxed.y + 3.0, w: b - a, h: (boxed.h - 6.0).max(0.0) }, t.picked);
    }
    let line = pen.fonts.layout(shown, style, None);
    let y = (cell.y * k + (cell.h * k - line.height) / 2.0).round();
    words_on(p, pen.fonts, line, (room.x * k).round(), y, t.fg);
    fill_on(p, pen, Rect { x: at(e.edit.caret), y: boxed.y + 4.0, w: 1.5, h: (boxed.h - 8.0).max(0.0) }, t.fg);
}

fn cut_selected(e: &mut Edit) -> Edited {
    let words = e.selected().to_string();
    if words.is_empty() {
        return Edited::Typing;
    }
    e.delete_to(e.caret);
    Edited::Copy(words)
}
