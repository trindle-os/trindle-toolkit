//! A ready text area for a window: many lines, wrapping, undo, a scroll bar, the clipboard and a menu.

use std::cell::RefCell;
use std::ops::Range;
use std::rc::Rc;

use super::doc::Doc;
use super::editor::{words, Checker, Did, Look, Pane, LINE};
use crate::input::{Cursor, Input, Keysym, Mods};
use crate::spelling::Speller;
use crate::widget::{drawing, Length, MenuRow, Node, Pointer, Reading};
use crate::window::Window;

#[derive(Clone, Debug)]
pub enum Event {
    Key(Input),
    Pointer(Pointer),
    Scroll(f32),
    Paste(String),
}

#[derive(Clone)]
pub struct TextArea {
    pane: Rc<RefCell<Pane>>,
}

const READING_MARGIN: usize = 50;

impl TextArea {
    pub fn new(text: &str, look: Look) -> TextArea {
        TextArea { pane: Rc::new(RefCell::new(Pane::new(Doc::new(text.to_string()), look))) }
    }

    pub fn pane(&self) -> Rc<RefCell<Pane>> {
        self.pane.clone()
    }

    pub fn spelling(&self, speller: Option<Rc<dyn Speller>>) {
        let mut pane = self.pane.borrow_mut();
        pane.checker = speller.map(|s| Rc::new(move |word: &str| s.known(word)) as Checker);
        pane.checked += 1;
    }

    fn told_lines(pane: &Pane) -> Range<usize> {
        let doc = &pane.doc;
        let first = pane.first_line().min(doc.lines().saturating_sub(1));
        first..(first + pane.shown_lines() + READING_MARGIN).min(doc.lines())
    }

    fn misspelt(&self) -> bool {
        let pane = self.pane.borrow();
        let Some(check) = &pane.checker else { return false };
        TextArea::told_lines(&pane).any(|i| {
            let line = pane.doc.line(i);
            words(line).any(|r| !check(&line[r]))
        })
    }

    pub fn text(&self) -> String {
        self.pane.borrow().doc.text().to_string()
    }

    pub fn set_text(&self, text: &str) {
        let mut pane = self.pane.borrow_mut();
        pane.doc = Doc::new(text.to_string());
        pane.reset();
    }

    pub fn reading(&self) -> Reading {
        let pane = self.pane.borrow();
        let doc = &pane.doc;
        let Range { start: first, end: last } = TextArea::told_lines(&pane);
        let lines: Vec<String> = (first..last).map(|i| doc.line(i).trim_end_matches('\r').to_string()).collect();
        let at = |byte: usize| {
            let line = doc.line_of(byte).clamp(first, last.saturating_sub(1).max(first));
            let range = doc.line_range(line);
            (line - first, doc.text()[range.start..byte.clamp(range.start, range.end)].chars().count())
        };
        let sel = doc.selection();
        let in_view = doc.line_of(doc.caret) >= first;
        Reading {
            lines,
            caret: in_view.then(|| at(doc.caret)),
            selection: (!sel.is_empty()).then(|| (at(doc.anchor), at(doc.caret))),
            editable: true,
        }
    }

    pub fn node<M: Clone + 'static>(&self, name: &str, least: f32, send: impl Fn(Event) -> M + Clone + 'static) -> Node<M> {
        let paint = self.pane.clone();
        let place = self.pane.clone();
        let shift = self.pane.clone();
        let own = self.pane.clone();
        let line = self.pane.borrow().look.size * LINE;
        let (keys, point, wheel, menu) = (send.clone(), send.clone(), send.clone(), send.clone());
        drawing((0.0, least), name, move |canvas, pen| paint.borrow_mut().paint(canvas, pen))
            .width(Length::Fill)
            .height(Length::Fill)
            .reads(self.reading())
            .misspelt(self.misspelt())
            .on_pointer_mods(move |p, mods| {
                shift.borrow_mut().hold_shift(mods.shift);
                point(Event::Pointer(p))
            })
            .on_hover()
            .on_scroll(move |lines| wheel(Event::Scroll(lines * line)))
            .on_keys(move |input| (Pane::takes(input) && !own.borrow().passes(input)).then(|| keys(Event::Key(input.clone()))))
            .menu(move |_, _| {
                let key = |sym| menu(Event::Key(Input::Key { sym, text: None, mods: Mods { ctrl: true, ..Mods::default() }, repeat: false }));
                vec![
                    MenuRow::send("edit-cut-symbolic", crate::t!("kit-area-cut"), key(Keysym::x)).keys("Ctrl+X"),
                    MenuRow::send("edit-copy-symbolic", crate::t!("kit-area-copy"), key(Keysym::c)).keys("Ctrl+C"),
                    MenuRow::send("edit-paste-symbolic", crate::t!("kit-area-paste"), key(Keysym::v)).keys("Ctrl+V"),
                    MenuRow::send("edit-select-all-symbolic", crate::t!("kit-area-select-all"), key(Keysym::a)).keys("Ctrl+A"),
                ]
            })
            .cursor_with(move |x, _| if x < place.borrow().text_left() { Cursor::Arrow } else { Cursor::Text })
    }

    pub fn handle(&self, event: Event, window: &Window, later: impl Fn(Event) + 'static) -> bool {
        let released = matches!(event, Event::Pointer(Pointer::Up(..)));
        if let Event::Pointer(Pointer::Middle(x, y)) = event {
            let mut pane = self.pane.borrow_mut();
            pane.pointer(Pointer::Down(x, y));
            pane.pointer(Pointer::Up(x, y));
            drop(pane);
            window.paste_selection(move |words| later(Event::Paste(words)));
            return false;
        }
        let did = self.apply(event);
        if released {
            let pane = self.pane.borrow();
            if !pane.doc.selection().is_empty() {
                window.select(pane.doc.selected());
            }
        }
        match did {
            Did::Copy(words) => window.copy(&words),
            Did::Paste => window.paste(move |words| later(Event::Paste(words))),
            Did::Edited => return true,
            Did::Moved | Did::Nothing => {}
        }
        false
    }

    pub fn apply(&self, event: Event) -> Did {
        let mut pane = self.pane.borrow_mut();
        match event {
            Event::Key(input) => pane.key(&input).unwrap_or(Did::Nothing),
            Event::Pointer(p) => pane.pointer(p),
            Event::Scroll(points) => {
                pane.scroll(0.0, points, false);
                Did::Nothing
            }
            Event::Paste(words) => {
                pane.paste(&words);
                Did::Edited
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn area(text: &str) -> TextArea {
        TextArea::new(text, Look { font: None, size: 14.0, wrap: true, numbers: false })
    }

    fn key(sym: Keysym, text: Option<&str>, ctrl: bool) -> Event {
        Event::Key(Input::Key { sym, text: text.map(String::from), mods: Mods { ctrl, ..Mods::default() }, repeat: false })
    }

    #[test]
    fn typing_makes_lines_and_undo_takes_them_back() {
        let a = area("");
        assert_eq!(a.apply(key(Keysym::a, Some("a"), false)), Did::Edited);
        a.apply(key(Keysym::Return, Some("\n"), false));
        a.apply(key(Keysym::b, Some("b"), false));
        assert_eq!(a.text(), "a\nb");
        a.apply(key(Keysym::z, None, true));
        assert_eq!(a.text(), "a\n", "the first undo takes back the typed b alone");
        for _ in 0..3 {
            a.apply(key(Keysym::z, None, true));
        }
        assert_eq!(a.text(), "");
    }

    #[test]
    fn a_paste_goes_where_the_caret_is_and_a_copy_is_handed_back() {
        let a = area("one two");
        a.apply(key(Keysym::a, None, true));
        assert_eq!(a.apply(key(Keysym::c, None, true)), Did::Copy("one two".into()));
        assert_eq!(a.apply(key(Keysym::v, None, true)), Did::Paste);
        assert_eq!(a.apply(Event::Paste("a\r\nb".into())), Did::Edited);
        assert_eq!(a.text(), "a\nb");
        a.apply(Event::Paste("c\rd\u{0}e\u{1b}f\u{202e}g\th".into()));
        assert_eq!(a.text(), "a\nbc\nde\u{0}".replace('\u{0}', "") + "fg\th", "a lone return is a line break; NUL, escape and bidi overrides are left out");
    }

    #[test]
    fn replacing_the_text_forgets_what_belonged_to_the_old() {
        let a = area("one two");
        a.pane().borrow_mut().marks.push(0..3);
        a.set_text("other words");
        assert!(a.pane().borrow().marks.is_empty());
    }

    #[test]
    fn the_area_takes_the_keys_the_pane_answers_and_leaves_the_rest() {
        let a = area("one two");
        let keys = [
            (Keysym::a, Some("a"), false, false), (Keysym::s, None, true, false), (Keysym::F5, None, false, false), (Keysym::a, None, false, false),
            (Keysym::z, None, true, false), (Keysym::Left, None, false, true), (Keysym::Escape, None, false, false), (Keysym::Tab, Some("\t"), false, false),
            (Keysym::Tab, None, true, false), (Keysym::Insert, None, false, false), (Keysym::Insert, None, true, false), (Keysym::x, Some("x"), false, true),
        ];
        for (sym, text, ctrl, alt) in keys {
            let input = Input::Key { sym, text: text.map(String::from), mods: Mods { ctrl, alt, ..Mods::default() }, repeat: false };
            assert_eq!(Pane::takes(&input), a.pane().borrow_mut().key(&input).is_some(), "{sym:?} ctrl {ctrl} alt {alt}");
        }
    }

    #[test]
    fn an_undo_with_nothing_to_undo_changes_nothing() {
        let a = area("");
        assert_eq!(a.apply(key(Keysym::z, None, true)), Did::Nothing);
        assert_eq!(a.apply(key(Keysym::y, None, true)), Did::Nothing);
    }

    #[test]
    fn a_reader_is_told_each_line_and_where_the_caret_stands() {
        let a = area("first\nsecond\nthird");
        a.pane().borrow_mut().doc.select(8, 8);
        let r = a.reading();
        assert_eq!(r.lines, ["first", "second", "third"]);
        assert_eq!((r.caret, r.selection, r.editable), (Some((1, 2)), None, true));
        a.pane().borrow_mut().doc.select(1, 8);
        assert_eq!(a.reading().selection, Some(((0, 1), (1, 2))));
    }
}
