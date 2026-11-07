//! Named actions a program keeps once and shows anywhere: words, an icon, a shortcut and whether it can be done now, shared by a menu, a toolbar and the command palette.

use super::input::Input;
use super::shortcuts::Chord;
use super::widget::{button, row, MenuRow, Node};

#[derive(Clone, Debug, PartialEq)]
pub struct Action<A> {
    pub id: A,
    pub label: String,
    pub icon: String,
    pub chord: Option<Chord>,
    pub enabled: bool,
}

impl<A> Action<A> {
    pub fn new(id: A, label: impl Into<String>) -> Self {
        Action { id, label: label.into(), icon: String::new(), chord: None, enabled: true }
    }

    pub fn icon(mut self, name: impl Into<String>) -> Self {
        self.icon = name.into();
        self
    }

    pub fn keys(mut self, chord: &str) -> Self {
        self.chord = Chord::parse(chord);
        self
    }

    pub fn keys_text(&self) -> String {
        self.chord.map(|c| c.text()).unwrap_or_default()
    }
}

#[derive(Clone, Debug)]
pub struct Actions<A> {
    list: Vec<Action<A>>,
}

impl<A> Default for Actions<A> {
    fn default() -> Self {
        Actions { list: Vec::new() }
    }
}

impl<A: Clone + PartialEq> Actions<A> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add(&mut self, action: Action<A>) -> Result<(), A> {
        if let Some(other) = self.list.iter().find(|a| a.id != action.id && a.chord.is_some() && a.chord == action.chord) {
            return Err(other.id.clone());
        }
        match self.list.iter_mut().find(|a| a.id == action.id) {
            Some(old) => *old = action,
            None => self.list.push(action),
        }
        Ok(())
    }

    pub fn get(&self, id: &A) -> Option<&Action<A>> {
        self.list.iter().find(|a| a.id == *id)
    }

    pub fn enabled(&self, id: &A) -> bool {
        self.get(id).is_some_and(|a| a.enabled)
    }

    pub fn set_enabled(&mut self, id: &A, yes: bool) {
        if let Some(a) = self.list.iter_mut().find(|a| a.id == *id) {
            a.enabled = yes;
        }
    }

    pub fn all(&self) -> impl Iterator<Item = &Action<A>> {
        self.list.iter()
    }

    pub fn find(&self, input: &Input) -> Option<A> {
        self.list.iter().find(|a| a.enabled && a.chord.is_some_and(|c| c.pressed(input))).map(|a| a.id.clone())
    }

    pub fn menu<M>(&self, ids: &[A], wrap: impl Fn(A) -> M) -> Vec<MenuRow<M>> {
        ids.iter().filter_map(|id| self.get(id)).filter(|a| a.enabled)
            .map(|a| MenuRow::send(a.icon.clone(), a.label.clone(), wrap(a.id.clone())).keys(a.keys_text()))
            .collect()
    }

    pub fn toolbar<M>(&self, ids: &[A], wrap: impl Fn(A) -> M) -> Node<M> {
        row(ids.iter().filter_map(|id| self.get(id))
            .map(|a| button(a.icon.clone(), a.label.clone()).on_press(wrap(a.id.clone())).off(!a.enabled))
            .collect()).gap(crate::theme::gaps().medium)
    }

    pub fn search(&self, query: &str) -> Vec<&Action<A>> {
        let query = query.trim().to_lowercase();
        let words: Vec<&str> = query.split_whitespace().collect();
        let mut found: Vec<(bool, &Action<A>)> = self.list.iter().filter(|a| a.enabled)
            .map(|a| (a.label.to_lowercase(), a))
            .filter(|(l, _)| words.iter().all(|w| l.contains(w)))
            .map(|(l, a)| (!l.starts_with(&query), a))
            .collect();
        found.sort_by_key(|(later, _)| *later);
        found.into_iter().map(|(_, a)| a).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::input::{Keysym, Mods};

    fn key(sym: Keysym, ctrl: bool, repeat: bool) -> Input {
        Input::Key { sym, text: None, mods: Mods { ctrl, ..Mods::default() }, repeat }
    }

    fn three() -> Actions<&'static str> {
        let mut all = Actions::new();
        all.add(Action::new("save", "Save").icon("document-save-symbolic").keys("Ctrl+S")).unwrap();
        all.add(Action::new("open", "Open a file").icon("document-open-symbolic").keys("Ctrl+O")).unwrap();
        all.add(Action::new("close", "Close").keys("Ctrl+W")).unwrap();
        all
    }

    #[test]
    fn a_shortcut_finds_its_action_only_while_enabled_and_not_on_repeat() {
        let mut all = three();
        assert_eq!(all.find(&key(Keysym::s, true, false)), Some("save"));
        assert_eq!(all.find(&key(Keysym::s, true, true)), None);
        assert_eq!(all.find(&key(Keysym::s, false, false)), None);
        all.set_enabled(&"save", false);
        assert_eq!(all.find(&key(Keysym::s, true, false)), None);
        assert!(!all.enabled(&"save") && all.enabled(&"open") && !all.enabled(&"nothing"));
    }

    #[test]
    fn a_taken_shortcut_is_refused_and_the_same_id_is_replaced() {
        let mut all = three();
        assert_eq!(all.add(Action::new("print", "Print").keys("Ctrl+S")), Err("save"));
        assert!(all.get(&"print").is_none());
        all.add(Action::new("save", "Save now").keys("Ctrl+S")).unwrap();
        assert_eq!((all.all().count(), all.get(&"save").unwrap().label.as_str()), (3, "Save now"));
        assert_eq!(Action::new(1, "x").keys("Ctrl+Nonsense+Q").chord, None);
    }

    #[test]
    fn a_menu_leaves_out_what_cannot_be_done_and_shows_the_keys() {
        let mut all = three();
        all.set_enabled(&"open", false);
        let rows = all.menu(&["save", "open", "close"], |id| id);
        assert_eq!(rows.iter().map(|r| (r.words.as_str(), r.keys.as_str())).collect::<Vec<_>>(), vec![("Save", "Ctrl+S"), ("Close", "Ctrl+W")]);
    }

    #[test]
    fn a_toolbar_greys_out_what_cannot_be_done() {
        let mut all = three();
        all.set_enabled(&"open", false);
        let bar = all.toolbar(&["save", "open"], |id| id);
        assert_eq!(bar.buttons(), vec![("Save".to_string(), true), ("Open a file".to_string(), false)]);
    }

    #[test]
    fn search_matches_every_word_and_puts_starts_first() {
        let mut all = three();
        all.add(Action::new("reopen", "Reopen closed tab")).unwrap();
        let labels = |all: &Actions<&str>, q| all.search(q).iter().map(|a| a.label.clone()).collect::<Vec<_>>();
        assert_eq!(labels(&all, "clo"), vec!["Close", "Reopen closed tab"]);
        assert_eq!(labels(&all, "TAB reopen"), vec!["Reopen closed tab"]);
        assert_eq!(labels(&all, "").len(), 4);
        all.set_enabled(&"close", false);
        assert_eq!(labels(&all, "clo"), vec!["Reopen closed tab"]);
    }
}
