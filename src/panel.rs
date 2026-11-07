//! A side panel beside the content that slides in and out as the program says, never a fold or a pop-up.

use super::widget::{row, slide, Length, Node};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Side {
    Start,
    End,
}

pub fn side_panel<M: 'static>(content: Node<M>, panel: Node<M>, width: f32, shown: f32, side: Side) -> Node<M> {
    let shown = shown.clamp(0.0, 1.0);
    if shown <= 0.0 {
        return content;
    }
    let seen = (width * shown).round();
    let by = if side == Side::Start { width - seen } else { 0.0 };
    let panel = slide(panel.width(Length::Fixed(width)).height(Length::Fill), seen, by).height(Length::Fill);
    let content = content.width(Length::Fill).height(Length::Fill);
    let items = if side == Side::Start { vec![panel, content] } else { vec![content, panel] };
    row(items).gap((crate::theme::gaps().large * shown).round()).height(Length::Fill)
}
