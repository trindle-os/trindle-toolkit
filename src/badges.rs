//! Badge counts and groups of avatars that end in "and so many more".

use std::rc::Rc;

use super::tiles::Sketch;
use super::widget::{avatar, drawing, row, Length, Node};
use crate::t;

const TALL: f32 = 20.0;

fn count_box<M>(words: String, name: String) -> Node<M> {
    let wide = (TALL + 8.0 * words.chars().count().saturating_sub(1) as f32).max(TALL);
    drawing((wide, TALL), "", move |canvas, pen| {
        let mut s = Sketch::fit(canvas, pen, (wide, TALL));
        let back = s.highlight();
        s.rect((0.0, 0.0, wide, TALL), TALL / 2.0, back);
        let ink = s.words_on(back);
        s.text(&words, (wide / 2.0, TALL / 2.0), 12.0, ink);
    }).called(name)
}

pub fn badge_count<M>(n: usize, name: impl Into<String>) -> Node<M> {
    count_box(if n > 99 { "99+".to_string() } else { n.to_string() }, name.into())
}

pub fn avatar_group<M: 'static>(people: Vec<(Option<Rc<image::DynamicImage>>, String)>, size: f32, most: usize) -> Node<M> {
    let more = people.len().saturating_sub(most);
    let mut all: Vec<Node<M>> = people.into_iter().take(most).map(|(picture, name)| avatar(picture, name, size)).collect();
    if more > 0 {
        all.push(count_box(format!("+{more}"), t!("kit-more-people", n = more as i64)));
    }
    row(all).gap(crate::theme::gaps().small).centred().width(Length::Shrink)
}
