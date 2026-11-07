//! One open file's text: the caret and selection, undo and redo, where each line starts, and the searching and counting done on it. No drawing here.

use std::collections::VecDeque;
use std::ops::Range;

use unicode_segmentation::UnicodeSegmentation;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Kind {
    Type,
    Delete,
    Other,
}

#[derive(Clone, Debug)]
struct Change {
    at: usize,
    removed: String,
    inserted: String,
    before: (usize, usize),
    after: (usize, usize),
    kind: Kind,
    id: u64,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Find {
    pub case: bool,
    pub whole: bool,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Counts {
    pub words: usize,
    pub letters: usize,
}

pub struct Doc {
    text: String,
    starts: Vec<usize>,
    pub caret: usize,
    pub anchor: usize,
    undo: VecDeque<Change>,
    redo: Vec<Change>,
    next_id: u64,
    saved: u64,
    sealed: bool,
    pub version: u64,
    ending: &'static str,
}

const UNDO_LIMIT: usize = 2000;
const UNDO_BYTES: usize = 64 << 20;

impl Doc {
    pub fn new(text: String) -> Doc {
        let mut ending = "\n";
        let text = if text.contains('\r') {
            let crlf = text.matches("\r\n").count();
            let (cr, lf) = (text.matches('\r').count() - crlf, text.matches('\n').count() - crlf);
            ending = if crlf >= lf && crlf >= cr { "\r\n" } else if lf >= cr { "\n" } else { "\r" };
            text.replace("\r\n", "\n").replace('\r', "\n")
        } else {
            text
        };
        let starts = line_starts(&text);
        Doc {
            text,
            starts,
            caret: 0,
            anchor: 0,
            undo: VecDeque::new(),
            redo: Vec::new(),
            next_id: 1,
            saved: 0,
            sealed: false,
            version: 0,
            ending,
        }
    }

    pub fn saved_text(&self) -> String {
        if self.ending == "\n" { self.text.clone() } else { self.text.replace('\n', self.ending) }
    }

    pub fn text(&self) -> &str {
        &self.text
    }

    pub fn len(&self) -> usize {
        self.text.len()
    }

    pub fn is_empty(&self) -> bool {
        self.text.is_empty()
    }

    pub fn lines(&self) -> usize {
        self.starts.len()
    }

    pub fn line_range(&self, i: usize) -> Range<usize> {
        let start = self.starts[i];
        let end = self.starts.get(i + 1).map(|s| s - 1).unwrap_or(self.text.len());
        start..end
    }

    pub fn line(&self, i: usize) -> &str {
        &self.text[self.line_range(i)]
    }

    pub fn line_of(&self, at: usize) -> usize {
        let at = at.min(self.text.len());
        match self.starts.binary_search(&at) {
            Ok(i) => i,
            Err(i) => i - 1,
        }
    }

    pub fn line_col(&self, at: usize) -> (usize, usize) {
        let at = self.fit(at);
        let line = self.line_of(at);
        let start = self.starts[line];
        (line + 1, self.text[start..at].chars().count() + 1)
    }

    pub fn line_start(&self, n: usize) -> Option<usize> {
        if n == 0 {
            return None;
        }
        Some(self.starts[(n - 1).min(self.starts.len() - 1)])
    }

    pub fn selection(&self) -> Range<usize> {
        self.caret.min(self.anchor)..self.caret.max(self.anchor)
    }

    pub fn selected(&self) -> &str {
        &self.text[self.selection()]
    }

    pub fn select(&mut self, anchor: usize, caret: usize) {
        self.anchor = self.fit(anchor);
        self.caret = self.fit(caret);
        self.sealed = true;
    }

    pub fn go(&mut self, to: usize, extend: bool) {
        self.caret = self.fit(to);
        if !extend {
            self.anchor = self.caret;
        }
        self.sealed = true;
    }

    fn fit(&self, mut at: usize) -> usize {
        at = at.min(self.text.len());
        while !self.text.is_char_boundary(at) {
            at -= 1;
        }
        at
    }

    pub fn put(&mut self, s: &str, kind: Kind) {
        let range = self.selection();
        self.replace(range, s, kind);
    }

    pub fn replace(&mut self, range: Range<usize>, s: &str, kind: Kind) {
        let range = self.fit(range.start)..self.fit(range.end).max(self.fit(range.start));
        if range.is_empty() && s.is_empty() {
            return;
        }
        let before = (self.caret, self.anchor);
        let removed = self.text[range.clone()].to_string();
        let at = range.start;
        self.splice(range, s);
        self.caret = at + s.len();
        self.anchor = self.caret;
        self.redo.clear();
        let after = (self.caret, self.anchor);
        if !self.sealed && self.joins(at, &removed, s, kind) {
            let last = self.undo.back_mut().expect("joins needs a last change");
            if kind == Kind::Type {
                last.inserted.push_str(s);
            } else {
                last.at = at;
                last.removed.insert_str(0, &removed);
            }
            last.after = after;
        } else {
            let id = self.next_id;
            self.next_id += 1;
            self.undo.push_back(Change { at, removed, inserted: s.to_string(), before, after, kind, id });
            let mut held: usize = self.undo.iter().map(|c| c.removed.len() + c.inserted.len()).sum();
            while self.undo.len() > 1 && (self.undo.len() > UNDO_LIMIT || held > UNDO_BYTES) {
                if let Some(old) = self.undo.pop_front() {
                    held -= old.removed.len() + old.inserted.len();
                }
            }
        }
        self.sealed = kind == Kind::Other || s.contains('\n');
    }

    fn joins(&self, at: usize, removed: &str, inserted: &str, kind: Kind) -> bool {
        let Some(last) = self.undo.back() else { return false };
        if last.kind != kind || last.id == self.saved {
            return false;
        }
        match kind {
            Kind::Type => removed.is_empty() && at == last.at + last.inserted.len() && !inserted.contains('\n'),
            Kind::Delete => inserted.is_empty() && last.inserted.is_empty() && at + removed.len() == last.at,
            Kind::Other => false,
        }
    }

    fn splice(&mut self, range: Range<usize>, s: &str) {
        let first = self.line_of(range.start);
        let last = self.line_of(range.end);
        self.text.replace_range(range.clone(), s);
        let shift = s.len() as isize - range.len() as isize;
        let new: Vec<usize> = s.match_indices('\n').map(|(i, _)| range.start + i + 1).collect();
        let from = first + 1 + new.len();
        self.starts.splice(first + 1..last + 1, new);
        for start in &mut self.starts[from..] {
            *start = (*start as isize + shift) as usize;
        }
        self.version += 1;
    }

    pub fn undo(&mut self) -> bool {
        let Some(c) = self.undo.pop_back() else { return false };
        self.splice(c.at..c.at + c.inserted.len(), &c.removed);
        (self.caret, self.anchor) = c.before;
        self.redo.push(c);
        self.sealed = true;
        true
    }

    pub fn redo(&mut self) -> bool {
        let Some(c) = self.redo.pop() else { return false };
        self.splice(c.at..c.at + c.removed.len(), &c.inserted);
        (self.caret, self.anchor) = c.after;
        self.undo.push_back(c);
        self.sealed = true;
        true
    }

    pub fn can_undo(&self) -> bool {
        !self.undo.is_empty()
    }

    pub fn can_redo(&self) -> bool {
        !self.redo.is_empty()
    }

    pub fn modified(&self) -> bool {
        self.undo.back().map_or(0, |c| c.id) != self.saved
    }

    pub fn mark_saved(&mut self) {
        self.saved = self.undo.back().map_or(0, |c| c.id);
        self.sealed = true;
    }

    pub fn mark_unsaved(&mut self) {
        self.saved = u64::MAX;
    }

    pub fn prev_char(&self, at: usize) -> usize {
        let at = self.fit(at);
        if at == 0 {
            return 0;
        }
        let start = self.starts[self.line_of(at)];
        if at == start {
            return at - 1;
        }
        self.text[start..at].grapheme_indices(true).next_back().map_or(start, |(i, _)| start + i)
    }

    pub fn next_char(&self, at: usize) -> usize {
        let at = self.fit(at);
        self.text[at..].graphemes(true).next().map_or(at, |g| at + g.len())
    }

    pub fn word_left(&self, at: usize) -> usize {
        let at = self.fit(at);
        let in_word = |j: usize, g: &str| part_of_word(&self.text, j, j + g.len());
        let mut letters = self.text[..at].grapheme_indices(true).rev().peekable();
        let mut i = at;
        while let Some((j, _)) = letters.next_if(|(_, g)| g.chars().all(char::is_whitespace)) {
            i = j;
        }
        match letters.peek() {
            Some(&(j, g)) if in_word(j, g) => {
                for (j, g) in letters {
                    if !in_word(j, g) {
                        break;
                    }
                    i = j;
                }
                i
            }
            Some(&(j, _)) => j,
            None => i,
        }
    }

    pub fn word_right(&self, at: usize) -> usize {
        let at = self.fit(at);
        let in_word = |j: usize, g: &str| part_of_word(&self.text, at + j, at + j + g.len());
        let mut letters = self.text[at..].grapheme_indices(true).peekable();
        while letters.next_if(|(_, g)| g.chars().all(char::is_whitespace)).is_some() {}
        match letters.peek() {
            Some(&(j, g)) if in_word(j, g) => letters.find(|&(j, g)| !in_word(j, g)).map_or(self.text.len(), |(j, _)| at + j),
            Some(&(j, g)) => at + j + g.len(),
            None => self.text.len(),
        }
    }

    pub fn word_at(&self, at: usize) -> Range<usize> {
        let at = self.fit(at);
        let start = self.text[..at].grapheme_indices(true).rev().take_while(|&(i, g)| part_of_word(&self.text, i, i + g.len())).last().map_or(at, |(i, _)| i);
        let end = self.text[at..].grapheme_indices(true).find(|&(i, g)| !part_of_word(&self.text, at + i, at + i + g.len())).map_or(self.text.len(), |(i, _)| at + i);
        start..end
    }

    pub fn counts(&self, range: Range<usize>) -> Counts {
        counts(&self.text[self.fit(range.start)..self.fit(range.end).max(self.fit(range.start))])
    }

    pub fn find(&self, needle: &str, how: Find) -> Vec<Range<usize>> {
        find_all(&self.text, needle, how)
    }

    pub fn replace_all(&mut self, needle: &str, with: &str, how: Find) -> usize {
        let found = self.find(needle, how);
        if found.is_empty() {
            return 0;
        }
        let mut out = String::with_capacity(self.text.len());
        let mut last = 0;
        for r in &found {
            out.push_str(&self.text[last..r.start]);
            out.push_str(with);
            last = r.end;
        }
        out.push_str(&self.text[last..]);
        let caret = self.caret;
        self.replace(0..self.text.len(), &out, Kind::Other);
        let caret = self.fit(caret);
        self.caret = caret;
        self.anchor = caret;
        found.len()
    }
}

pub fn is_word(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
        || matches!(c, '\u{300}'..='\u{36f}' | '\u{1ab0}'..='\u{1aff}' | '\u{1dc0}'..='\u{1dff}' | '\u{20d0}'..='\u{20ff}' | '\u{fe00}'..='\u{fe0f}' | '\u{fe20}'..='\u{fe2f}' | '\u{200d}')
}

fn part_of_word(text: &str, from: usize, to: usize) -> bool {
    match text[from..to].chars().next() {
        Some('\'') => text[..from].chars().next_back().is_some_and(char::is_alphanumeric) && text[to..].chars().next().is_some_and(char::is_alphanumeric),
        Some(c) => is_word(c),
        None => false,
    }
}

fn line_starts(text: &str) -> Vec<usize> {
    std::iter::once(0).chain(text.match_indices('\n').map(|(i, _)| i + 1)).collect()
}

pub fn counts(text: &str) -> Counts {
    Counts {
        words: text.split_whitespace().filter(|w| w.chars().any(char::is_alphanumeric)).count(),
        letters: text.chars().filter(|c| !c.is_whitespace()).count(),
    }
}

pub fn find_all(text: &str, needle: &str, how: Find) -> Vec<Range<usize>> {
    let mut out = Vec::new();
    if needle.is_empty() {
        return out;
    }
    let mut from = 0;
    while from < text.len() {
        let Some(r) = find_from(text, needle, how, from) else { break };
        from = r.end;
        out.push(r);
    }
    out
}

fn find_from(text: &str, needle: &str, how: Find, from: usize) -> Option<Range<usize>> {
    if how.case {
        let mut at = from;
        while let Some(i) = text[at..].find(needle) {
            let r = at + i..at + i + needle.len();
            if !how.whole || whole(text, &r) {
                return Some(r);
            }
            at = text[r.start..].chars().next().map(|c| r.start + c.len_utf8())?;
        }
        return None;
    }
    let first = needle.chars().next()?;
    for (i, c) in text[from..].char_indices() {
        if !same(c, first) {
            continue;
        }
        let start = from + i;
        if let Some(end) = match_here(text, start, needle) {
            let r = start..end;
            if !how.whole || whole(text, &r) {
                return Some(r);
            }
        }
    }
    None
}

fn match_here(text: &str, start: usize, needle: &str) -> Option<usize> {
    let mut hay = text[start..].char_indices();
    for n in needle.chars() {
        let (_, h) = hay.next()?;
        if !same(h, n) {
            return None;
        }
    }
    Some(hay.next().map(|(i, _)| start + i).unwrap_or(text.len()))
}

fn same(a: char, b: char) -> bool {
    a == b || a.to_lowercase().eq(b.to_lowercase())
}

fn whole(text: &str, r: &Range<usize>) -> bool {
    let before = text[..r.start].grapheme_indices(true).next_back().is_none_or(|(i, g)| !part_of_word(text, i, i + g.len()));
    let after = text[r.end..].graphemes(true).next().is_none_or(|g| !part_of_word(text, r.end, r.end + g.len()));
    before && after
}

#[cfg(test)]
mod tests {
    use super::*;

    fn typed(doc: &mut Doc, s: &str) {
        for c in s.chars() {
            doc.put(&c.to_string(), Kind::Type);
        }
    }

    #[test]
    fn line_starts_stay_true_after_every_change() {
        let pieces = ["", "x", "é\n", "\n", "## Choosing a model\n\n- Every job \u{2014} helpers too.\n- Haiku 5.5\n", "a\nb\nc\nd\ne\nf\n"];
        let mut d = Doc::new("one\ntwo \u{2014} three\nfour\n\nfive".into());
        let mut seed = 7usize;
        for _ in 0..400 {
            seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            let text = d.text().to_string();
            let bounds: Vec<usize> = (0..=text.len()).filter(|&i| text.is_char_boundary(i)).collect();
            let a = bounds[(seed >> 8) % bounds.len()];
            let b = bounds[(seed >> 24) % bounds.len()];
            let (a, b) = (a.min(b), a.max(b));
            let b = if (seed >> 40) % 3 == 0 { b } else { a };
            d.replace(a..b, pieces[(seed >> 33) % pieces.len()], Kind::Other);
            assert_eq!(d.starts, line_starts(d.text()), "after a change at {a}..{b}");
            for p in 0..d.lines() {
                let _ = d.line(p);
            }
        }
    }

    #[test]
    fn lines_and_columns_follow_edits() {
        let mut d = Doc::new("one\ntwo\nthree".into());
        assert_eq!(d.lines(), 3);
        assert_eq!(d.line(1), "two");
        assert_eq!(d.line_col(9), (3, 2));
        d.go(4, false);
        typed(&mut d, "a\nb");
        assert_eq!(d.text(), "one\na\nbtwo\nthree");
        assert_eq!(d.lines(), 4);
        assert_eq!(d.line(2), "btwo");
        assert_eq!(d.line_col(d.caret), (3, 2));
        d.replace(2..8, "", Kind::Other);
        assert_eq!(d.text(), "onwo\nthree");
        assert_eq!(d.lines(), 2);
        assert_eq!(d.line(1), "three");
        assert_eq!(Doc::new("é\nü".into()).line_col(5), (2, 2));
    }

    #[test]
    fn a_letter_moves_whole_and_windows_line_breaks_are_kept_for_saving() {
        let d = Doc::new("e\u{301}\u{1f468}\u{200d}\u{1f469}\nx".into());
        let (accent, family_end) = (d.next_char(0), d.next_char(d.next_char(0)));
        assert_eq!((accent, d.prev_char(accent)), (3, 0));
        assert_eq!(d.text()[accent..family_end], *"\u{1f468}\u{200d}\u{1f469}");
        assert_eq!(d.prev_char(family_end), accent);
        assert_eq!(d.prev_char(family_end + 1), family_end);
        assert_eq!(d.word_at(1), 0..3);
        let w = Doc::new("a\r\nb\rc\n".into());
        assert_eq!((w.text(), w.lines()), ("a\nb\nc\n", 4));
        assert_eq!(w.saved_text(), "a\r\nb\r\nc\r\n");
        assert_eq!(Doc::new("a\nb".into()).saved_text(), "a\nb");
        assert_eq!(Doc::new("a\nb\nc\r\nd".into()).saved_text(), "a\nb\nc\nd", "most lines end in LF, so one CRLF does not change them all");
        assert_eq!(Doc::new("a\rb\r".into()).saved_text(), "a\rb\r");
    }

    #[test]
    fn go_to_line() {
        let d = Doc::new("a\nbb\nccc\n".into());
        assert_eq!(d.line_start(1), Some(0));
        assert_eq!(d.line_start(3), Some(5));
        assert_eq!(d.line_start(4), Some(9));
        assert_eq!(d.line_start(99), Some(9));
        assert_eq!(d.line_start(0), None);
    }

    #[test]
    fn typing_undoes_as_one_step_and_saved_is_tracked() {
        let mut d = Doc::new(String::new());
        typed(&mut d, "hello");
        assert!(d.modified());
        d.mark_saved();
        assert!(!d.modified());
        typed(&mut d, " world");
        assert!(d.modified());
        assert!(d.undo());
        assert_eq!(d.text(), "hello");
        assert!(!d.modified());
        assert!(d.undo());
        assert_eq!(d.text(), "");
        assert!(d.redo() && d.redo());
        assert_eq!(d.text(), "hello world");
        assert_eq!(d.caret, 11);
    }

    #[test]
    fn backspacing_undoes_as_one_step() {
        let mut d = Doc::new("abcdef".into());
        d.go(6, false);
        for _ in 0..3 {
            let to = d.prev_char(d.caret);
            d.replace(to..d.caret, "", Kind::Delete);
        }
        assert_eq!(d.text(), "abc");
        d.undo();
        assert_eq!(d.text(), "abcdef");
        assert!(!d.modified());
    }

    #[test]
    fn find_with_and_without_capitals_and_whole_words() {
        let text = "The cat sat on the Cathedral. THE END the";
        let any = Find::default();
        assert_eq!(find_all(text, "the", any).len(), 5, "Cathedral holds one too");
        assert_eq!(find_all(text, "the", Find { case: true, whole: false }).len(), 3);
        assert_eq!(find_all(text, "the", Find { case: false, whole: true }).len(), 4);
        assert_eq!(find_all(text, "cat", any), vec![4..7, 19..22]);
        assert_eq!(find_all(text, "cat", Find { case: false, whole: true }), vec![4..7]);
        assert_eq!(find_all("Straße STRASSE", "straße", any), vec![0..7]);
        assert_eq!(find_all("aaaa", "aa", any), vec![0..2, 2..4]);
        assert!(find_all("abc", "", any).is_empty());
    }

    #[test]
    fn replace_all_is_one_undo() {
        let mut d = Doc::new("red, Red, bored, red".into());
        let n = d.replace_all("red", "blue", Find { case: false, whole: true });
        assert_eq!(n, 3);
        assert_eq!(d.text(), "blue, blue, bored, blue");
        d.undo();
        assert_eq!(d.text(), "red, Red, bored, red");
        assert_eq!(d.replace_all("green", "x", Find::default()), 0);
    }

    #[test]
    fn words_and_letters() {
        assert_eq!(counts("Hello, world! It's 2026 - fine.\n"), Counts { words: 5, letters: 26 });
        assert_eq!(counts("   \n "), Counts { words: 0, letters: 0 });
        let mut d = Doc::new("one two three".into());
        d.select(4, 13);
        assert_eq!(d.counts(d.selection()), Counts { words: 2, letters: 8 });
    }

    #[test]
    fn word_moves() {
        let d = Doc::new("hello big  world".into());
        assert_eq!(d.word_right(0), 5);
        assert_eq!(d.word_right(5), 9);
        assert_eq!(d.word_left(16), 11);
        assert_eq!(d.word_left(11), 6);
        assert_eq!(d.word_at(7), 6..9);
    }

    #[test]
    fn an_apostrophe_is_a_word_letter_only_between_letters() {
        let d = Doc::new("don't 'quoted' stop".into());
        assert_eq!(&d.text()[d.word_at(2)], "don't");
        assert_eq!(&d.text()[d.word_at(9)], "quoted");
        assert_eq!(d.word_right(0), 5);
        assert_eq!(d.word_left(5), 0);
        assert_eq!(d.word_right(5), 7, "past the space and over the opening quote, which is no word letter");
        assert!(d.find("t", Find { case: true, whole: true }).is_empty(), "t in don't is not a whole word");
        assert_eq!(d.find("quoted", Find { case: true, whole: true }).len(), 1);
    }

    #[test]
    fn a_mark_on_a_letter_goes_with_it_in_word_moves() {
        let d = Doc::new("cafe\u{301} ok".into());
        assert_eq!(d.word_right(0), 6);
        assert_eq!(d.word_left(6), 0);
        let hindi = Doc::new("\u{939}\u{93f}\u{928}\u{94d}\u{926}\u{940} x".into());
        assert_eq!(&hindi.text()[hindi.word_at(0)], "\u{939}\u{93f}\u{928}\u{94d}\u{926}\u{940}");
    }

    #[test]
    fn offsets_past_the_end_or_inside_a_letter_are_fitted_not_panicked_on() {
        let mut d = Doc::new("a\u{e9}b".into());
        assert_eq!(d.line_col(99), (1, 4));
        assert_eq!(d.counts(0..99).letters, 3);
        assert_eq!(d.prev_char(2), 0, "inside a letter counts from the letter's start");
        assert_eq!(d.next_char(2), 3);
        d.replace(2..99, "z", Kind::Other);
        assert_eq!(d.text(), "az");
        assert!(!d.is_empty());
    }

    #[test]
    fn only_so_many_steps_of_undo_are_kept() {
        let mut d = Doc::new(String::new());
        for i in 0..UNDO_LIMIT + 50 {
            d.replace(0..0, &i.to_string(), Kind::Other);
        }
        let mut steps = 0;
        while d.undo() {
            steps += 1;
        }
        assert_eq!(steps, UNDO_LIMIT);
    }
}
