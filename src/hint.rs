//! The "i" button of a card and the help that covers the card when it is on; Escape or the button again puts it away.

use super::widget::{button, card, column, label, layers, placed, Length, Node};
use crate::t;

pub fn help_over<M: Clone + 'static>(content: Node<M>, help: impl Into<String>, open: bool, toggle: M) -> Node<M> {
    let tip = if open { t!("kit-help-hide") } else { t!("kit-help-show") };
    let i = placed(button("dialog-information-symbolic", tip).on(open).on_press(toggle.clone()), |(w, _), (area, _)| (area - w - 8.0, 8.0));
    if open {
        let words = card(column(vec![label(help)]).padding(crate::theme::gaps().roomy)).width(Length::Fill).height(Length::Fill);
        layers(vec![content, words, i]).on_escape(toggle)
    } else {
        layers(vec![content, i])
    }
}
