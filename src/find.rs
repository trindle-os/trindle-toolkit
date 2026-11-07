//! A find bar: a typing box, how many matches and which one, and buttons for the next and the one before.

use std::ops::Range;

use super::widget::{button, entry, note, row, Length, Node};
use crate::t;

#[derive(Clone, Debug, PartialEq)]
pub enum Edit {
    Query(String),
    Next,
    Previous,
    Close,
}

pub fn find_in(text: &str, query: &str) -> Vec<Range<usize>> {
    super::editing::doc::find_all(text, query, super::editing::doc::Find::default())
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Find {
    pub query: String,
    pub count: usize,
    pub at: usize,
}

impl Find {
    pub fn update(&mut self, edit: Edit) {
        match edit {
            Edit::Query(q) => { self.query = q; self.at = 0; }
            Edit::Next if self.count > 0 => self.at = (self.at + 1) % self.count,
            Edit::Previous if self.count > 0 => self.at = (self.at + self.count - 1) % self.count,
            _ => {}
        }
    }

    pub fn set_count(&mut self, count: usize) {
        self.count = count;
        self.at = if count == 0 { 0 } else { self.at.min(count - 1) };
    }

    pub fn view<M: Clone + 'static>(&self, wrap: impl Fn(Edit) -> M + Clone + 'static) -> Node<M> {
        let typed = wrap.clone();
        let said = match (self.query.is_empty(), self.count) {
            (true, _) => String::new(),
            (false, 0) => t!("kit-find-none").to_string(),
            (false, n) => t!("kit-find-count", at = self.at as i64 + 1, count = n as i64),
        };
        let off = self.count == 0;
        row(vec![
            entry(self.query.clone(), t!("kit-find"), move |s| typed(Edit::Query(s))).on_submit(wrap(Edit::Next)).on_escape(wrap(Edit::Close)),
            note(said).width(Length::Shrink),
            button("go-up-symbolic", t!("kit-find-previous")).on_press(wrap(Edit::Previous)).off(off),
            button("go-down-symbolic", t!("kit-find-next")).on_press(wrap(Edit::Next)).off(off),
        ]).gap(crate::theme::gaps().medium).centred()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matches_ignore_case_and_never_overlap() {
        assert_eq!(find_in("Banana bandana", "ana"), [1..4, 11..14]);
        assert_eq!(find_in("aaaa", "aa"), [0..2, 2..4]);
        assert_eq!(find_in("Straße STRASSE", "ß"), [4..6]);
        assert_eq!(find_in("Éclair éclat", "éCL"), [0..4, 8..12]);
        assert!(find_in("abc", "").is_empty() && find_in("abc", "d").is_empty());
    }

    #[test]
    fn next_and_previous_wrap_and_a_new_query_starts_again() {
        let mut f = Find::default();
        f.update(Edit::Query("a".into()));
        f.set_count(3);
        f.update(Edit::Previous);
        assert_eq!(f.at, 2);
        f.update(Edit::Next);
        assert_eq!(f.at, 0);
        f.update(Edit::Next);
        f.update(Edit::Query("ab".into()));
        assert_eq!(f.at, 0);
        f.set_count(0);
        f.update(Edit::Next);
        assert_eq!(f.at, 0);
    }
}
