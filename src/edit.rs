//! The text in a text box and its caret, by what a person sees as one letter.

use std::ops::Range;

use unicode_segmentation::UnicodeSegmentation;
use zeroize::Zeroize;

use crate::input::Keysym;

#[derive(Clone, Copy, Debug, PartialEq)]
enum Run {
    None,
    Typing,
    Deleting,
}

const MOST_UNDO: usize = 100;

pub fn facing(sym: Keysym, text: &str) -> Keysym {
    match sym {
        Keysym::Left | Keysym::KP_Left if right_to_left(text) => Keysym::Right,
        Keysym::Right | Keysym::KP_Right if right_to_left(text) => Keysym::Left,
        other => other,
    }
}

pub fn right_to_left(text: &str) -> bool {
    unicode_bidi::get_base_direction(text) == unicode_bidi::Direction::Rtl
}

#[derive(Clone)]
pub(crate) struct Edit {
    pub text: String,
    pub caret: usize,
    pub anchor: usize,
    pub scroll: f32,
    undo: Vec<(String, usize)>,
    redo: Vec<(String, usize)>,
    run: Run,
    pub keep_history: bool,
}

impl std::fmt::Debug for Edit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Edit").field("caret", &self.caret).field("anchor", &self.anchor).finish_non_exhaustive()
    }
}

impl Edit {
    pub fn new(text: &str) -> Edit {
        Edit { text: text.into(), caret: text.len(), anchor: text.len(), scroll: 0.0, undo: Vec::new(), redo: Vec::new(), run: Run::None, keep_history: true }
    }

    pub fn sync(&mut self, text: &str) {
        if self.text == text {
            return;
        }
        self.text.zeroize();
        self.text = text.into();
        self.caret = self.edge_at(self.caret);
        self.anchor = self.edge_at(self.anchor);
        self.run = Run::None;
    }

    fn edge_at(&self, at: usize) -> usize {
        if at >= self.text.len() {
            return self.text.len();
        }
        self.text.grapheme_indices(true).map(|(i, _)| i).take_while(|&i| i <= at).last().unwrap_or(0)
    }

    pub fn selection(&self) -> Range<usize> {
        self.caret.min(self.anchor)..self.caret.max(self.anchor)
    }

    pub fn selected(&self) -> &str {
        &self.text[self.selection()]
    }

    pub fn prev(&self, at: usize) -> usize {
        self.text[..at].grapheme_indices(true).next_back().map_or(0, |(i, _)| i)
    }

    pub fn next(&self, at: usize) -> usize {
        self.text[at..].graphemes(true).next().map_or(at, |g| at + g.len())
    }

    pub fn prev_word(&self, at: usize) -> usize {
        self.text.unicode_word_indices().map(|(i, _)| i).rfind(|&i| i < at).unwrap_or(0)
    }

    pub fn next_word(&self, at: usize) -> usize {
        self.text.unicode_word_indices().map(|(i, w)| i + w.len()).find(|&e| e > at).unwrap_or(self.text.len())
    }

    pub fn word_round(&self, at: usize) -> Range<usize> {
        self.text.split_word_bound_indices().map(|(i, w)| i..i + w.len())
            .find(|r| r.contains(&at)).or_else(|| self.text.split_word_bound_indices().next_back().map(|(i, w)| i..i + w.len()))
            .unwrap_or(0..0)
    }

    pub fn move_to(&mut self, at: usize, extend: bool) {
        self.caret = self.edge_at(at);
        if !extend {
            self.anchor = self.caret;
        }
        self.run = Run::None;
    }

    pub fn select(&mut self, r: Range<usize>) {
        self.anchor = self.edge_at(r.start);
        self.caret = self.edge_at(r.end);
        self.run = Run::None;
    }

    fn remember(&mut self, run: Run) {
        if self.keep_history && (run == Run::None || run != self.run) {
            if self.undo.len() >= MOST_UNDO {
                let (mut oldest, _) = self.undo.remove(0);
                oldest.zeroize();
            }
            self.undo.push((self.text.clone(), self.caret));
        }
        wipe(&mut self.redo);
        self.run = run;
    }

    pub fn insert(&mut self, s: &str) -> bool {
        if s.is_empty() {
            return false;
        }
        let run = if self.anchor == self.caret && !s.contains(char::is_whitespace) && s.chars().count() == 1 { Run::Typing } else { Run::None };
        self.remember(run);
        let r = self.selection();
        self.make_room(s.len());
        self.text.replace_range(r.clone(), s);
        self.caret = r.start + s.len();
        self.anchor = self.caret;
        true
    }

    fn make_room(&mut self, extra: usize) {
        if self.text.len() + extra <= self.text.capacity() {
            return;
        }
        let mut bigger = String::with_capacity((self.text.len() + extra).max(32).next_power_of_two());
        bigger.push_str(&self.text);
        self.text.zeroize();
        self.text = bigger;
    }

    pub fn delete_to(&mut self, to: usize) -> bool {
        let r = if self.anchor != self.caret {
            self.selection()
        } else {
            let to = self.edge_at(to);
            to.min(self.caret)..to.max(self.caret)
        };
        if r.is_empty() {
            return false;
        }
        self.remember(if self.anchor == self.caret { Run::Deleting } else { Run::None });
        self.text.replace_range(r.clone(), "");
        self.caret = r.start;
        self.anchor = r.start;
        true
    }

    pub fn undo(&mut self) -> bool {
        let Some((text, caret)) = self.undo.pop() else { return false };
        self.redo.push((std::mem::replace(&mut self.text, text), self.caret));
        self.caret = caret.min(self.text.len());
        self.anchor = self.caret;
        self.run = Run::None;
        true
    }

    pub fn redo(&mut self) -> bool {
        let Some((text, caret)) = self.redo.pop() else { return false };
        self.undo.push((std::mem::replace(&mut self.text, text), self.caret));
        self.caret = caret.min(self.text.len());
        self.anchor = self.caret;
        self.run = Run::None;
        true
    }

    pub fn can_undo(&self) -> bool {
        !self.undo.is_empty()
    }

    pub fn can_redo(&self) -> bool {
        !self.redo.is_empty()
    }
}

pub(crate) fn dots(text: &str) -> String {
    "\u{2022}".repeat(text.graphemes(true).count())
}

pub(crate) fn to_dots(text: &str, at: usize) -> usize {
    text[..at.min(text.len())].graphemes(true).count() * '\u{2022}'.len_utf8()
}

pub(crate) fn from_dots(text: &str, at: usize) -> usize {
    let n = at / '\u{2022}'.len_utf8();
    text.grapheme_indices(true).nth(n).map_or(text.len(), |(i, _)| i)
}

impl Drop for Edit {
    fn drop(&mut self) {
        self.text.zeroize();
        wipe(&mut self.undo);
        wipe(&mut self.redo);
    }
}

fn wipe(history: &mut Vec<(String, usize)>) {
    for (text, _) in history.iter_mut() {
        text.zeroize();
    }
    history.clear();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_caret_steps_over_whole_letters() {
        let e = Edit::new("e\u{301}\u{1f1ec}\u{1f1e7}\u{1f468}\u{200d}\u{1f469}\u{200d}\u{1f467}x");
        let mut at = 0;
        let mut stops = vec![at];
        while at < e.text.len() {
            at = e.next(at);
            stops.push(at);
        }
        assert_eq!(stops.len(), 5);
        assert_eq!(e.prev(e.text.len()), e.text.len() - 1);
        assert_eq!(e.prev(3), 0);
    }

    #[test]
    fn a_caret_or_selection_set_past_the_end_or_inside_a_letter_snaps_to_an_edge() {
        let mut e = Edit::new("ae\u{301}b");
        e.move_to(99, false);
        assert_eq!((e.caret, e.anchor), (e.text.len(), e.text.len()));
        e.move_to(3, false);
        assert_eq!(e.caret, 1);
        e.select(2..99);
        assert_eq!(e.selected(), "e\u{301}b");
        e.select(0..2);
        assert_eq!(e.selected(), "a");
        e.move_to(1, false);
        assert!(e.delete_to(99));
        assert_eq!(e.text, "a");
    }

    #[test]
    fn words_are_found_both_ways() {
        let e = Edit::new("one two, three");
        assert_eq!(e.prev_word(14), 9);
        assert_eq!(e.prev_word(9), 4);
        assert_eq!(e.prev_word(2), 0);
        assert_eq!(e.next_word(0), 3);
        assert_eq!(e.next_word(3), 7);
        assert_eq!(e.next_word(13), 14);
        assert_eq!(e.word_round(5), 4..7);
        assert_eq!(e.word_round(14), 9..14);
    }

    #[test]
    fn a_secret_box_keeps_no_earlier_version() {
        let mut e = Edit::new("");
        e.keep_history = false;
        for c in ["a", "b", " ", "c"] {
            e.insert(c);
        }
        assert!(!e.undo());
        assert_eq!(e.text, "ab c");
    }

    #[test]
    fn typing_replaces_the_selection_and_undoes_a_word_at_a_time() {
        let mut e = Edit::new("");
        for c in ["h", "i", " ", "y", "o", "u"] {
            e.insert(c);
        }
        assert_eq!(e.text, "hi you");
        e.undo();
        assert_eq!(e.text, "hi ");
        e.undo();
        assert_eq!(e.text, "hi");
        e.redo();
        e.redo();
        assert_eq!((e.text.as_str(), e.caret), ("hi you", 6));
        e.select(3..6);
        e.insert("me");
        assert_eq!(e.text, "hi me");
        e.undo();
        assert_eq!(e.text, "hi you");
    }

    #[test]
    fn deleting_takes_the_selection_or_a_letter() {
        let mut e = Edit::new("ab\u{1f600}c");
        e.move_to(6, false);
        assert!(e.delete_to(e.prev(e.caret)));
        assert_eq!(e.text, "abc");
        e.move_to(0, false);
        assert!(!e.delete_to(e.prev(e.caret)));
        e.select(0..2);
        e.delete_to(3);
        assert_eq!((e.text.as_str(), e.caret), ("c", 0));
        let mut e = Edit::new("hello");
        e.delete_to(e.prev(5));
        e.delete_to(e.prev(4));
        e.undo();
        assert_eq!(e.text, "hello");
    }

    #[test]
    fn the_programs_own_change_keeps_the_caret_on_a_letter() {
        let mut e = Edit::new("caf\u{e9}s");
        e.sync("caf\u{e9}");
        assert_eq!(e.caret, 5);
        e.sync("ab");
        assert_eq!(e.caret, 2);
        e.move_to(1, false);
        e.sync("\u{e9}\u{e9}");
        assert_eq!(e.caret, 0);
    }

    #[test]
    fn a_password_shows_one_dot_a_letter() {
        let t = "a\u{1f600}b";
        assert_eq!(dots(t), "\u{2022}\u{2022}\u{2022}");
        assert_eq!(to_dots(t, 5), 6);
        assert_eq!(from_dots(t, 6), 5);
        assert_eq!(from_dots(t, 9), t.len());
    }
}
