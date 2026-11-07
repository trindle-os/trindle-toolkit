//! Plays a recorded list onto tiny-skia with the same calls the widgets make drawing directly.

use tiny_skia::{FillRule, IntRect, Mask, Path, PathBuilder, Pixmap, PixmapMut, PixmapPaint, Stroke, Transform};

use super::list::{How, Op};
use crate::colour::paint;
use crate::shadow::Shadows;
use crate::widget::Rect;

struct Layer {
    pix: Option<Pixmap>,
    at: IntRect,
    opacity: f32,
    radius: f32,
}

pub(crate) fn play(ops: Vec<Op>, canvas: &mut PixmapMut, shadows: &Shadows) {
    let mut layers: Vec<Layer> = Vec::new();
    let mut laid = Vec::new();
    for op in ops {
        match op {
            Op::Push { at, opacity, radius } => {
                laid.push(top(&mut layers, canvas).map(|on| crate::theme::laid_over(&on, (at.x(), at.y(), at.width(), at.height()), opacity)));
                layers.push(Layer { pix: Pixmap::new(at.width(), at.height()), at, opacity, radius });
            }
            Op::Pop => {
                pop(&mut layers, canvas);
                laid.pop();
            }
            op => {
                if let Some(mut on) = top(&mut layers, canvas) {
                    draw(&mut on, op, shadows);
                }
            }
        }
    }
    while !layers.is_empty() {
        pop(&mut layers, canvas);
        laid.pop();
    }
}

fn top<'c>(layers: &'c mut [Layer], canvas: &'c mut PixmapMut) -> Option<PixmapMut<'c>> {
    match layers.last_mut() {
        Some(l) => l.pix.as_mut().map(Pixmap::as_mut),
        None => {
            let (w, h) = (canvas.width(), canvas.height());
            PixmapMut::from_bytes(canvas.data_mut(), w, h)
        }
    }
}

fn pop(layers: &mut Vec<Layer>, canvas: &mut PixmapMut) {
    let Some(Layer { pix: Some(pix), at, opacity, radius }) = layers.pop() else { return };
    let Some(mut on) = top(layers, canvas) else { return };
    let mask = (radius > 0.0).then(|| {
        let mut m = Mask::new(on.width(), on.height())?;
        let r = Rect { x: at.x() as f32, y: at.y() as f32, w: pix.width() as f32, h: pix.height() as f32 };
        m.fill_path(&crate::ui::rounded(r, radius)?, FillRule::Winding, true, Transform::identity());
        Some(m)
    }).flatten();
    if opacity >= 1.0 && mask.is_none() {
        lay_flat(&mut on, at, &pix);
        return;
    }
    let paint = PixmapPaint { opacity, ..PixmapPaint::default() };
    on.draw_pixmap(at.x(), at.y(), pix.as_ref(), &paint, Transform::identity(), mask.as_ref());
}

fn lay_flat(on: &mut PixmapMut, at: IntRect, pix: &Pixmap) {
    let (cw, ch) = (on.width() as i32, on.height() as i32);
    let (x0, y0) = (at.x().max(0), at.y().max(0));
    let (x1, y1) = ((at.x() + at.width() as i32).min(cw), (at.y() + at.height() as i32).min(ch));
    if x1 <= x0 || y1 <= y0 {
        return;
    }
    let (n, from) = ((x1 - x0) as usize * 4, (x0 - at.x()) as usize * 4);
    let (src, dst) = (pix.data(), on.data_mut());
    for y in y0..y1 {
        let s = &src[(y - at.y()) as usize * pix.width() as usize * 4 + from..][..n];
        let d = &mut dst[(y as usize * cw as usize + x0 as usize) * 4..][..n];
        for (s, d) in s.chunks_exact(4).zip(d.chunks_exact_mut(4)) {
            match s[3] {
                0 => {}
                255 => d.copy_from_slice(s),
                a => {
                    let keep = 255 - u32::from(a);
                    for (s, d) in s.iter().zip(d.iter_mut()) {
                        *d = (u32::from(*s) + (u32::from(*d) * keep + 127) / 255) as u8;
                    }
                }
            }
        }
    }
}

fn draw(on: &mut PixmapMut, op: Op, shadows: &Shadows) {
    let id = Transform::identity();
    match op {
        Op::Fill { r, colour } => {
            if let Some(r) = tiny_skia::Rect::from_xywh(r.x, r.y, r.w, r.h) {
                on.fill_rect(r, &paint(colour), id, None);
            }
        }
        Op::Round { r, radii, colour } => {
            if let Some(path) = corners(r, radii) {
                on.fill_path(&path, &paint(colour), FillRule::Winding, id, None);
            }
        }
        Op::Line { from, to, width, colour } => {
            let mut b = PathBuilder::new();
            b.move_to(from.0, from.1);
            b.line_to(to.0, to.1);
            if let Some(path) = b.finish() {
                on.stroke_path(&path, &paint(colour), &Stroke { width, ..Stroke::default() }, id, None);
            }
        }
        Op::Path { path, how: How::Fill(rule), colour } => on.fill_path(&path, &paint(colour), rule, id, None),
        Op::Path { path, how: How::Stroke(stroke), colour } => on.stroke_path(&path, &paint(colour), &stroke, id, None),
        Op::Glyphs { run, colour, table } => {
            for g in run {
                crate::sharp::blit(on, &g.mask, g.x, g.y, colour, &table);
            }
        }
        Op::Image { pixmap, x, y, .. } => on.draw_pixmap(x, y, pixmap.as_ref().as_ref(), &PixmapPaint::default(), id, None),
        Op::Shadow { r, radius, falls, colour } => shadows.below(on, r, radius, &falls, colour),
        Op::Inside { r, radius, fall, colour } => shadows.inside(on, r, radius, fall, colour),
        Op::Pixels { draw, .. } => draw(on),
        Op::Laid { x, y, pixmap } => on.draw_pixmap(x, y, pixmap.as_ref(), &PixmapPaint::default(), Transform::identity(), None),
        Op::Push { .. } | Op::Pop => {}
    }
}

pub(super) fn corners(r: Rect, radii: [f32; 4]) -> Option<Path> {
    let k = radii.map(|v| v.min(r.w / 2.0).min(r.h / 2.0).max(0.0));
    let c = k.map(|k| k * (1.0 - 0.552_284_8));
    let (x0, y0, x1, y1) = (r.x, r.y, r.x + r.w, r.y + r.h);
    let mut p = PathBuilder::new();
    p.move_to(x0 + k[0], y0);
    p.line_to(x1 - k[1], y0);
    p.cubic_to(x1 - c[1], y0, x1, y0 + c[1], x1, y0 + k[1]);
    p.line_to(x1, y1 - k[2]);
    p.cubic_to(x1, y1 - c[2], x1 - c[2], y1, x1 - k[2], y1);
    p.line_to(x0 + k[3], y1);
    p.cubic_to(x0 + c[3], y1, x0, y1 - c[3], x0, y1 - k[3]);
    p.line_to(x0, y0 + k[0]);
    p.cubic_to(x0, y0 + c[0], x0 + c[0], y0, x0 + k[0], y0);
    p.close();
    p.finish()
}

#[cfg(test)]
mod tests {
    use std::rc::Rc;

    use super::super::{Glyph, Painter};
    use super::*;
    use crate::colour::Colour;
    use crate::sharp::{Hinting, MaskKey, Sharp};
    use crate::theme::Fall;

    const SIDE: u32 = 96;
    const INK: Colour = Colour { r: 0.2, g: 0.5, b: 0.9, a: 0.7 };

    fn ground() -> Pixmap {
        let mut p = Pixmap::new(SIDE, SIDE).unwrap();
        p.fill(tiny_skia::Color::from_rgba8(240, 230, 200, 255));
        let half = tiny_skia::Rect::from_xywh(SIDE as f32 / 2.0, 0.0, SIDE as f32 / 2.0, SIDE as f32).unwrap();
        let mut clear = tiny_skia::Paint::default();
        clear.set_color(tiny_skia::Color::from_rgba8(30, 40, 50, 90));
        clear.blend_mode = tiny_skia::BlendMode::Source;
        p.fill_rect(half, &clear, Transform::identity(), None);
        p
    }

    fn same(record: impl FnOnce(&mut Painter), direct: impl FnOnce(&mut PixmapMut)) {
        let (mut played, mut drawn) = (ground(), ground());
        let mut painter = Painter::default();
        record(&mut painter);
        play(painter.ops, &mut played.as_mut(), &Shadows::default());
        direct(&mut drawn.as_mut());
        assert!(drawn.data() != ground().data(), "the direct drawing drew nothing");
        assert!(played.data() == drawn.data(), "the played list differs from the direct drawing");
    }

    fn r(x: f32, y: f32, w: f32, h: f32) -> Rect {
        Rect { x, y, w, h }
    }

    fn triangle() -> Path {
        let mut b = PathBuilder::new();
        b.move_to(10.3, 80.0);
        b.line_to(70.6, 12.2);
        b.line_to(88.0, 70.5);
        b.close();
        b.finish().unwrap()
    }

    fn at(x: i32, y: i32, w: u32, h: u32) -> IntRect {
        IntRect::from_xywh(x, y, w, h).unwrap()
    }

    #[test]
    fn a_layer_laid_flat_gives_the_bytes_tiny_skia_gives() {
        let lay = |src: &Pixmap, ground: &Pixmap| {
            let (mut flat, mut skia) = (ground.clone(), ground.clone());
            lay_flat(&mut flat.as_mut(), at(0, 0, 256, 256), src);
            skia.draw_pixmap(0, 0, src.as_ref(), &PixmapPaint::default(), Transform::identity(), None);
            (flat, skia)
        };
        for alpha in 0..=255u32 {
            for canvas in [255u32, 200, 128, 7, 0] {
                let (mut src, mut ground) = (Pixmap::new(256, 256).unwrap(), Pixmap::new(256, 256).unwrap());
                for y in 0..256u32 {
                    for x in 0..256u32 {
                        let i = ((y * 256 + x) * 4) as usize;
                        let colour = x.min(alpha) as u8;
                        src.data_mut()[i..i + 4].copy_from_slice(&[colour, colour, colour, alpha as u8]);
                        let under = y.min(canvas) as u8;
                        ground.data_mut()[i..i + 4].copy_from_slice(&[under, under, under, canvas as u8]);
                    }
                }
                let (flat, skia) = lay(&src, &ground);
                assert!(flat.data() == skia.data(), "differs at layer alpha {alpha}, canvas alpha {canvas}");
            }
        }
    }

    #[test]
    fn a_filled_rectangle_plays_as_drawn() {
        let b = r(3.3, 4.6, 70.2, 30.5);
        same(|p| p.fill(b, INK), |c| {
            c.fill_rect(tiny_skia::Rect::from_xywh(b.x, b.y, b.w, b.h).unwrap(), &paint(INK), Transform::identity(), None)
        });
    }

    #[test]
    fn a_rounded_rectangle_plays_as_ui_rounded_draws() {
        let b = r(6.4, 8.2, 80.0, 50.7);
        same(|p| p.round(b, [9.5; 4], INK), |c| {
            c.fill_path(&crate::ui::rounded(b, 9.5).unwrap(), &paint(INK), FillRule::Winding, Transform::identity(), None)
        });
    }

    #[test]
    fn each_corner_takes_its_own_radius() {
        let mut canvas = Pixmap::new(SIDE, SIDE).unwrap();
        let mut painter = Painter::default();
        painter.round(r(0.0, 0.0, 96.0, 96.0), [0.0, 40.0, 0.0, 40.0], INK);
        play(painter.ops, &mut canvas.as_mut(), &Shadows::default());
        let alpha = |x: u32, y: u32| canvas.pixel(x, y).unwrap().alpha();
        assert!(alpha(0, 0) > 0 && alpha(95, 95) > 0);
        assert_eq!((alpha(95, 0), alpha(0, 95)), (0, 0));
    }

    #[test]
    fn a_line_plays_as_drawn() {
        same(|p| p.line((5.5, 7.0), (90.2, 60.3), 2.5, INK), |c| {
            let mut b = PathBuilder::new();
            b.move_to(5.5, 7.0);
            b.line_to(90.2, 60.3);
            let stroke = Stroke { width: 2.5, ..Stroke::default() };
            c.stroke_path(&b.finish().unwrap(), &paint(INK), &stroke, Transform::identity(), None);
        });
    }

    #[test]
    fn a_filled_and_a_stroked_path_play_as_drawn() {
        same(|p| p.fill_path(triangle(), FillRule::EvenOdd, INK), |c| {
            c.fill_path(&triangle(), &paint(INK), FillRule::EvenOdd, Transform::identity(), None)
        });
        let stroke = Stroke { width: 3.2, ..Stroke::default() };
        same(|p| p.stroke_path(triangle(), stroke.clone(), INK), |c| {
            c.stroke_path(&triangle(), &paint(INK), &stroke, Transform::identity(), None)
        });
    }

    #[test]
    fn a_glyph_run_plays_as_blit_draws() {
        let sharp = Sharp::default();
        let data = include_bytes!("../../share/fonts/NotoSans-Regular.ttf");
        let hinting = Hinting::default();
        let run = || -> Vec<Glyph> {
            (36..44u16).enumerate().filter_map(|(n, glyph)| {
                let key = MaskKey { font: fontdb::ID::dummy(), glyph, size: 18.0f32.to_bits(), weight: 400, lean: false };
                let mask = sharp.mask(data, 0, "Noto Sans", &hinting, key)?;
                Some(Glyph { key, mask, x: 4 + 11 * n as i32, y: 50 })
            }).collect()
        };
        let table = sharp.table(INK);
        assert_eq!(run().len(), 8);
        same(|p| p.glyphs(run(), INK, table.clone()), |c| {
            for g in run() {
                crate::sharp::blit(c, &g.mask, g.x, g.y, INK, &table);
            }
        });
    }

    #[test]
    fn an_image_plays_as_drawn() {
        let mut pix = Pixmap::new(30, 20).unwrap();
        pix.fill(tiny_skia::Color::from_rgba8(20, 120, 60, 160));
        let pix = Rc::new(pix);
        same(|p| p.image(7, pix.clone(), 40, 50), |c| {
            c.draw_pixmap(40, 50, pix.as_ref().as_ref(), &PixmapPaint::default(), Transform::identity(), None)
        });
    }

    #[test]
    fn shadows_below_and_inside_play_as_drawn() {
        let falls = [Fall { y: 3.0, blur: 8.0, spread: 0.0, alpha: 0.4 }, Fall { y: 1.0, blur: 2.0, spread: 0.0, alpha: 0.2 }];
        let b = r(20.0, 20.0, 50.0, 40.0);
        let ink = Colour { a: 1.0, ..INK };
        if crate::theme::drawn_by_processor() {
            return;
        }
        same(|p| p.shadow(b, 8.0, &falls, ink), |c| Shadows::default().below(c, b, 8.0, &falls, ink));
        same(|p| p.shadow_inside(b, 8.0, falls[0], ink), |c| Shadows::default().inside(c, b, 8.0, falls[0], ink));
    }

    #[test]
    fn a_clip_plays_as_a_layer_of_its_own_cut_round() {
        let (b, radius) = (at(10, 12, 60, 50), 12.0);
        let inner = r(-5.0, -5.0, 80.0, 70.0);
        same(|p| {
            p.push_clip(b, radius);
            p.fill(inner, INK);
            p.pop();
        }, |c| {
            let mut layer = Pixmap::new(60, 50).unwrap();
            layer.fill_rect(tiny_skia::Rect::from_xywh(inner.x, inner.y, inner.w, inner.h).unwrap(), &paint(INK), Transform::identity(), None);
            let mut mask = Mask::new(SIDE, SIDE).unwrap();
            mask.fill_path(&crate::ui::rounded(r(10.0, 12.0, 60.0, 50.0), radius).unwrap(), FillRule::Winding, true, Transform::identity());
            c.draw_pixmap(10, 12, layer.as_ref(), &PixmapPaint::default(), Transform::identity(), Some(&mask));
        });
    }

    #[test]
    fn layers_fade_nest_and_close_at_the_end() {
        let inner = r(2.5, 3.5, 30.0, 20.0);
        same(|p| {
            p.push_layer(at(8, 9, 70, 60), 0.6);
            p.push_clip(at(5, 5, 40, 30), 0.0);
            p.fill(inner, INK);
            p.pop();
            p.line((0.0, 0.0), (60.0, 50.0), 1.5, INK);
        }, |c| {
            let mut faded = Pixmap::new(70, 60).unwrap();
            let mut cut = Pixmap::new(40, 30).unwrap();
            cut.fill_rect(tiny_skia::Rect::from_xywh(inner.x, inner.y, inner.w, inner.h).unwrap(), &paint(INK), Transform::identity(), None);
            faded.draw_pixmap(5, 5, cut.as_ref(), &PixmapPaint::default(), Transform::identity(), None);
            let mut b = PathBuilder::new();
            b.move_to(0.0, 0.0);
            b.line_to(60.0, 50.0);
            let stroke = Stroke { width: 1.5, ..Stroke::default() };
            faded.stroke_path(&b.finish().unwrap(), &paint(INK), &stroke, Transform::identity(), None);
            let fade = PixmapPaint { opacity: 0.6, ..PixmapPaint::default() };
            c.draw_pixmap(8, 9, faded.as_ref(), &fade, Transform::identity(), None);
        });
    }

    #[test]
    fn pixels_draw_straight_onto_the_layer() {
        let scribble = |c: &mut PixmapMut| {
            c.fill_path(&triangle(), &paint(INK), FillRule::Winding, Transform::identity(), None);
            let px = c.data_mut();
            px[..8].copy_from_slice(&[1, 2, 3, 4, 5, 6, 7, 8]);
        };
        same(|p| p.pixels(at(0, 0, SIDE, SIDE), scribble), scribble);
    }
}
