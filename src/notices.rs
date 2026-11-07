//! The bottom notice: one pill over the foot of the window that never moves the page, the next waiting its turn.

use std::collections::VecDeque;
use std::time::{Duration, Instant};

use super::pill::{band, paint_pill};
use super::theme;
use super::widget::{drawing, layers, placed, Length, Node};

const MOST_WAITING: usize = 5;

#[derive(Clone, Debug, Default)]
pub struct Notices {
    waiting: VecDeque<String>,
    shown: Option<(String, Instant)>,
}

impl Notices {
    pub fn new() -> Notices {
        Notices::default()
    }

    pub fn push(&mut self, words: impl Into<String>, now: Instant) {
        let words = words.into();
        let last = self.waiting.back().or(self.shown.as_ref().map(|s| &s.0));
        if last != Some(&words) {
            if self.waiting.len() >= MOST_WAITING {
                self.waiting.pop_front();
            }
            self.waiting.push_back(words);
        }
        self.tick(now);
    }

    pub fn tick(&mut self, now: Instant) -> bool {
        let stays = theme::sheet().notice_stays;
        if self.shown.as_ref().is_some_and(|(_, at)| now.duration_since(*at) < stays) {
            return false;
        }
        let was = self.shown.take().is_some();
        self.shown = self.waiting.pop_front().map(|w| (w, now));
        was || self.shown.is_some()
    }

    pub fn dismiss(&mut self, now: Instant) {
        self.shown = None;
        self.tick(now);
    }

    pub fn shown(&self) -> Option<&str> {
        self.shown.as_ref().map(|s| s.0.as_str())
    }

    pub fn waiting(&self) -> usize {
        self.waiting.len()
    }

    pub fn next_wake(&self, now: Instant) -> Option<Duration> {
        let (_, at) = self.shown.as_ref()?;
        Some((*at + theme::sheet().notice_stays).saturating_duration_since(now))
    }

    pub fn over<M: 'static>(&self, content: Node<M>) -> Node<M> {
        let Some(words) = self.shown().map(str::to_string) else { return content };
        let pill = drawing((0.0, band()), words.clone(), move |canvas, pen| paint_pill(canvas, pen, &words))
            .width(Length::Fill)
            .passes()
            .polite()
            .key("notice");
        layers(vec![content, placed(pill, |(_, h), (_, area)| {
            let s = theme::sheet();
            (0.0, area - h - (s.pill_foot - s.pill_room))
        })])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn notices_come_one_at_a_time_in_order() {
        let (mut n, now) = (Notices::new(), Instant::now());
        n.push("Saved", now);
        n.push("Copied", now);
        n.push("Copied", now);
        assert_eq!((n.shown(), n.waiting()), (Some("Saved"), 1), "the same words twice running are one notice");
        let stays = theme::sheet().notice_stays;
        assert_eq!(n.next_wake(now), Some(stays));
        assert!(!n.tick(now + stays / 2));
        assert!(n.tick(now + stays));
        assert_eq!(n.shown(), Some("Copied"));
        assert!(n.tick(now + stays * 2));
        assert_eq!((n.shown(), n.next_wake(now)), (None, None));
        assert!(!n.tick(now + stays * 3));
    }

    #[test]
    fn a_burst_of_notices_keeps_the_newest_few_waiting() {
        let (mut n, now) = (Notices::new(), Instant::now());
        for i in 0..40 {
            n.push(format!("Notice {i}"), now);
        }
        assert_eq!((n.shown(), n.waiting()), (Some("Notice 0"), MOST_WAITING));
        let stays = theme::sheet().notice_stays;
        n.tick(now + stays);
        assert_eq!(n.shown(), Some("Notice 35"), "the oldest waiting went, the newest stayed");
    }

    #[test]
    fn a_dismissed_notice_makes_way_for_the_next() {
        let (mut n, now) = (Notices::new(), Instant::now());
        n.push("One", now);
        n.push("Two", now);
        n.dismiss(now);
        assert_eq!(n.shown(), Some("Two"));
        n.push("Two", now);
        assert_eq!(n.waiting(), 0, "the one shown is not queued again");
    }

    #[test]
    fn with_no_notice_the_content_is_left_as_it_is() {
        let content: Node<()> = super::super::widget::label("Page").key("page");
        assert!(matches!(Notices::new().over(content).key.as_deref(), Some("page")));
    }
}
