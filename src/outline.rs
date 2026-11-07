//! A table of contents for a long page: its headings as places to go, the one being read marked, for a side panel.

use super::widget::{item, list, scroll, Length, Node};

#[derive(Clone, Debug, PartialEq)]
pub struct Entry {
    pub title: String,
    pub depth: usize,
    pub key: String,
}

impl Entry {
    pub fn new(title: impl Into<String>, depth: usize, key: impl Into<String>) -> Entry {
        Entry { title: title.into(), depth, key: key.into() }
    }
}

pub fn outline<M>(key: impl Into<String>, entries: &[Entry], current: Option<usize>, choose: impl Fn(usize) -> M + 'static) -> Node<M> {
    let rows = entries.iter().map(|e| item(format!("{}{}", "\u{2002}\u{2002}".repeat(e.depth), e.title))).collect();
    scroll(key, list(rows, current, choose).here().called(crate::t!("kit-places"))).width(Length::Fill)
}
