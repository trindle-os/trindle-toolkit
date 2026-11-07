//! Following a long page for its outline: which heading is being read, and going to one.

use super::*;

const READING: f32 = 24.0;

impl<M: Clone> Ui<M> {
    pub fn reading(&mut self, keys: &[&str]) -> Option<usize> {
        self.settle();
        let line = |key: &str| {
            let s = self.spot(key)?;
            let area = self.areas_round(key).pop().and_then(|a| self.spot(&a)).map_or(0.0, |a| a.rect.y);
            Some(s.rect.y - area)
        };
        let tops: Vec<Option<f32>> = keys.iter().map(|k| line(k)).collect();
        let first = tops.iter().position(Option::is_some)?;
        Some(tops.iter().rposition(|t| t.is_some_and(|y| y <= self.px(READING))).unwrap_or(first))
    }

    pub fn go_to(&mut self, key: &str) -> bool {
        self.settle();
        let Some(top) = self.spot(key).map(|s| s.rect.y) else { return false };
        let Some(area) = self.areas_round(key).pop() else { return false };
        let Some(a) = self.spot(&area) else { return false };
        let to = (self.offset(a) + top - a.rect.y).clamp(0.0, (a.full - a.rect.h).max(0.0));
        self.scroll_area(&area, to)
    }
}
