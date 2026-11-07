//! Drawing helpers shared by the tables, tabs, crop box and cell grid: a filled rectangle and one line of cut words.

use tiny_skia::{IntRect, PixmapMut};

use super::colour::Colour;
use super::paint::Painter;
use super::text::{Fonts, Layout, Style};
use super::widget::{Pen, Rect};

pub(crate) fn fill(canvas: &mut PixmapMut, pen: &Pen, r: Rect, colour: Colour) {
    let mut p = Painter::default();
    fill_on(&mut p, pen, r, colour);
    p.play(canvas);
}

pub(crate) fn fill_on(p: &mut Painter, pen: &Pen, r: Rect, colour: Colour) {
    let k = pen.scale;
    p.fill(Rect { x: r.x * k, y: r.y * k, w: (r.w * k).max(0.0), h: (r.h * k).max(0.0) }, colour);
}

pub(crate) fn edge(p: &mut Painter, pen: &Pen, r: Rect, width: f32, colour: Colour) {
    for side in [Rect { h: width, ..r }, Rect { y: r.y + r.h - width, h: width, ..r }, Rect { w: width, ..r }, Rect { x: r.x + r.w - width, w: width, ..r }] {
        fill_on(p, pen, side, colour);
    }
}

pub(crate) fn text_in_on<'a>(p: &mut Painter<'a>, pen: &Pen<'a>, text: &str, style: &Style, r: Rect, colour: Colour, end: bool) {
    if text.is_empty() || r.w <= 0.0 {
        return;
    }
    let k = pen.scale;
    let full = pen.fonts.layout(text, style, None);
    let layout = if full.width > r.w * k { full.cut(r.w * k, &pen.fonts.layout("\u{2026}", style, None)) } else { full };
    let x = if end { r.x * k + r.w * k - layout.width } else { r.x * k };
    let y = r.y * k + (r.h * k - layout.height) / 2.0;
    words_on(p, pen.fonts, layout, x, y, colour);
}

pub(crate) fn words_on<'a>(p: &mut Painter<'a>, fonts: &'a Fonts, layout: Layout, x: f32, y: f32, colour: Colour) {
    let pad = layout.lines.first().map_or(0.0, |l| l.height);
    p.pixels(around(x, y, layout.width, layout.height, pad), move |c| fonts.draw(c, &layout, x, y, colour));
}

pub(crate) fn around(x: f32, y: f32, w: f32, h: f32, pad: f32) -> IntRect {
    let (l, t) = ((x - pad).floor() as i32, (y - pad).floor() as i32);
    let (r, b) = ((x + w + pad).ceil() as i32, (y + h + pad).ceil() as i32);
    IntRect::from_ltrb(l, t, r.max(l.saturating_add(1)), b.max(t.saturating_add(1)))
        .unwrap_or_else(|| IntRect::from_xywh(0, 0, 1, 1).expect("one pixel"))
}
