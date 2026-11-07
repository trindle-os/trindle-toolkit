//! A colour picker a program keeps in its state: a painted square for how much colour and how bright, a slider for the hue and a box for the hex code. `ColourBox::update` takes what happened, `view` draws it.

use tiny_skia::{Color, GradientStop, LinearGradient, Paint, PathBuilder, Point, SpreadMode, Stroke, Transform};

use super::colour::Colour;
use super::theme::{corner_px, Part};
use super::widget::{button, column, drawing, entry, row, slider, word_button, Length, Node, Pen, Pointer, Rect};
use super::input::{Input, Keysym};
use crate::t;

const STEP: f32 = 0.02;

#[derive(Clone, Debug, PartialEq)]
pub enum Edit {
    Square(f32, f32),
    Hue(f32),
    Hex(String),
    Stay,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ColourBox {
    pub hue: f32,
    pub saturation: f32,
    pub brightness: f32,
    pub hex: String,
}

impl ColourBox {
    pub fn new(c: Colour) -> ColourBox {
        let (hue, saturation, brightness) = to_hsv(c);
        ColourBox { hue, saturation, brightness, hex: Colour { a: 1.0, ..c }.css() }
    }

    pub fn colour(&self) -> Colour {
        from_hsv(self.hue, self.saturation, self.brightness)
    }

    pub fn update(&mut self, edit: Edit) {
        match edit {
            Edit::Stay => {}
            Edit::Square(s, v) => {
                self.saturation = s.clamp(0.0, 1.0);
                self.brightness = v.clamp(0.0, 1.0);
                self.hex = self.colour().css();
            }
            Edit::Hue(h) => {
                self.hue = h.rem_euclid(360.0);
                self.hex = self.colour().css();
            }
            Edit::Hex(text) => {
                if let Some(c) = Colour::parse(&text).filter(|c| c.a >= 1.0) {
                    let (h, s, v) = to_hsv(c);
                    if s > 0.0 && v > 0.0 {
                        self.hue = h;
                    }
                    self.saturation = s;
                    self.brightness = v;
                }
                self.hex = text;
            }
        }
    }

    pub fn view<M: 'static>(&self, wrap: impl Fn(Edit) -> M + Clone + 'static) -> Node<M> {
        let (hue, s, v) = (self.hue, self.saturation, self.brightness);
        let (at, keyed, hued, typed) = (wrap.clone(), wrap.clone(), wrap.clone(), wrap);
        let sizes = crate::theme::sheet();
        let square_size = sizes.colour_square;
        let square = drawing(square_size, "", move |canvas, pen| paint_square(canvas, pen, hue, s, v))
            .called(t!("kit-colour-square"))
            .on_pointer(move |p| match p {
                Pointer::Down(x, y) | Pointer::Move(x, y) | Pointer::Up(x, y) => at(Edit::Square(x / square_size.0, 1.0 - y / square_size.1)),
                _ => at(Edit::Stay),
            })
            .on_keys(move |input| {
                let Input::Key { sym, .. } = input else { return None };
                let (ds, dv) = match *sym {
                    Keysym::Left => (-STEP, 0.0),
                    Keysym::Right => (STEP, 0.0),
                    Keysym::Up => (0.0, STEP),
                    Keysym::Down => (0.0, -STEP),
                    _ => return None,
                };
                Some(keyed(Edit::Square(s + ds, v + dv)))
            });
        let colour = self.colour();
        let side = sizes.colour_swatch;
        let swatch = drawing((side, side), "", move |canvas, pen| {
            super::ui::fill(canvas, scaled(Rect { x: 0.0, y: 0.0, w: side, h: side }, pen), crate::theme::sheet().colour_swatch_corner * pen.scale, colour);
        }).called(t!("kit-colour-swatch", colour = colour.css()));
        column(vec![
            row(vec![square, swatch]).gap(crate::theme::gaps().roomy),
            slider(hue, 0.0, 360.0, 1.0, t!("kit-colour-hue"), move |h| hued(Edit::Hue(h))),
            entry(self.hex.clone(), t!("kit-colour-hex"), move |text| typed(Edit::Hex(text))).width(Length::Fixed(sizes.colour_code)),
        ]).gap(crate::theme::gaps().roomy).width(Length::Shrink)
    }
}

pub fn swatch_grid<M: Clone + 'static>(colours: &[Colour], chosen: Option<usize>, columns: usize, pick: impl Fn(usize) -> M) -> Node<M> {
    let columns = columns.max(1);
    let rows = colours.chunks(columns).enumerate().map(|(r, line)| {
        row(line.iter().enumerate().map(|(c, &colour)| {
            let i = r * columns + c;
            button("", colour.css()).face(move |canvas, rect, _, pen| {
                super::ui::fill(canvas, rect, corner_px(Part::Chip, rect.h, pen.scale), colour);
            }).on(chosen == Some(i)).on_press(pick(i))
        }).collect()).gap(crate::theme::gaps().medium).width(Length::Shrink)
    }).collect();
    column(rows).gap(crate::theme::gaps().medium).width(Length::Shrink)
}

#[derive(Clone, Debug, PartialEq)]
pub enum Shade {
    Choose(usize),
    Place(f32),
    Add,
    Remove,
    Colour(Edit),
}

#[derive(Clone, Debug, PartialEq)]
pub struct GradientBox {
    pub stops: Vec<(f32, Colour)>,
    pub chosen: usize,
    pub picker: ColourBox,
}

impl GradientBox {
    pub fn new(stops: Vec<(f32, Colour)>) -> Option<GradientBox> {
        let picker = ColourBox::new(stops.first()?.1);
        (stops.len() >= 2).then_some(GradientBox { stops, chosen: 0, picker })
    }

    fn sorted(&self) -> Vec<(f32, Colour)> {
        let mut s = self.stops.clone();
        s.sort_by(|a, b| a.0.total_cmp(&b.0));
        s
    }

    pub fn colour_at(&self, p: f32) -> Colour {
        let s = self.sorted();
        let after = s.iter().position(|(at, _)| *at >= p).unwrap_or(s.len() - 1);
        let (l, r) = (s[after.saturating_sub(1)], s[after]);
        let t = if r.0 > l.0 { ((p - l.0) / (r.0 - l.0)).clamp(0.0, 1.0) } else { 1.0 };
        let mix = |a: f32, b: f32| a + (b - a) * t;
        Colour { r: mix(l.1.r, r.1.r), g: mix(l.1.g, r.1.g), b: mix(l.1.b, r.1.b), a: mix(l.1.a, r.1.a) }
    }

    pub fn update(&mut self, edit: Shade) {
        match edit {
            Shade::Choose(i) if i < self.stops.len() => {
                self.chosen = i;
                self.picker = ColourBox::new(self.stops[i].1);
            }
            Shade::Choose(_) => {}
            Shade::Place(p) => self.stops[self.chosen].0 = p.clamp(0.0, 1.0),
            Shade::Add => {
                self.stops.push((0.5, self.colour_at(0.5)));
                self.update(Shade::Choose(self.stops.len() - 1));
            }
            Shade::Remove if self.stops.len() > 2 => {
                self.stops.remove(self.chosen);
                self.update(Shade::Choose(self.chosen.min(self.stops.len() - 1)));
            }
            Shade::Remove => {}
            Shade::Colour(e) => {
                self.picker.update(e);
                self.stops[self.chosen].1 = self.picker.colour();
            }
        }
    }

    pub fn css(&self) -> String {
        let stops: Vec<String> = self.sorted().iter().map(|(p, c)| format!("{} {}%", c.css(), (p * 100.0).round())).collect();
        format!("linear-gradient(90deg, {})", stops.join(", "))
    }

    pub fn view<M: Clone + 'static>(&self, wrap: impl Fn(Shade) -> M + Clone + 'static) -> Node<M> {
        let (sorted, chosen) = (self.sorted(), self.stops[self.chosen].0);
        let marks: Vec<(f32, bool)> = self.stops.iter().enumerate().map(|(i, s)| (s.0, i == self.chosen)).collect();
        let bar = drawing((crate::theme::sheet().colour_square.0, crate::theme::sheet().gradient_bar), t!("kit-gradient-bar", n = self.stops.len() as i64), move |canvas, pen| paint_bar(canvas, pen, &sorted, &marks))
            .width(Length::Fill);
        let colours: Vec<Colour> = self.stops.iter().map(|s| s.1).collect();
        let (place, add, remove, edit, pick) = (wrap.clone(), wrap.clone(), wrap.clone(), wrap.clone(), wrap);
        column(vec![
            bar,
            swatch_grid(&colours, Some(self.chosen), 8, move |i| pick(Shade::Choose(i))),
            slider(chosen, 0.0, 1.0, 0.01, t!("kit-gradient-place"), move |p| place(Shade::Place(p))),
            row(vec![
                word_button(t!("kit-gradient-add")).on_press(add(Shade::Add)),
                word_button(t!("kit-gradient-remove")).on_press(remove(Shade::Remove)).off(self.stops.len() <= 2),
            ]).gap(crate::theme::gaps().medium),
            self.picker.view(move |e| edit(Shade::Colour(e))),
        ]).gap(crate::theme::gaps().roomy).width(Length::Shrink)
    }
}

fn paint_bar(canvas: &mut tiny_skia::PixmapMut, pen: &Pen, sorted: &[(f32, Colour)], marks: &[(f32, bool)]) {
    let (w, h, k) = (canvas.width() as f32, canvas.height() as f32, pen.scale);
    let stops = sorted.iter().map(|(p, c)| GradientStop::new(*p, c.to_skia())).collect();
    let shader = LinearGradient::new(Point::from_xy(0.0, 0.0), Point::from_xy(w, 0.0), stops, SpreadMode::Pad, Transform::identity());
    if let (Some(shader), Some(path)) = (shader, super::ui::rounded(Rect { x: 0.0, y: 0.0, w, h }, corner_px(Part::Box, h, k))) {
        canvas.fill_path(&path, &Paint { shader, anti_alias: true, ..Paint::default() }, tiny_skia::FillRule::Winding, Transform::identity(), None);
    }
    for &(p, on) in marks {
        let x = (p * w).max(k * 2.0).min(w - k * 2.0);
        let mut pb = PathBuilder::new();
        pb.move_to(x, 0.0);
        pb.line_to(x, h);
        let Some(line) = pb.finish() else { continue };
        let width = if on { 3.0 } else { 1.5 };
        for (colour, extra) in [(Colour::rgb(0.0, 0.0, 0.0), 2.0), (Colour::rgb(1.0, 1.0, 1.0), 0.0)] {
            let stroke = Stroke { width: (width + extra) * k, ..Stroke::default() };
            canvas.stroke_path(&line, &super::colour::paint(colour), &stroke, Transform::identity(), None);
        }
    }
}

fn scaled(r: Rect, pen: &Pen) -> Rect {
    Rect { x: r.x * pen.scale, y: r.y * pen.scale, w: r.w * pen.scale, h: r.h * pen.scale }
}

fn paint_square(canvas: &mut tiny_skia::PixmapMut, pen: &Pen, hue: f32, s: f32, v: f32) {
    let (w, h) = (canvas.width() as f32, canvas.height() as f32);
    let full = from_hsv(hue, 1.0, 1.0);
    let Some(area) = tiny_skia::Rect::from_xywh(0.0, 0.0, w, h) else { return };
    let across = LinearGradient::new(
        Point::from_xy(0.0, 0.0),
        Point::from_xy(w, 0.0),
        vec![GradientStop::new(0.0, Color::WHITE), GradientStop::new(1.0, full.to_skia())],
        SpreadMode::Pad,
        Transform::identity(),
    );
    let down = LinearGradient::new(
        Point::from_xy(0.0, 0.0),
        Point::from_xy(0.0, h),
        vec![GradientStop::new(0.0, Color::TRANSPARENT), GradientStop::new(1.0, Color::BLACK)],
        SpreadMode::Pad,
        Transform::identity(),
    );
    for shader in [across, down].into_iter().flatten() {
        let paint = Paint { shader, ..Paint::default() };
        canvas.fill_rect(area, &paint, Transform::identity(), None);
    }
    let (cx, cy, k) = (s * w, (1.0 - v) * h, pen.scale);
    let Some(ring) = PathBuilder::from_circle(cx, cy, 6.0 * k) else { return };
    for (colour, width) in [(Colour::rgb(0.0, 0.0, 0.0), 4.0), (Colour::rgb(1.0, 1.0, 1.0), 2.0)] {
        let stroke = Stroke { width: width * k, ..Stroke::default() };
        canvas.stroke_path(&ring, &super::colour::paint(colour), &stroke, Transform::identity(), None);
    }
}

fn to_hsv(c: Colour) -> (f32, f32, f32) {
    let (max, min) = (c.r.max(c.g).max(c.b), c.r.min(c.g).min(c.b));
    let d = max - min;
    let hue = if d == 0.0 {
        0.0
    } else if max == c.r {
        60.0 * ((c.g - c.b) / d).rem_euclid(6.0)
    } else if max == c.g {
        60.0 * ((c.b - c.r) / d + 2.0)
    } else {
        60.0 * ((c.r - c.g) / d + 4.0)
    };
    (hue, if max == 0.0 { 0.0 } else { d / max }, max)
}

fn from_hsv(h: f32, s: f32, v: f32) -> Colour {
    let (c, h) = (v * s, (h.rem_euclid(360.0)) / 60.0);
    let x = c * (1.0 - (h % 2.0 - 1.0).abs());
    let (r, g, b) = match h as u32 {
        0 => (c, x, 0.0),
        1 => (x, c, 0.0),
        2 => (0.0, c, x),
        3 => (0.0, x, c),
        4 => (x, 0.0, c),
        _ => (c, 0.0, x),
    };
    let m = v - c;
    Colour::rgb(r + m, g + m, b + m)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hsv_round_trips_and_hex_sets_the_square() {
        for hex in ["#fbc711", "#3366cc", "#808080", "#000000", "#ffffff"] {
            let c = Colour::parse(hex).unwrap();
            let (h, s, v) = to_hsv(c);
            assert_eq!(from_hsv(h, s, v).css(), hex);
        }
        let mut b = ColourBox::new(Colour::parse("#ff0000").unwrap());
        b.update(Edit::Hex("#0000ff".into()));
        assert_eq!((b.hue, b.saturation, b.brightness), (240.0, 1.0, 1.0));
        b.update(Edit::Hex("#12".into()));
        assert_eq!(b.hex, "#12");
        assert_eq!(b.colour().css(), "#0000ff");
        b.update(Edit::Stay);
        assert_eq!(b.hex, "#12");
    }

    #[test]
    fn a_gradient_blends_between_stops_adds_removes_and_writes_css() {
        let (red, blue) = (Colour::parse("#ff0000").unwrap(), Colour::parse("#0000ff").unwrap());
        assert!(GradientBox::new(vec![(0.0, red)]).is_none());
        let mut g = GradientBox::new(vec![(1.0, blue), (0.0, red)]).unwrap();
        assert_eq!(g.css(), "linear-gradient(90deg, #ff0000 0%, #0000ff 100%)");
        assert_eq!((g.colour_at(-1.0).css(), g.colour_at(2.0).css(), g.colour_at(0.5).css()), ("#ff0000".into(), "#0000ff".into(), "#800080".into()));
        g.update(Shade::Add);
        assert_eq!((g.stops.len(), g.chosen, g.picker.colour().css()), (3, 2, "#800080".into()));
        g.update(Shade::Place(7.0));
        g.update(Shade::Colour(Edit::Hex("#00ff00".into())));
        assert_eq!(g.css(), "linear-gradient(90deg, #ff0000 0%, #0000ff 100%, #00ff00 100%)");
        g.update(Shade::Choose(9));
        g.update(Shade::Remove);
        g.update(Shade::Remove);
        assert_eq!((g.stops.len(), g.chosen), (2, 1));
    }

    #[test]
    fn a_see_through_colour_shows_its_solid_hex() {
        assert_eq!(ColourBox::new(Colour { a: 0.5, ..Colour::rgb(1.0, 0.0, 0.0) }).hex, "#ff0000");
    }

    #[test]
    fn the_square_and_the_hue_rewrite_the_hex_box() {
        let mut b = ColourBox::new(Colour::parse("#ff0000").unwrap());
        b.update(Edit::Square(0.0, 1.0));
        assert_eq!(b.hex, "#ffffff");
        b.update(Edit::Square(2.0, -1.0));
        assert_eq!((b.saturation, b.brightness, b.hex.as_str()), (1.0, 0.0, "#000000"));
        b.update(Edit::Hue(480.0));
        assert_eq!(b.hue, 120.0);
    }
}
