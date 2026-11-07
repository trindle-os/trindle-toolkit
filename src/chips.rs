//! A chips box: type, press Enter and the words become a chip; Backspace in the empty box takes the last chip away.

use super::widget::{entry, row, word_button, Length, Node};
use crate::t;

#[derive(Clone, Debug, PartialEq)]
pub enum Edit {
    Draft(String),
    Add,
    Remove(usize),
    Back,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Chips {
    pub chips: Vec<String>,
    pub draft: String,
}

impl Chips {
    pub fn update(&mut self, edit: Edit) {
        match edit {
            Edit::Draft(s) => self.draft = s,
            Edit::Add => {
                let word = self.draft.trim().to_string();
                if !word.is_empty() && !self.chips.contains(&word) {
                    self.chips.push(word);
                }
                self.draft.clear();
            }
            Edit::Remove(i) => {
                if i < self.chips.len() {
                    self.chips.remove(i);
                }
            }
            Edit::Back => {
                self.chips.pop();
            }
        }
    }

    pub fn view<M: Clone + 'static>(&self, tip: impl Into<String>, wrap: impl Fn(Edit) -> M + Clone + 'static) -> Node<M> {
        let typed = wrap.clone();
        let mut all: Vec<Node<M>> = self.chips.iter().enumerate().map(|(i, c)| {
            word_button(c.clone()).spoken(t!("kit-chip-remove", name = c.as_str())).on_press(wrap(Edit::Remove(i)))
        }).collect();
        all.push(entry(self.draft.clone(), tip, move |s| typed(Edit::Draft(s))).on_submit(wrap(Edit::Add)).on_backspace_empty(wrap(Edit::Back)).width(Length::Fixed(160.0)));
        row(all).gap(crate::theme::gaps().medium).wraps()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn enter_makes_a_chip_once_and_backspace_takes_the_last_away() {
        let mut c = Chips::default();
        for w in [" red ", "red", "", "blue"] {
            c.update(Edit::Draft(w.into()));
            c.update(Edit::Add);
        }
        assert_eq!((c.chips.as_slice(), c.draft.as_str()), (["red".to_string(), "blue".to_string()].as_slice(), ""));
        c.update(Edit::Remove(9));
        c.update(Edit::Back);
        assert_eq!(c.chips, ["red"]);
        c.update(Edit::Remove(0));
        c.update(Edit::Back);
        assert!(c.chips.is_empty());
    }
}
