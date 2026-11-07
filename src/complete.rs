//! A text box with suggestions in a list under it (never floating): typing narrows them, Up and Down go through them, Enter or a press takes one.

use crate::model::ListModel;
use crate::t;
use crate::widget::{column, entry, item, list, note, Field, Node};

pub trait Words<'a>: Copy {
    fn count(&self) -> usize;
    fn word(&self, i: usize) -> &'a str;
}

impl<'a> Words<'a> for &'a [String] {
    fn count(&self) -> usize {
        self.len()
    }
    fn word(&self, i: usize) -> &'a str {
        &self[i]
    }
}

impl<'a> Words<'a> for &'a Vec<String> {
    fn count(&self) -> usize {
        self.len()
    }
    fn word(&self, i: usize) -> &'a str {
        &self[i]
    }
}

impl<'a, const N: usize> Words<'a> for &'a [String; N] {
    fn count(&self) -> usize {
        N
    }
    fn word(&self, i: usize) -> &'a str {
        &self[i]
    }
}

pub struct Rows<'a, T> {
    model: &'a ListModel<T>,
    word: fn(&T) -> &str,
}

impl<T> Clone for Rows<'_, T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for Rows<'_, T> {}

pub fn from_model<T>(model: &ListModel<T>, word: fn(&T) -> &str) -> Rows<'_, T> {
    Rows { model, word }
}

impl<'a, T> Words<'a> for Rows<'a, T> {
    fn count(&self) -> usize {
        self.model.len()
    }
    fn word(&self, i: usize) -> &'a str {
        self.model.row(i).map_or("", self.word)
    }
}

pub const MOST: usize = 8;

#[derive(Clone, Debug, PartialEq)]
pub enum Did {
    Typed(String),
    Step(bool),
    Enter,
    Take(usize),
    Shut,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Complete {
    pub text: String,
    pub lit: Option<usize>,
    pub open: bool,
}

impl Complete {
    pub fn new(text: impl Into<String>) -> Complete {
        Complete { text: text.into(), ..Complete::default() }
    }

    pub fn shown<'a>(&self, all: impl Words<'a>) -> Vec<&'a str> {
        let typed = self.text.trim().to_lowercase();
        if typed.is_empty() || !self.open {
            return Vec::new();
        }
        let all: Vec<&str> = (0..all.count()).map(|i| all.word(i)).collect();
        let low: Vec<String> = all.iter().map(|s| s.to_lowercase()).collect();
        let starts = all.iter().zip(&low).filter(|(_, l)| l.starts_with(&typed));
        let holds = all.iter().zip(&low).filter(|(_, l)| !l.starts_with(&typed) && l.contains(&typed));
        starts.chain(holds).map(|(s, _)| *s).filter(|s| s.to_lowercase() != typed).take(MOST).collect()
    }

    pub fn update<'a>(&mut self, did: Did, all: impl Words<'a>) -> bool {
        let count = self.shown(all).len();
        match did {
            Did::Typed(text) => {
                self.text = text;
                self.open = true;
                self.lit = None;
            }
            Did::Step(down) if count > 0 => {
                self.lit = Some(match (self.lit, down) {
                    (None, true) => 0,
                    (None, false) => count - 1,
                    (Some(i), true) => (i + 1) % count,
                    (Some(i), false) => (i + count - 1) % count,
                });
            }
            Did::Step(_) => {}
            Did::Enter => return self.lit.is_some_and(|i| self.take(i, all)),
            Did::Take(i) => return self.take(i, all),
            Did::Shut => {
                self.open = false;
                self.lit = None;
            }
        }
        false
    }

    fn take<'a>(&mut self, i: usize, all: impl Words<'a>) -> bool {
        let Some(s) = self.shown(all).get(i).map(|s| s.to_string()) else { return false };
        self.text = s;
        self.open = false;
        self.lit = None;
        true
    }

    pub fn view<'a, M: Clone + 'static>(&self, key: &str, tip: &str, all: impl Words<'a>, wrap: impl Fn(Did) -> M + Clone + 'static) -> Node<M> {
        let (typed, step, enter, take) = (wrap.clone(), wrap.clone(), wrap.clone(), wrap.clone());
        let mut items = vec![entry(self.text.clone(), tip, move |s| typed(Did::Typed(s)))
            .key(key)
            .on_up_down(move |down| step(Did::Step(down)))
            .on_submit(enter(Did::Enter))];
        let shown = self.shown(all);
        if !shown.is_empty() {
            let typed = self.text.trim().to_lowercase();
            let rows = shown.iter().map(|s| {
                let at = s.to_lowercase().find(&typed).filter(|_| s.len() == s.to_lowercase().len());
                item(*s).marked(Field::Text, &at.map(|a| a..a + typed.len()).into_iter().collect::<Vec<_>>())
            }).collect();
            items.push(list(rows, self.lit, move |i| take(Did::Take(i))).key(format!("{key}-suggestions")).called(t!("kit-complete-list", name = tip)));
            items.push(note(t!("kit-complete-count", n = shown.len() as i64)).polite());
        }
        column(items).gap(crate::theme::gaps().small).on_escape(wrap(Did::Shut))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fruit() -> Vec<String> {
        ["Apple", "Apricot", "Banana", "Pineapple", "Grape"].map(String::from).to_vec()
    }

    #[test]
    fn typing_narrows_the_suggestions_starting_ones_first() {
        let mut c = Complete::new("");
        assert!(c.shown(&fruit()).is_empty(), "nothing typed, nothing shown");
        c.update(Did::Typed("ap".into()), &fruit());
        assert_eq!(c.shown(&fruit()), ["Apple", "Apricot", "Pineapple", "Grape"]);
        c.update(Did::Shut, &fruit());
        assert!(c.shown(&fruit()).is_empty());
    }

    #[test]
    fn the_list_of_suggestions_has_a_name_a_reader_can_say() {
        use crate::theme::Theme;
        use crate::ui::Ui;
        let c = Complete { text: "ap".into(), lit: None, open: true };
        let mut ui = Ui::new(c.view("fruit", "Fruit", &fruit(), |d| d), Theme::new(true, "#fbc711"));
        ui.lay_at(400, 300);
        assert!(ui.unnamed_lists().is_empty(), "{:?}", ui.unnamed_lists());
        let named = ui.access("Fruit").nodes.iter().filter(|(_, p)| p.role() == accesskit::Role::ListBox).any(|(_, p)| p.label() == Some("Suggestions for Fruit"));
        assert!(named);
    }

    #[test]
    fn up_and_down_go_round_and_enter_takes_the_lit_one() {
        let mut c = Complete::new("");
        c.update(Did::Typed("ap".into()), &fruit());
        assert!(!c.update(Did::Enter, &fruit()), "nothing lit, nothing taken");
        c.update(Did::Step(false), &fruit());
        assert_eq!(c.lit, Some(3));
        c.update(Did::Step(true), &fruit());
        c.update(Did::Step(true), &fruit());
        assert_eq!(c.lit, Some(1));
        assert!(c.update(Did::Enter, &fruit()));
        assert_eq!((c.text.as_str(), c.open), ("Apricot", false));
        c.update(Did::Typed("ban".into()), &fruit());
        assert!(c.update(Did::Take(0), &fruit()));
        assert_eq!(c.text, "Banana");
    }

    #[test]
    fn a_model_gives_the_words_without_strings_being_built() {
        struct Town {
            name: String,
        }
        let mut towns = ListModel::new(["Ely", "Eye", "Bury", "Diss"].map(|n| Town { name: n.into() }).into());
        fn words(m: &ListModel<Town>) -> Rows<'_, Town> {
            from_model(m, |t| t.name.as_str())
        }
        let mut c = Complete::new("");
        c.update(Did::Typed("y".into()), words(&towns));
        assert_eq!(c.shown(words(&towns)), ["Ely", "Eye", "Bury"]);
        towns.filter(Some(Box::new(|t: &Town| t.name != "Eye")));
        assert_eq!(c.shown(words(&towns)), ["Ely", "Bury"], "a row the model hides is not offered");
        assert!(c.update(Did::Take(1), words(&towns)));
        assert_eq!(c.text, "Bury");
        let _ = c.view("town", "Town", words(&towns), |_: Did| ());
    }
}
