//! A rich text box: words typed and changed in place in bold, italic and code, kept as spans and saved as `rich` markup.

use std::cell::RefCell;
use std::ops::Range;
use std::rc::Rc;

use super::editor::{rect, Did, LINE};
use crate::input::{Input, Keysym};
use crate::rich::{self, Flow, Span};
use crate::text::Style;
use crate::theme;
use crate::widget::{drawing, Length, Node, Pointer, Reading};

const PAD: f32 = 8.0;
const MOST_UNDO: usize = 100;

const BOLD: u8 = 1;
const ITALIC: u8 = 2;
const CODE: u8 = 4;

#[derive(Clone, Debug)]
pub enum Event {
    Key(Input),
    Pointer(Pointer),
    Paste(String),
}

struct State {
    text: String,
    looks: Vec<u8>,
    caret: usize,
    anchor: usize,
    typing: u8,
    undo: Vec<(String, Vec<u8>, usize)>,
    redo: Vec<(String, Vec<u8>, usize)>,
    flow: Option<Flow>,
    scale: f32,
    dragging: bool,
    joining: Option<usize>,
}

#[derive(Clone)]
pub struct RichBox {
    state: Rc<RefCell<State>>,
}

impl RichBox {
    pub fn new(markup: &str) -> RichBox {
        let (mut text, mut looks) = (String::new(), Vec::new());
        for s in rich::parse(markup) {
            let look = (u8::from(s.bold) * BOLD) | (u8::from(s.italic) * ITALIC) | (u8::from(s.code) * CODE);
            looks.extend(std::iter::repeat_n(look, s.text.len()));
            text.push_str(&s.text);
        }
        let (end, typing) = (text.len(), looks.last().copied().unwrap_or(0));
        RichBox { state: Rc::new(RefCell::new(State {
            text, looks, caret: end, anchor: end, typing, undo: Vec::new(), redo: Vec::new(), flow: None, scale: 1.0, dragging: false, joining: None,
        })) }
    }

    pub fn text(&self) -> String {
        self.state.borrow().text.clone()
    }

    pub fn spans(&self) -> Vec<Span> {
        self.state.borrow().spans()
    }

    pub fn markup(&self) -> String {
        rich::markup(&self.spans())
    }

    pub fn selection(&self) -> Range<usize> {
        let s = self.state.borrow();
        s.caret.min(s.anchor)..s.caret.max(s.anchor)
    }

    pub fn select(&self, anchor: usize, caret: usize) {
        let mut s = self.state.borrow_mut();
        let len = s.text.len();
        s.anchor = snap(&s.text, anchor.min(len));
        let caret = snap(&s.text, caret.min(len));
        s.go(caret, true);
    }

    pub fn node<M: 'static>(&self, name: &str, least: f32, send: impl Fn(Event) -> M + Clone + 'static) -> Node<M> {
        let paint = self.state.clone();
        let (keys, point) = (send.clone(), send);
        drawing((0.0, least), name, move |canvas, pen| paint.borrow_mut().paint(canvas, pen))
            .width(Length::Fill)
            .height(Length::Fill)
            .reads(self.reading())
            .on_pointer(move |p| point(Event::Pointer(p)))
            .on_keys(move |input| takes(input).then(|| keys(Event::Key(input.clone()))))
    }

    pub fn reading(&self) -> Reading {
        let s = self.state.borrow();
        let at = |b: usize| {
            let before = &s.text[..b];
            let line = before.matches('\n').count();
            (line, before.rsplit('\n').next().unwrap_or("").chars().count())
        };
        Reading {
            lines: s.text.split('\n').map(String::from).collect(),
            caret: Some(at(s.caret)),
            selection: (s.caret != s.anchor).then(|| (at(s.anchor), at(s.caret))),
            editable: true,
        }
    }

    pub fn apply(&self, event: Event) -> Did {
        let mut s = self.state.borrow_mut();
        match event {
            Event::Key(input) => s.key(&input),
            Event::Pointer(p) => s.pointer(p),
            Event::Paste(words) => {
                s.put(&words.replace("\r\n", "\n").replace('\r', "\n"), false);
                Did::Edited
            }
        }
    }
}

fn takes(input: &Input) -> bool {
    let Input::Key { sym, text, mods, .. } = input else { return false };
    if mods.ctrl {
        return matches!(*sym, Keysym::b | Keysym::B | Keysym::i | Keysym::I | Keysym::e | Keysym::E | Keysym::a | Keysym::A | Keysym::z | Keysym::Z
            | Keysym::y | Keysym::Y | Keysym::c | Keysym::C | Keysym::x | Keysym::X | Keysym::v | Keysym::V);
    }
    matches!(*sym, Keysym::Left | Keysym::Right | Keysym::Up | Keysym::Down | Keysym::Home | Keysym::End | Keysym::BackSpace | Keysym::Delete
        | Keysym::Return | Keysym::KP_Enter)
        || (!mods.alt && !mods.logo && text.as_deref().is_some_and(|t| !t.is_empty() && !t.chars().any(char::is_control)))
}

fn snap(text: &str, mut at: usize) -> usize {
    while !text.is_char_boundary(at) {
        at -= 1;
    }
    at
}

impl State {
    fn spans(&self) -> Vec<Span> {
        let mut out: Vec<(u8, String)> = Vec::new();
        for (i, c) in self.text.char_indices() {
            let look = self.looks[i];
            match out.last_mut() {
                Some((l, s)) if *l == look => s.push(c),
                _ => out.push((look, c.to_string())),
            }
        }
        out.into_iter().map(|(l, text)| Span { text, bold: l & BOLD != 0, italic: l & ITALIC != 0, code: l & CODE != 0, link: None }).collect()
    }

    fn selection(&self) -> Range<usize> {
        self.caret.min(self.anchor)..self.caret.max(self.anchor)
    }

    fn keep(&mut self) {
        self.undo.push((self.text.clone(), self.looks.clone(), self.caret));
        if self.undo.len() > MOST_UNDO {
            self.undo.remove(0);
        }
        self.redo.clear();
        self.joining = None;
    }

    fn put(&mut self, words: &str, typed: bool) {
        let r = self.selection();
        if !(typed && r.is_empty() && self.joining == Some(r.start)) {
            self.keep();
        }
        let words: String = words.chars().filter(|c| *c == '\n' || *c == '\t' || !c.is_control()).collect();
        self.text.replace_range(r.clone(), &words);
        self.looks.splice(r.clone(), std::iter::repeat_n(self.typing, words.len()));
        self.caret = r.start + words.len();
        self.anchor = self.caret;
        self.joining = (typed && !words.ends_with(char::is_whitespace)).then_some(self.caret);
    }

    fn delete(&mut self, r: Range<usize>) -> Did {
        if r.is_empty() {
            return Did::Nothing;
        }
        self.keep();
        self.text.replace_range(r.clone(), "");
        self.looks.drain(r.clone());
        (self.caret, self.anchor) = (r.start, r.start);
        Did::Edited
    }

    fn toggle(&mut self, bit: u8) -> Did {
        let r = self.selection();
        if r.is_empty() {
            self.typing ^= bit;
            self.joining = None;
            return Did::Moved;
        }
        self.keep();
        let on = self.looks[r.clone()].iter().all(|l| l & bit != 0);
        for l in &mut self.looks[r] {
            *l = if on { *l & !bit } else { *l | bit };
        }
        Did::Edited
    }

    fn prev(&self, at: usize) -> usize {
        self.text[..at].chars().next_back().map_or(0, |c| at - c.len_utf8())
    }

    fn next(&self, at: usize) -> usize {
        self.text[at..].chars().next().map_or(at, |c| at + c.len_utf8())
    }

    fn go(&mut self, to: usize, extend: bool) -> Did {
        (self.caret, self.joining) = (to, None);
        if !extend {
            self.anchor = to;
        }
        self.typing = if to == 0 { self.looks.first().copied().unwrap_or(0) } else { self.looks[self.prev(to)] };
        Did::Moved
    }

    fn key(&mut self, input: &Input) -> Did {
        let Input::Key { sym, text, mods, .. } = input else { return Did::Nothing };
        let (shift, at) = (mods.shift, self.caret);
        let line_start = self.text[..at].rfind('\n').map_or(0, |i| i + 1);
        let line_end = self.text[at..].find('\n').map_or(self.text.len(), |i| at + i);
        if mods.ctrl {
            return match *sym {
                Keysym::b | Keysym::B => self.toggle(BOLD),
                Keysym::i | Keysym::I => self.toggle(ITALIC),
                Keysym::e | Keysym::E => self.toggle(CODE),
                Keysym::a | Keysym::A => {
                    (self.anchor, self.caret) = (0, self.text.len());
                    Did::Moved
                }
                Keysym::z | Keysym::Z if !shift => self.step(false),
                Keysym::z | Keysym::Z | Keysym::y | Keysym::Y => self.step(true),
                Keysym::c | Keysym::C => Did::Copy(self.text[self.selection()].to_string()),
                Keysym::x | Keysym::X => {
                    let words = self.text[self.selection()].to_string();
                    self.delete(self.selection());
                    Did::Copy(words)
                }
                Keysym::v | Keysym::V => Did::Paste,
                _ => Did::Nothing,
            };
        }
        match crate::edit::facing(*sym, &self.text[line_start..line_end]) {
            Keysym::Left if !shift && at != self.anchor => self.go(self.selection().start, false),
            Keysym::Right if !shift && at != self.anchor => self.go(self.selection().end, false),
            Keysym::Left => self.go(self.prev(at), shift),
            Keysym::Right => self.go(self.next(at), shift),
            Keysym::Home => self.go(line_start, shift),
            Keysym::End => self.go(line_end, shift),
            Keysym::Up | Keysym::Down => {
                let to = self.flow.as_ref().map_or(at, |f| {
                    let (x, y, h) = f.caret(at);
                    let y = if *sym == Keysym::Down { y + h * 1.5 } else { y - h * 0.5 };
                    if y < 0.0 { 0 } else if y > f.height { self.text.len() } else { f.hit(x, y) }
                });
                self.go(to, shift)
            }
            Keysym::BackSpace if at == self.anchor => self.delete(self.prev(at)..at),
            Keysym::Delete if at == self.anchor => self.delete(at..self.next(at)),
            Keysym::BackSpace | Keysym::Delete => self.delete(self.selection()),
            Keysym::Return | Keysym::KP_Enter => {
                self.put("\n", true);
                Did::Edited
            }
            _ => match text {
                Some(t) if !t.is_empty() => {
                    self.put(t, true);
                    Did::Edited
                }
                _ => Did::Nothing,
            },
        }
    }

    fn step(&mut self, forward: bool) -> Did {
        let (from, to) = if forward { (&mut self.redo, &mut self.undo) } else { (&mut self.undo, &mut self.redo) };
        let Some((text, looks, caret)) = from.pop() else { return Did::Nothing };
        self.joining = None;
        to.push((std::mem::replace(&mut self.text, text), std::mem::replace(&mut self.looks, looks), self.caret));
        (self.caret, self.anchor) = (caret.min(self.text.len()), caret.min(self.text.len()));
        Did::Edited
    }

    fn pointer(&mut self, p: Pointer) -> Did {
        let hit = |s: &State, x: f32, y: f32| {
            let pad = PAD * s.scale;
            s.flow.as_ref().map_or(s.caret, |f| f.hit(x * s.scale - pad, y * s.scale - pad))
        };
        match p {
            Pointer::Down(x, y) => {
                self.dragging = true;
                let to = hit(self, x, y);
                self.go(to, false)
            }
            Pointer::Move(x, y) if self.dragging => {
                let to = hit(self, x, y);
                self.go(to, true)
            }
            Pointer::Up(..) => {
                self.dragging = false;
                Did::Nothing
            }
            _ => Did::Nothing,
        }
    }

    fn paint(&mut self, canvas: &mut tiny_skia::PixmapMut, pen: &crate::widget::Pen) {
        let (s, t) = (pen.scale, pen.theme);
        self.scale = s;
        let (w, h) = (canvas.width() as f32, canvas.height() as f32);
        let pad = PAD * s;
        rect(canvas, 0.0, 0.0, w, h, t.field);
        let base = Style::new(theme::sheet().text.0 * s);
        let spans = self.spans();
        let flow = rich::flow(&|text, st| pen.fonts.layout(text, st, None), &spans, &base, Some((w - 2.0 * pad).max(1.0)));
        let field = theme::over(t.field, theme::over(t.panel, t.bg));
        let sel = self.selection();
        if !sel.is_empty() {
            let wash = t.wash(theme::alpha(t.accent, theme::sheet().selection), &[field]);
            for p in &flow.pieces {
                let local = sel.start.saturating_sub(p.at)..sel.end.saturating_sub(p.at).min(p.len);
                for (x, y, bw, bh) in p.layout.mark_bands(&[local]) {
                    rect(canvas, pad + p.x + x, pad + p.y + y, bw, bh, wash);
                }
            }
        }
        flow.draw(pen.fonts, canvas, pad, pad, &|_| t.words_on(&[field]));
        let (x, y, ch) = flow.caret(self.caret);
        let line = if ch > 0.0 { ch } else { base.size * LINE };
        rect(canvas, (pad + x).round(), pad + y, theme::sheet().caret * s, line, t.accent);
        self.flow = Some(flow);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::input::Mods;

    fn key(sym: Keysym, text: Option<&str>, ctrl: bool, shift: bool) -> Event {
        Event::Key(Input::Key { sym, text: text.map(String::from), mods: Mods { ctrl, shift, ..Mods::default() }, repeat: false })
    }

    fn typed(b: &RichBox, words: &str) {
        for c in words.chars() {
            b.apply(key(Keysym::a, Some(&c.to_string()), false, false));
        }
    }

    #[test]
    fn markup_is_read_in_and_written_back() {
        let b = RichBox::new("plain **bold** *lean* `code`");
        assert_eq!(b.text(), "plain bold lean code");
        assert_eq!(b.markup(), "plain **bold** *lean* `code`");
    }

    #[test]
    fn a_look_is_turned_on_for_the_selection_or_for_what_is_typed_next() {
        let b = RichBox::new("one two");
        b.select(4, 7);
        assert_eq!(b.apply(key(Keysym::b, None, true, false)), Did::Edited);
        assert_eq!(b.markup(), "one **two**");
        b.apply(key(Keysym::b, None, true, false));
        assert_eq!(b.markup(), "one two", "all bold already: turned off");
        b.select(7, 7);
        b.apply(key(Keysym::i, None, true, false));
        typed(&b, " three");
        b.apply(key(Keysym::i, None, true, false));
        typed(&b, "!");
        assert_eq!(b.markup(), "one two* three*!");
        b.apply(key(Keysym::z, None, true, false));
        assert_eq!(b.text(), "one two three");
    }

    #[test]
    fn the_chords_work_with_caps_lock_on_and_shift_z_redoes() {
        let b = RichBox::new("one two");
        b.select(4, 7);
        assert_eq!(b.apply(key(Keysym::B, None, true, false)), Did::Edited);
        assert_eq!(b.markup(), "one **two**");
        let t = RichBox::new("");
        typed(&t, "ab");
        assert_eq!(t.apply(key(Keysym::Z, None, true, false)), Did::Edited, "Ctrl+Z with Caps Lock undoes");
        assert_eq!(t.text(), "");
        assert_eq!(t.apply(key(Keysym::Z, None, true, true)), Did::Edited, "Ctrl+Shift+Z redoes");
        assert_eq!(t.text(), "ab");
        assert_eq!(b.apply(key(Keysym::C, None, true, false)), Did::Copy("two".into()));
    }

    #[test]
    fn a_typed_word_undoes_as_one_step() {
        let b = RichBox::new("");
        typed(&b, "one two");
        b.apply(key(Keysym::z, None, true, false));
        assert_eq!(b.text(), "one ");
        b.apply(key(Keysym::z, None, true, false));
        assert_eq!(b.text(), "");
    }

    #[test]
    fn typing_carries_the_look_of_the_letter_before_and_deleting_takes_its_look_too() {
        let b = RichBox::new("**ab**cd");
        b.select(1, 1);
        typed(&b, "x");
        assert_eq!(b.markup(), "**axb**cd");
        b.apply(key(Keysym::End, None, false, false));
        b.apply(key(Keysym::BackSpace, None, false, false));
        b.apply(key(Keysym::Left, None, false, true));
        assert_eq!(b.selection(), 3..4);
        b.apply(key(Keysym::Delete, None, false, false));
        assert_eq!(b.markup(), "**axb**");
        assert_eq!(b.apply(key(Keysym::c, None, true, false)), Did::Copy(String::new()));
    }

    #[test]
    fn a_reader_is_told_the_lines_and_the_caret() {
        let b = RichBox::new("first\n**second**");
        b.select(8, 8);
        let r = b.reading();
        assert_eq!((r.lines, r.caret, r.editable), (vec!["first".to_string(), "second".to_string()], Some((1, 2)), true));
    }
}
