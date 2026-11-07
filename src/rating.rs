//! Rating stars: press a star or use the arrow keys; the same star again clears it.

use super::input::{Input, Keysym};
use super::tiles::Sketch;
use super::widget::{drawing, Node, Pointer};
use crate::t;

const STAR: f32 = 22.0;

fn star(at: (f32, f32)) -> Vec<(f32, f32)> {
    (0..10).map(|i| {
        let (r, a) = (if i % 2 == 0 { 9.5 } else { 4.2 }, std::f32::consts::PI * i as f32 / 5.0 - std::f32::consts::FRAC_PI_2);
        (at.0 + r * a.cos(), at.1 + r * a.sin())
    }).collect()
}

pub fn rating<M: 'static>(value: u8, max: u8, tip: impl Into<String>, change: impl Fn(u8) -> M + Clone + 'static) -> Node<M> {
    let value = value.min(max);
    let width = STAR * max as f32;
    let (pressed, keyed) = (change.clone(), change);
    drawing((width, STAR), "", move |canvas, pen| {
        let mut s = Sketch::fit(canvas, pen, (width, STAR));
        for i in 0..max {
            let c = if i < value { s.highlight() } else { s.track() };
            s.shape(&star((STAR * (i as f32 + 0.5), STAR / 2.0)), c);
        }
    })
    .called(t!("kit-rating", name = tip.into(), value = value as i64, max = max as i64))
    .on_pointer_when(move |p| match p {
        Pointer::Down(x, _) => {
            let n = ((x / STAR).floor() as i64 + 1).clamp(1, (max as i64).max(1)) as u8;
            Some(pressed(if n == value { 0 } else { n }))
        }
        _ => None,
    })
    .on_keys(move |input| {
        let Input::Key { sym, .. } = input else { return None };
        let to = match *sym {
            Keysym::Left | Keysym::Down => value.saturating_sub(1),
            Keysym::Right | Keysym::Up => (value + 1).min(max),
            Keysym::Home => 0,
            Keysym::End => max,
            _ => return None,
        };
        Some(keyed(to))
    })
}
