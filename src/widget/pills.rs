//! How wide each pill of a choice is, and which pill a point falls on.

use crate::text::Fonts;
use crate::theme;

use super::{ARROW, ARROW_GAP, Sub, Texts, Words};

pub(crate) fn pill_widths(fonts: &Fonts, texts: &Texts, options: &[String], arrows: Option<&[Sub]>, scale: f32) -> Vec<f32> {
    let style = Words::Plain.style(scale);
    let pad = (theme::sheet().entry_padding * scale).round();
    let mark = ((ARROW + ARROW_GAP) * scale).round();
    options.iter().enumerate().map(|(i, o)| {
        let l = texts.get(fonts, o, &style, None);
        let arrow = arrows.is_some_and(|a| has_arrow(a, i));
        l.widest().ceil() + 2.0 * pad + if arrow { mark } else { 0.0 }
    }).collect()
}

pub(crate) fn has_arrow(subs: &[Sub], i: usize) -> bool {
    !matches!(subs.get(i), None | Some(Sub::None))
}

pub(crate) fn pills_in(mut widths: Vec<f32>, room: f32) -> Vec<f32> {
    let all: f32 = widths.iter().sum();
    if all > room && all > 0.0 {
        let k = room.max(0.0) / all;
        for w in &mut widths {
            *w = (*w * k).floor();
        }
    }
    widths
}

pub(crate) fn pill_at(widths: &[f32], x: f32) -> Option<usize> {
    let mut left = 0.0;
    for (i, w) in widths.iter().enumerate() {
        if x >= left && x < left + w {
            return Some(i);
        }
        left += w;
    }
    None
}
