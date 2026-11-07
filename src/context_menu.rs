//! A context menu for any widget: the rows a right click, a held finger, the Menu key or Shift+F10 inside it opens.

use crate::widget::{MenuRow, Node};

impl<M> Node<M> {
    pub fn context_menu(mut self, rows: impl Fn() -> Vec<MenuRow<M>> + 'static) -> Self {
        self.context = Some(Box::new(rows));
        self
    }
}
