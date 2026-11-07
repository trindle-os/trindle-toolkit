//! An info panel that stands in the page where the news belongs, in place of a banner or a toast: an icon, the words and any buttons, spoken when it appears.

use super::widget::{card, icon_image, label, row, Length, Node, Tone};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Note,
    Done,
    Warning,
    Failed,
}

impl Kind {
    fn icon(self) -> (&'static str, Tone) {
        match self {
            Kind::Note => ("dialog-information-symbolic", Tone::Plain),
            Kind::Done => ("emblem-ok-symbolic", Tone::Good),
            Kind::Warning => ("dialog-warning-symbolic", Tone::Warn),
            Kind::Failed => ("dialog-error-symbolic", Tone::Problem),
        }
    }
}

pub fn info_panel<M>(kind: Kind, words: impl Into<String>, buttons: Vec<Node<M>>) -> Node<M> {
    let (icon, tone) = kind.icon();
    let said = label(words).tone(if kind == Kind::Failed { Tone::Problem } else { Tone::Plain }).wraps().width(Length::Fill);
    let said = if kind == Kind::Failed { said } else { said.polite() };
    let mut items = vec![icon_image(icon, crate::theme::sheet().info_icon, tone), said];
    items.extend(buttons);
    card(row(items).gap(crate::theme::gaps().roomy))
}
