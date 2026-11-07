//! Frames round what a page holds: a named frame, a fixed layout, corner badges, a revealer, and a pager with its dots.

use tiny_skia::{PathBuilder, Transform};

use super::colour::paint;
use super::theme;
use super::widget::{button, card, column, heading, layers, placed, row, slide, space, Length, Node};

pub fn frame<M>(name: impl Into<String>, inner: Node<M>) -> Node<M> {
    column(vec![heading(name), card(inner)]).gap(theme::sheet().gap)
}

pub fn fixed<M>(items: Vec<((f32, f32), Node<M>)>) -> Node<M> {
    layers(items.into_iter().map(|((x, y), n)| placed(n, move |_, _| (x, y))).collect())
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Corner {
    TopStart,
    TopEnd,
    BottomStart,
    BottomEnd,
}

pub fn badged<M>(inner: Node<M>, badge: Node<M>, corner: Corner) -> Node<M> {
    let (width, height) = (inner.width, inner.height);
    let at = move |(w, h): (f32, f32), (rw, rh): (f32, f32)| match corner {
        Corner::TopStart => (0.0, 0.0),
        Corner::TopEnd => (rw - w, 0.0),
        Corner::BottomStart => (0.0, rh - h),
        Corner::BottomEnd => (rw - w, rh - h),
    };
    layers(vec![inner, placed(badge, at)]).width(width).height(height)
}

pub fn revealer<M>(inner: Node<M>, share: f32) -> Node<M> {
    if share <= 0.0 {
        return space().width(Length::Shrink);
    }
    inner.opacity(share.min(1.0))
}

pub fn dots<M>(count: usize, current: usize, choose: impl Fn(usize) -> M) -> Node<M> {
    let k = theme::sheet();
    row((0..count).map(|i| {
        let here = i == current;
        button("", crate::t!("kit-page-of", n = i + 1, of = count)).size(k.dot_target).on(here).on_press(choose(i))
            .face(move |canvas, r, ink, pen| {
                let colour = if here { pen.theme.accent } else { ink };
                if let Some(dot) = PathBuilder::from_circle(r.x + r.w / 2.0, r.y + r.h / 2.0, k.dot * pen.scale / 2.0) {
                    canvas.fill_path(&dot, &paint(colour), tiny_skia::FillRule::Winding, Transform::identity(), None);
                }
            })
    }).collect()).gap(0.0)
}

pub fn pager<M>(pages: Vec<Node<M>>, width: f32, at: f32, choose: impl Fn(usize) -> M) -> Node<M> {
    let count = pages.len();
    let strip = row(pages.into_iter().map(|p| p.width(Length::Fixed(width))).collect()).gap(0.0);
    let current = (at.round().max(0.0) as usize).min(count.saturating_sub(1));
    column(vec![slide(strip, width, at * width), row(vec![space(), dots(count, current, choose), space()])]).gap(theme::sheet().gap)
}
