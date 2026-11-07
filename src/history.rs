//! Undo and redo across a window: the steps a program keeps, newest last, with typing in one box joined into one step.

use std::time::{Duration, Instant};

pub const JOIN: Duration = Duration::from_secs(1);

#[derive(Clone, Debug)]
pub struct History<T> {
    done: Vec<T>,
    undone: Vec<T>,
    most: usize,
    typing: Option<(String, Instant)>,
}

impl<T> History<T> {
    pub fn new(most: usize) -> Self {
        History { done: Vec::new(), undone: Vec::new(), most: most.max(1), typing: None }
    }

    pub fn push(&mut self, step: T) {
        self.typing = None;
        self.keep(step);
    }

    pub fn typed(&mut self, place: &str, step: T, now: Instant, join: impl FnOnce(&mut T, T)) {
        let joins = self.undone.is_empty() && self.typing.as_ref()
            .is_some_and(|(p, at)| p == place && now.saturating_duration_since(*at) < JOIN);
        match self.done.last_mut() {
            Some(last) if joins => join(last, step),
            _ => self.keep(step),
        }
        self.typing = Some((place.to_string(), now));
    }

    fn keep(&mut self, step: T) {
        self.undone.clear();
        self.done.push(step);
        if self.done.len() > self.most {
            self.done.remove(0);
        }
    }

    pub fn undo(&mut self) -> Option<&T> {
        self.typing = None;
        let step = self.done.pop()?;
        self.undone.push(step);
        self.undone.last()
    }

    pub fn redo(&mut self) -> Option<&T> {
        self.typing = None;
        let step = self.undone.pop()?;
        self.done.push(step);
        self.done.last()
    }

    pub fn can_undo(&self) -> bool {
        !self.done.is_empty()
    }

    pub fn can_redo(&self) -> bool {
        !self.undone.is_empty()
    }

    pub fn clear(&mut self) {
        self.done.clear();
        self.undone.clear();
        self.typing = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn steps_undo_and_redo_in_order_and_a_new_step_forgets_the_undone() {
        let mut h = History::new(10);
        assert!(!h.can_undo() && h.undo().is_none());
        h.push("a");
        h.push("b");
        assert_eq!(h.undo(), Some(&"b"));
        assert!(h.can_redo());
        assert_eq!(h.redo(), Some(&"b"));
        assert_eq!(h.redo(), None);
        h.undo();
        h.push("c");
        assert!(!h.can_redo());
        assert_eq!(h.undo(), Some(&"c"));
        assert_eq!(h.undo(), Some(&"a"));
        assert_eq!(h.undo(), None);
    }

    #[test]
    fn only_the_newest_steps_are_kept() {
        let mut h = History::new(2);
        for s in [1, 2, 3] {
            h.push(s);
        }
        assert_eq!((h.undo().copied(), h.undo().copied(), h.undo().copied()), (Some(3), Some(2), None));
    }

    #[test]
    fn quick_typing_in_one_box_is_one_step() {
        let mut h: History<String> = History::new(10);
        let now = Instant::now();
        let join = |last: &mut String, s: String| last.push_str(&s);
        h.typed("name", "J".into(), now, join);
        h.typed("name", "o".into(), now + Duration::from_millis(300), join);
        h.typed("mail", "x".into(), now + Duration::from_millis(400), join);
        h.typed("mail", "y".into(), now + Duration::from_secs(3), join);
        assert_eq!(h.undo().cloned().as_deref(), Some("y"));
        assert_eq!(h.undo().cloned().as_deref(), Some("x"));
        assert_eq!(h.undo().cloned().as_deref(), Some("Jo"));
        h.redo();
        h.typed("name", "e".into(), now + Duration::from_millis(500), join);
        assert_eq!(h.undo().cloned().as_deref(), Some("e"));
    }
}
