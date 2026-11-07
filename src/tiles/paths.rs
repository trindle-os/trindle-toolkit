//! Free shapes on a sketch: paths of lines and curves, filled flat or with a gradient, stroked whole or dashed, clipped and shadowed.

use tiny_skia::{FillRule, GradientStop, LinearGradient, Mask, Paint, PathBuilder, Pixmap, PixmapMut, PixmapPaint, Point, RadialGradient, SpreadMode, Stroke, StrokeDash, Transform};

use super::super::colour::{paint, Colour};
use super::super::ui::rounded;
use super::super::widget::{Pen, Rect};
use super::super::paint::Painter;
use super::{Sketch, To};

#[derive(Clone, Default)]
pub struct Way(PathBuilder);

impl Way {
    pub fn new() -> Way {
        Way::default()
    }

    pub fn rounded((x, y, w, h): (f32, f32, f32, f32), radius: f32) -> Way {
        let mut way = Way::new();
        if let Some(p) = rounded(Rect { x, y, w, h }, radius) {
            way.0.push_path(&p);
        }
        way
    }

    pub fn circle((x, y): (f32, f32), r: f32) -> Way {
        let mut way = Way::new();
        way.0.push_circle(x, y, r);
        way
    }

    pub fn move_to(mut self, x: f32, y: f32) -> Way {
        self.0.move_to(x, y);
        self
    }

    pub fn line_to(mut self, x: f32, y: f32) -> Way {
        self.0.line_to(x, y);
        self
    }

    pub fn quad_to(mut self, c: (f32, f32), to: (f32, f32)) -> Way {
        self.0.quad_to(c.0, c.1, to.0, to.1);
        self
    }

    pub fn cubic_to(mut self, c1: (f32, f32), c2: (f32, f32), to: (f32, f32)) -> Way {
        self.0.cubic_to(c1.0, c1.1, c2.0, c2.1, to.0, to.1);
        self
    }

    pub fn close(mut self) -> Way {
        self.0.close();
        self
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Fill {
    Flat(Colour),
    Linear { from: (f32, f32), to: (f32, f32), stops: Vec<(f32, Colour)> },
    Radial { centre: (f32, f32), r: f32, stops: Vec<(f32, Colour)> },
}

impl Fill {
    fn paint(&self) -> Paint<'static> {
        let stops = |s: &[(f32, Colour)]| s.iter().map(|&(at, c)| GradientStop::new(at, c.to_skia())).collect::<Vec<_>>();
        let first = |s: &[(f32, Colour)]| s.first().map_or(Colour { a: 0.0, ..Colour::rgb(0.0, 0.0, 0.0) }, |p| p.1);
        let shader = match self {
            Fill::Flat(c) => return paint(*c),
            Fill::Linear { from, to, stops: s } => {
                LinearGradient::new(Point::from_xy(from.0, from.1), Point::from_xy(to.0, to.1), stops(s), SpreadMode::Pad, Transform::identity())
                    .ok_or_else(|| first(s))
            }
            Fill::Radial { centre, r, stops: s } => {
                let c = Point::from_xy(centre.0, centre.1);
                RadialGradient::new(c, 0.0, c, *r, stops(s), SpreadMode::Pad, Transform::identity()).ok_or_else(|| first(s))
            }
        };
        match shader {
            Ok(shader) => Paint { shader, anti_alias: true, ..Paint::default() },
            Err(c) => paint(c),
        }
    }
}

impl<'a, 'b, 'c> Sketch<'a, 'b, 'c> {
    pub fn placed(canvas: &'a mut PixmapMut<'b>, pen: &'a Pen<'c>, k: f32, at: (f32, f32)) -> Self {
        let size = (canvas.width(), canvas.height());
        Sketch { to: To::Canvas(canvas), pen, size, k, at }
    }

    pub fn placed_on(painter: &'a mut Painter<'b>, pen: &'a Pen<'c>, size: (u32, u32), k: f32, at: (f32, f32)) -> Self where 'c: 'b {
        Sketch { to: To::Painter(painter, pen.fonts), pen, size, k, at }
    }

    fn units(&self) -> Transform {
        Transform::from_row(self.k, 0.0, 0.0, self.k, self.at.0, self.at.1)
    }

    pub fn fill(&mut self, way: &Way, fill: &Fill) {
        let Some(path) = way.0.clone().finish() else { return };
        let units = self.units();
        if let (To::Painter(p, _), Fill::Flat(c)) = (&mut self.to, fill) {
            if let Some(path) = path.transform(units) {
                p.fill_path(path, FillRule::Winding, *c);
            }
            return;
        }
        let paint = fill.paint();
        self.pixels(move |c| c.fill_path(&path, &paint, FillRule::Winding, units, None));
    }

    pub fn stroke(&mut self, way: &Way, width: f32, fill: &Fill, dash: &[f32]) {
        let Some(path) = way.0.clone().finish() else { return };
        let dashed = if dash.is_empty() { None } else { StrokeDash::new(dash.to_vec(), 0.0) };
        if dashed.is_none() && !dash.is_empty() {
            return;
        }
        let stroke = Stroke {
            width: width.max(1.0 / self.k),
            line_cap: tiny_skia::LineCap::Round,
            line_join: tiny_skia::LineJoin::Round,
            dash: dashed,
            ..Stroke::default()
        };
        let (paint, units) = (fill.paint(), self.units());
        self.pixels(move |c| c.stroke_path(&path, &paint, &stroke, units, None));
    }

    pub fn vector(&mut self, tree: &super::super::picture::SvgTree, (x, y, w, h): (f32, f32, f32, f32)) {
        let r = self.r(x, y, w, h);
        match &mut self.to {
            To::Canvas(canvas) => super::super::vector::draw(canvas, tree, r),
            To::Painter(..) => {
                let tree = tree.clone();
                self.pixels(move |c| super::super::vector::draw(c, &tree, r));
            }
        }
    }

    pub fn clipped(&mut self, way: &Way, art: impl FnOnce(&mut Sketch)) {
        let (w, h) = self.size;
        let (Some(path), Some(mut mask), Some(mut layer)) = (way.0.clone().finish(), Mask::new(w, h), Pixmap::new(w, h)) else { return };
        mask.fill_path(&path, FillRule::Winding, true, self.units());
        let reached = self.reach(&path, self.units(), 0.0);
        let mut inner = layer.as_mut();
        art(&mut Sketch { to: To::Canvas(&mut inner), pen: self.pen, size: self.size, k: self.k, at: self.at });
        let Some((x0, y0, x1, y1)) = reached else { return };
        let Some(part) = tiny_skia::IntRect::from_xywh(x0 as i32, y0 as i32, (x1 - x0) as u32, (y1 - y0) as u32).and_then(|r| layer.as_ref().clone_rect(r)) else { return };
        self.pixels(move |c| c.draw_pixmap(x0 as i32, y0 as i32, part.as_ref(), &PixmapPaint::default(), Transform::identity(), Some(&mask)));
    }

    fn reach(&self, path: &tiny_skia::Path, by: Transform, room: f32) -> Option<(usize, usize, usize, usize)> {
        let b = path.clone().transform(by)?.bounds();
        let (w, h) = (self.size.0 as f32, self.size.1 as f32);
        let edge = |from: f32, to: f32, most: f32| ((from - room - 1.0).floor().clamp(0.0, most) as usize, (to + room + 1.0).ceil().clamp(0.0, most) as usize);
        let ((x0, x1), (y0, y1)) = (edge(b.left(), b.right(), w), edge(b.top(), b.bottom(), h));
        (x0 < x1 && y0 < y1).then_some((x0, y0, x1, y1))
    }

    pub fn shadow(&mut self, way: &Way, by: (f32, f32), blur: f32, c: Colour) {
        if super::super::theme::drawn_by_processor() {
            return;
        }
        let (w, h) = self.size;
        let (Some(path), Some(mut mask)) = (way.0.clone().finish(), Mask::new(w, h)) else { return };
        let (at, reach) = (self.units().pre_translate(by.0, by.1), (blur * self.k / 2.0).round().max(0.0) as usize);
        let Some((x0, y0, x1, y1)) = self.reach(&path, at, 3.0 * reach as f32) else { return };
        mask.fill_path(&path, FillRule::Winding, true, at);
        let (rw, rh) = (x1 - x0, y1 - y0);
        let mut cover = Vec::with_capacity(rw * rh);
        for y in y0..y1 {
            cover.extend(mask.data()[y * w as usize + x0..y * w as usize + x1].iter().map(|&a| a as f32 / 255.0));
        }
        super::super::shadow::blur(&mut cover, rw, rh, reach);
        for (y, row) in (y0..y1).zip(cover.chunks(rw)) {
            for (m, v) in mask.data_mut()[y * w as usize + x0..y * w as usize + x1].iter_mut().zip(row) {
                *m = (v.clamp(0.0, 1.0) * 255.0).round() as u8;
            }
        }
        let part = tiny_skia::Rect::from_xywh(x0 as f32, y0 as f32, rw as f32, rh as f32).expect("a part of a canvas has a size");
        self.pixels(move |canvas| canvas.fill_rect(part, &paint(c), Transform::identity(), Some(&mask)));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::icon::Icons;
    use crate::text::Fonts;
    use crate::theme::Theme;

    fn draw(art: impl FnOnce(&mut Sketch)) -> Pixmap {
        let theme = Theme::new(true, "#fbc711");
        let (fonts, icons) = (Fonts::new(), Icons::new());
        let pen = Pen { theme: &theme, fonts: &fonts, icons: &icons, scale: 1.0 };
        let mut pix = Pixmap::new(100, 100).unwrap();
        art(&mut Sketch::fit(&mut pix.as_mut(), &pen, (50.0, 50.0)));
        pix
    }

    fn at(p: &Pixmap, x: u32, y: u32) -> tiny_skia::PremultipliedColorU8 {
        p.pixel(x, y).unwrap()
    }

    const RED: Colour = Colour::rgb(1.0, 0.0, 0.0);
    const BLUE: Colour = Colour::rgb(0.0, 0.0, 1.0);

    #[test]
    fn a_curved_path_fills_in_units_scaled_to_the_canvas() {
        let p = draw(|s| s.fill(&Way::new().move_to(0.0, 0.0).line_to(50.0, 0.0).quad_to((50.0, 50.0), (0.0, 50.0)).close(), &Fill::Flat(RED)));
        assert_eq!(at(&p, 10, 10).red(), 255, "inside");
        assert_eq!(at(&p, 98, 98).alpha(), 0, "outside the curve");
    }

    #[test]
    fn a_linear_gradient_runs_from_its_first_colour_to_its_last() {
        let fill = Fill::Linear { from: (0.0, 0.0), to: (50.0, 0.0), stops: vec![(0.0, RED), (1.0, BLUE)] };
        let p = draw(|s| s.fill(&Way::rounded((0.0, 0.0, 50.0, 50.0), 0.0), &fill));
        assert!(at(&p, 2, 50).red() > 240 && at(&p, 2, 50).blue() < 15);
        assert!(at(&p, 97, 50).blue() > 240 && at(&p, 97, 50).red() < 15);
    }

    #[test]
    fn a_radial_gradient_and_a_one_stop_gradient_both_draw() {
        let fill = Fill::Radial { centre: (25.0, 25.0), r: 25.0, stops: vec![(0.0, RED), (1.0, BLUE)] };
        let p = draw(|s| s.fill(&Way::circle((25.0, 25.0), 25.0), &fill));
        assert!(at(&p, 50, 50).red() > 240, "red at the middle");
        assert!(at(&p, 50, 3).blue() > 200, "blue at the rim");
        let one = Fill::Linear { from: (0.0, 0.0), to: (1.0, 0.0), stops: vec![(0.0, BLUE)] };
        assert_eq!(at(&draw(|s| s.fill(&Way::circle((25.0, 25.0), 20.0), &one)), 50, 50).blue(), 255);
    }

    #[test]
    fn a_dashed_line_leaves_gaps_and_a_whole_one_does_not() {
        let line = Way::new().move_to(0.0, 25.0).line_to(50.0, 25.0);
        let whole = draw(|s| s.stroke(&line, 2.0, &Fill::Flat(RED), &[]));
        let dashed = draw(|s| s.stroke(&line, 2.0, &Fill::Flat(RED), &[10.0, 10.0]));
        let lit = |p: &Pixmap| (0..100).filter(|&x| at(p, x, 50).alpha() > 128).count();
        assert!(lit(&whole) > 95);
        assert!((40..70).contains(&lit(&dashed)), "{}", lit(&dashed));
    }

    #[test]
    fn clipping_keeps_drawing_inside_the_shape_including_words() {
        let p = draw(|s| s.clipped(&Way::circle((25.0, 25.0), 10.0), |s| {
            s.rect((0.0, 0.0, 50.0, 50.0), 0.0, RED);
            s.text("WWWWWWWW", (25.0, 25.0), 10.0, BLUE);
        }));
        assert_eq!(at(&p, 50, 50).alpha(), 255, "inside");
        assert_eq!(at(&p, 5, 5).alpha(), 0, "outside the circle nothing shows");
        assert_eq!(at(&p, 95, 50).alpha(), 0, "nor words past it");
    }

    #[test]
    fn a_shadow_is_soft_and_falls_where_it_is_moved() {
        let p = draw(|s| s.shadow(&Way::rounded((15.0, 15.0, 20.0, 20.0), 4.0), (0.0, 5.0), 4.0, Colour::rgb(0.0, 0.0, 0.0)));
        assert!(at(&p, 50, 75).alpha() > at(&p, 50, 25).alpha(), "it falls downwards");
        let edge = at(&p, 50, 82).alpha();
        assert!(edge > 0 && edge < 255, "its edge is soft: {edge}");
    }

    fn on_the_whole_canvas(pix: &mut Pixmap, k: f32, way: &Way, by: (f32, f32), blur: f32, clip: bool) {
        let (w, h) = (pix.width(), pix.height());
        let units = Transform::from_row(k, 0.0, 0.0, k, 3.0, 2.0);
        let path = way.0.clone().finish().unwrap();
        let mut mask = Mask::new(w, h).unwrap();
        if clip {
            mask.fill_path(&path, FillRule::Winding, true, units);
            let mut layer = Pixmap::new(w, h).unwrap();
            layer.fill(tiny_skia::Color::from_rgba8(200, 30, 30, 255));
            pix.draw_pixmap(0, 0, layer.as_ref(), &PixmapPaint::default(), Transform::identity(), Some(&mask));
            return;
        }
        mask.fill_path(&path, FillRule::Winding, true, units.pre_translate(by.0, by.1));
        let mut cover: Vec<f32> = mask.data().iter().map(|&a| a as f32 / 255.0).collect();
        crate::shadow::blur(&mut cover, w as usize, h as usize, (blur * k / 2.0).round().max(0.0) as usize);
        for (m, v) in mask.data_mut().iter_mut().zip(&cover) {
            *m = (v.clamp(0.0, 1.0) * 255.0).round() as u8;
        }
        let whole = tiny_skia::Rect::from_xywh(0.0, 0.0, w as f32, h as f32).unwrap();
        pix.fill_rect(whole, &paint(Colour { r: 0.1, g: 0.2, b: 0.3, a: 0.6 }), Transform::identity(), Some(&mask));
    }

    #[test]
    fn a_shadow_and_a_clip_are_the_pixels_they_were_when_worked_on_the_whole_canvas() {
        let theme = Theme::new(true, "#fbc711");
        let (fonts, icons) = (Fonts::new(), Icons::new());
        let pen = Pen { theme: &theme, fonts: &fonts, icons: &icons, scale: 1.0 };
        let shapes = [
            Way::rounded((20.0, 15.0, 30.0, 20.0), 5.0), Way::circle((40.0, 30.0), 12.0), Way::rounded((-15.0, -10.0, 30.0, 25.0), 4.0),
            Way::rounded((75.0, 50.0, 30.0, 40.0), 6.0), Way::rounded((400.0, 400.0, 30.0, 20.0), 4.0), Way::rounded((0.0, 0.0, 200.0, 200.0), 8.0),
        ];
        for (w, h) in [(100, 100), (131, 77)] {
            for k in [1.0, 2.0] {
                for (i, way) in shapes.iter().enumerate() {
                    for (by, blur) in [((0.0, 0.0), 0.0), ((1.5, 4.0), 4.0), ((-3.0, 7.0), 16.0), ((0.0, 2.0), 40.0)] {
                        for clip in [false, true] {
                            let mut whole = Pixmap::new(w, h).unwrap();
                            let mut part = Pixmap::new(w, h).unwrap();
                            whole.fill(tiny_skia::Color::from_rgba8(240, 240, 250, 255));
                            part.fill(tiny_skia::Color::from_rgba8(240, 240, 250, 255));
                            on_the_whole_canvas(&mut whole, k, way, by, blur, clip);
                            let mut canvas = part.as_mut();
                            let mut sketch = Sketch::placed(&mut canvas, &pen, k, (3.0, 2.0));
                            if clip {
                                sketch.clipped(way, |s| s.pixels(|c| c.fill(tiny_skia::Color::from_rgba8(200, 30, 30, 255))));
                            } else {
                                sketch.shadow(way, by, blur, Colour { r: 0.1, g: 0.2, b: 0.3, a: 0.6 });
                            }
                            assert!(whole.data() == part.data(), "{w}x{h} k {k} shape {i} by {by:?} blur {blur} clip {clip}: the part worked differs from the whole");
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn a_sketch_on_a_painter_draws_as_one_on_the_canvas() {
        let art = |s: &mut Sketch| {
            s.rect((2.0, 2.0, 30.0, 20.0), 4.0, RED);
            s.shadow(&Way::circle((25.0, 25.0), 8.0), (0.0, 3.0), 4.0, BLUE);
            s.fill(&Way::circle((25.0, 25.0), 10.0), &Fill::Flat(BLUE));
            s.fill(&Way::circle((10.0, 40.0), 8.0), &Fill::Linear { from: (0.0, 0.0), to: (50.0, 0.0), stops: vec![(0.0, RED), (1.0, BLUE)] });
            s.stroke(&Way::new().move_to(0.0, 45.0).line_to(50.0, 30.0), 2.0, &Fill::Flat(RED), &[4.0, 2.0]);
            s.line((0.0, 0.0), (50.0, 50.0), 1.5, BLUE);
            s.arc((25.0, 25.0), 15.0, 3.0, 0.1, 0.6, RED);
            s.pie((40.0, 10.0), 6.0, 0.3, BLUE);
            s.text("Wy", (25.0, 25.0), 10.0, BLUE);
            s.clipped(&Way::circle((25.0, 25.0), 12.0), |s| s.rect((0.0, 0.0, 50.0, 50.0), 0.0, RED));
        };
        let direct = draw(art);
        let theme = Theme::new(true, "#fbc711");
        let (fonts, icons) = (Fonts::new(), Icons::new());
        let pen = Pen { theme: &theme, fonts: &fonts, icons: &icons, scale: 1.0 };
        let mut painter = Painter::default();
        art(&mut Sketch::fit_on(&mut painter, &pen, (100, 100), (50.0, 50.0)));
        let mut played = Pixmap::new(100, 100).unwrap();
        painter.play(&mut played.as_mut());
        assert!(played.data() == direct.data(), "the recorded sketch differs from the drawn one");
    }
}
