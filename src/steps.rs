//! Wizard pages: the step you are on ("Step 2 of 5"), a strip of the steps done, the page, and Back and Next.

use super::tiles::Sketch;
use super::widget::{column, drawing, heading, main_word, note, row, space, word_button, Length, Node};
use crate::t;

const SEGMENT: f32 = 6.0;
const GAP: f32 = 4.0;

pub fn wizard<M: Clone + 'static>(at: usize, titles: &[String], page: Node<M>, back: M, forward: M) -> Node<M> {
    let count = titles.len().max(1);
    let at = at.min(count - 1);
    let strip = drawing((1.0, SEGMENT), "", move |canvas, pen| {
        let mut s = Sketch::strip(canvas, pen, SEGMENT);
        let wide = s.wide();
        let each = (wide - GAP * (count as f32 - 1.0)) / count as f32;
        for i in 0..count {
            let c = if i <= at { s.highlight() } else { s.track() };
            s.rect((i as f32 * (each + GAP), 0.0, each, SEGMENT), SEGMENT / 2.0, c);
        }
    }).width(Length::Fill).height(Length::Fixed(SEGMENT));
    let last = at + 1 == count;
    let first = if at == 0 { word_button(t!("kit-step-back")).off(true) } else { word_button(t!("kit-step-back")).on_press(back) };
    let next = if last { main_word(t!("kit-step-finish")) } else { main_word(t!("kit-step-next")) };
    column(vec![
        note(t!("kit-step-of", n = at as i64 + 1, count = count as i64)),
        strip,
        heading(titles.get(at).cloned().unwrap_or_default()),
        page,
        row(vec![first, space(), next.on_press(forward)]).gap(crate::theme::gaps().medium),
    ]).gap(crate::theme::gaps().roomy)
}
