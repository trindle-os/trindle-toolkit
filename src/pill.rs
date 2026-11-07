//! The notice's pill for any window to paint, and the soft shadow of things that float.

use tiny_skia::PixmapMut;

use super::draw::words_on;
use super::paint::Painter;
use super::text::Style;
use super::theme::Fall;
use super::widget::{Pen, Rect};

pub const HEIGHT: f32 = 34.0;
pub const WIDEST: f32 = 640.0;
pub const ROOM: f32 = 40.0;
pub const FOOT: f32 = 48.0;
pub const BAND: f32 = HEIGHT + 2.0 * ROOM;

pub fn band() -> f32 {
    let s = crate::theme::sheet();
    s.pill_height + 2.0 * s.pill_room
}

pub fn float_shadow(canvas: &mut PixmapMut, pen: &Pen, r: Rect, radius: f32) {
    let mut p = Painter::default();
    float_shadow_on(&mut p, pen, r, radius);
    p.play(canvas);
}

pub fn float_shadow_on(p: &mut Painter, pen: &Pen, r: Rect, radius: f32) {
    let k = pen.scale;
    let falls: Vec<Fall> = pen.theme.float.iter().map(|f| Fall { y: f.y * k, blur: f.blur * k, spread: f.spread * k, alpha: f.alpha }).collect();
    if !falls.is_empty() {
        p.shadow(r, radius, &falls, pen.theme.shadow);
    }
}

pub fn paint_pill(canvas: &mut PixmapMut, pen: &Pen, words: &str) {
    let mut p = Painter::default();
    paint_pill_on(&mut p, pen, (canvas.width(), canvas.height()), words);
    p.play(canvas);
}

pub fn paint_pill_on<'a>(p: &mut Painter<'a>, pen: &Pen<'a>, size: (u32, u32), words: &str) {
    let s = pen.scale;
    let sh = crate::theme::sheet();
    let mut style = Style::new((sh.pill_words * s).round());
    style.weight = 600;
    let dots = pen.fonts.layout("\u{2026}", &style, None);
    let room = ((sh.pill_widest - 2.0 * sh.pill_side) * s).min(size.0 as f32 - 2.0 * (sh.pill_room + sh.pill_side) * s).max(1.0);
    let line = pen.fonts.layout(words, &style, None).cut(room, &dots);
    let (w, h) = ((line.width + 2.0 * sh.pill_side * s).round(), (sh.pill_height * s).round());
    let (x, y) = (((size.0 as f32 - w) / 2.0).round(), ((size.1 as f32 - h) / 2.0).round());
    let pill = Rect { x, y, w, h };
    let corner = crate::theme::corner_px(crate::theme::Part::Tooltip, h, s);
    float_shadow_on(p, pen, pill, corner);
    p.round(pill, [corner; 4], pen.theme.raised);
    let ty = y + ((h - line.height) / 2.0).round();
    words_on(p, pen.fonts, line, x + sh.pill_side * s, ty, pen.theme.fg);
}
