//! The shared tiles of Trindle's style: card head, picture tile, stat tile and small pictures; colour only in the pictures.

use tiny_skia::{FillRule, Path, PathBuilder, PixmapMut, Stroke, Transform};

use super::colour::{paint, Colour};
use super::draw::{around, words_on};
use super::paint::Painter;
use super::text::{Fonts, Style};
use super::theme::{self, mix, Palette, Theme};
use super::ui::rounded;
use super::widget::{card, column, drawing, heading, icon_circle, icon_image, note, row, figure, space, Length, Node, Pen, Rect};

mod paths;
pub use paths::{Fill, Way};

pub struct Sketch<'a, 'b, 'c> {
    to: To<'a, 'b>,
    pen: &'a Pen<'c>,
    size: (u32, u32),
    k: f32,
    at: (f32, f32),
}

enum To<'a, 'b> {
    Canvas(&'a mut PixmapMut<'b>),
    Painter(&'a mut Painter<'b>, &'b Fonts),
}

impl<'a, 'b, 'c> Sketch<'a, 'b, 'c> {
    pub fn fit(canvas: &'a mut PixmapMut<'b>, pen: &'a Pen<'c>, view: (f32, f32)) -> Self {
        let size = (canvas.width(), canvas.height());
        Sketch::fitted(To::Canvas(canvas), pen, size, view)
    }

    pub fn fit_on(painter: &'a mut Painter<'b>, pen: &'a Pen<'c>, size: (u32, u32), view: (f32, f32)) -> Self where 'c: 'b {
        Sketch::fitted(To::Painter(painter, pen.fonts), pen, size, view)
    }

    fn fitted(to: To<'a, 'b>, pen: &'a Pen<'c>, size: (u32, u32), view: (f32, f32)) -> Self {
        let (w, h) = (size.0 as f32, size.1 as f32);
        let k = (w / view.0.max(f32::EPSILON)).min(h / view.1.max(f32::EPSILON));
        let at = ((w - view.0 * k) / 2.0, (h - view.1 * k) / 2.0);
        Sketch { to, pen, size, k, at }
    }

    pub fn strip(canvas: &'a mut PixmapMut<'b>, pen: &'a Pen<'c>, tall: f32) -> Self {
        let size = (canvas.width(), canvas.height());
        Sketch { to: To::Canvas(canvas), pen, size, k: size.1 as f32 / tall.max(f32::EPSILON), at: (0.0, 0.0) }
    }

    pub fn strip_on(painter: &'a mut Painter<'b>, pen: &'a Pen<'c>, size: (u32, u32), tall: f32) -> Self where 'c: 'b {
        Sketch { to: To::Painter(painter, pen.fonts), pen, size, k: size.1 as f32 / tall.max(f32::EPSILON), at: (0.0, 0.0) }
    }

    pub fn wide(&self) -> f32 {
        self.size.0 as f32 / self.k
    }

    fn fill_path(&mut self, path: Path, c: Colour) {
        match &mut self.to {
            To::Canvas(canvas) => canvas.fill_path(&path, &paint(c), FillRule::Winding, Transform::identity(), None),
            To::Painter(p, _) => p.fill_path(path, FillRule::Winding, c),
        }
    }

    fn stroke_path(&mut self, path: Path, stroke: Stroke, c: Colour) {
        match &mut self.to {
            To::Canvas(canvas) => canvas.stroke_path(&path, &paint(c), &stroke, Transform::identity(), None),
            To::Painter(p, _) => p.stroke_path(path, stroke, c),
        }
    }

    fn pixels(&mut self, draw: impl FnOnce(&mut PixmapMut) + 'b) {
        match &mut self.to {
            To::Canvas(canvas) => draw(canvas),
            To::Painter(p, _) => p.pixels(around(0.0, 0.0, self.size.0 as f32, self.size.1 as f32, 0.0), draw),
        }
    }

    pub fn theme(&self) -> &Theme {
        self.pen.theme
    }

    fn r(&self, x: f32, y: f32, w: f32, h: f32) -> Rect {
        Rect { x: self.at.0 + x * self.k, y: self.at.1 + y * self.k, w: w * self.k, h: h * self.k }
    }

    fn p(&self, x: f32, y: f32) -> (f32, f32) {
        (self.at.0 + x * self.k, self.at.1 + y * self.k)
    }

    pub fn rect(&mut self, (x, y, w, h): (f32, f32, f32, f32), radius: f32, c: Colour) {
        if let Some(path) = rounded(self.r(x, y, w, h), radius * self.k) {
            self.fill_path(path, c);
        }
    }

    pub fn circle(&mut self, (x, y): (f32, f32), r: f32, c: Colour) {
        let (cx, cy) = self.p(x, y);
        if let Some(path) = PathBuilder::from_circle(cx, cy, (r * self.k).max(0.1)) {
            self.fill_path(path, c);
        }
    }

    pub fn arc(&mut self, (x, y): (f32, f32), r: f32, width: f32, from: f32, share: f32, c: Colour) {
        let share = share.clamp(0.0, 1.0);
        if share <= 0.0 {
            return;
        }
        let (cx, cy) = self.p(x, y);
        let rr = r * self.k;
        let mut pb = PathBuilder::new();
        let steps = (96.0 * share).ceil().max(2.0) as usize;
        for i in 0..=steps {
            let a = std::f32::consts::TAU * (from + share * i as f32 / steps as f32) - std::f32::consts::FRAC_PI_2;
            let (px, py) = (cx + rr * a.cos(), cy + rr * a.sin());
            if i == 0 { pb.move_to(px, py) } else { pb.line_to(px, py) }
        }
        if share >= 1.0 {
            pb.close();
        }
        let cap = if share >= 1.0 { tiny_skia::LineCap::Butt } else { tiny_skia::LineCap::Round };
        if let Some(path) = pb.finish() {
            let stroke = Stroke { width: (width * self.k).max(1.0), line_cap: cap, ..Stroke::default() };
            self.stroke_path(path, stroke, c);
        }
    }

    pub fn ring(&mut self, at: (f32, f32), r: f32, width: f32, c: Colour) {
        self.arc(at, r, width, 0.0, 1.0, c);
    }

    pub fn pie(&mut self, (x, y): (f32, f32), r: f32, share: f32, c: Colour) {
        let share = share.clamp(0.0, 1.0);
        if share <= 0.0 {
            return;
        }
        if share >= 1.0 {
            self.circle((x, y), r, c);
            return;
        }
        let (cx, cy) = self.p(x, y);
        let rr = r * self.k;
        let mut pb = PathBuilder::new();
        pb.move_to(cx, cy);
        let steps = (96.0 * share).ceil().max(2.0) as usize;
        for i in 0..=steps {
            let a = std::f32::consts::TAU * share * i as f32 / steps as f32 - std::f32::consts::FRAC_PI_2;
            pb.line_to(cx + rr * a.cos(), cy + rr * a.sin());
        }
        pb.close();
        if let Some(path) = pb.finish() {
            self.fill_path(path, c);
        }
    }

    pub fn line(&mut self, a: (f32, f32), b: (f32, f32), width: f32, c: Colour) {
        let (a, b) = (self.p(a.0, a.1), self.p(b.0, b.1));
        let mut pb = PathBuilder::new();
        pb.move_to(a.0, a.1);
        pb.line_to(b.0, b.1);
        if let Some(path) = pb.finish() {
            let stroke = Stroke { width: (width * self.k).max(1.0), line_cap: tiny_skia::LineCap::Round, ..Stroke::default() };
            self.stroke_path(path, stroke, c);
        }
    }

    pub fn shape(&mut self, points: &[(f32, f32)], c: Colour) {
        let mut pb = PathBuilder::new();
        for (i, &(x, y)) in points.iter().enumerate() {
            let (px, py) = self.p(x, y);
            if i == 0 { pb.move_to(px, py) } else { pb.line_to(px, py) }
        }
        pb.close();
        if let Some(path) = pb.finish() {
            self.fill_path(path, c);
        }
    }

    pub fn text(&mut self, words: &str, at: (f32, f32), size: f32, c: Colour) {
        let style = Style { weight: 500, ..Style::new(size * self.k) };
        let layout = self.pen.fonts.layout(words, &style, None);
        let (x, y) = self.p(at.0, at.1);
        let (x, y) = ((x - layout.width / 2.0).round(), (y - layout.height / 2.0).round());
        match &mut self.to {
            To::Canvas(canvas) => self.pen.fonts.draw(canvas, &layout, x, y, c),
            To::Painter(p, fonts) => words_on(p, fonts, layout, x, y, c),
        }
    }

    pub fn page(&self) -> Colour {
        self.pen.theme.bg
    }
    pub fn boxed(&self) -> Colour {
        self.pen.theme.raised
    }
    pub fn words(&self) -> Colour {
        self.pen.theme.fg
    }
    pub fn quiet(&self) -> Colour {
        self.pen.theme.muted
    }
    pub fn words_on(&self, fill: Colour) -> Colour {
        self.pen.theme.words_on(&[fill])
    }
    pub fn highlight(&self) -> Colour {
        self.pen.theme.soft
    }
    pub fn wash(&self, c: Palette, by: f32) -> Colour {
        mix(self.pen.theme.panel, c.colour(), by)
    }
    pub fn track(&self) -> Colour {
        self.pen.theme.well
    }
}

pub fn sketch<M>(tip: impl Into<String>, view: (f32, f32), least: f32, art: impl Fn(&mut Sketch) + 'static) -> Node<M> {
    drawing((view.0 * least / view.1, least), tip, move |canvas, pen| {
        let mut s = Sketch::fit(canvas, pen, view);
        art(&mut s);
    }).width(Length::Fill).height(Length::Fill)
}

fn strip<M>(tall: f32, art: impl Fn(&mut Sketch) + 'static) -> Node<M> {
    drawing((120.0, tall), "", move |canvas, pen| {
        let mut s = Sketch::strip(canvas, pen, tall);
        art(&mut s);
    }).width(Length::Fill)
}

pub fn tile_head<M>(icon: impl Into<String>, name: impl Into<String>, end: Vec<Node<M>>) -> Node<M> {
    let mut all = vec![icon_circle(icon), heading(name)];
    all.extend(end);
    all.push(space());
    row(all).gap(crate::theme::gaps().roomy)
}

pub fn bars<M>(c: Palette, values: Vec<f32>) -> Node<M> {
    let k = theme::sheet();
    let (tall, most, least, gap, corner) = (k.bars, k.bars_width, k.bars_least, k.bars_gap, k.bars_corner);
    strip(tall, move |s| {
        let n = values.len().max(1);
        let w = ((s.wide() - gap * (n - 1) as f32) / n as f32).clamp(1.0, most);
        let track = s.track();
        for (i, v) in values.iter().enumerate() {
            let h = (v.clamp(0.0, 1.0) * tall).max(least);
            s.rect((i as f32 * (w + gap), 0.0, w, tall), corner, track);
            s.rect((i as f32 * (w + gap), tall - h, w, h), corner, c.colour());
        }
    })
}

pub const WORRY: (f64, f64) = (0.85, 0.95);

pub fn worry(share: f64) -> Option<Colour> {
    if share >= WORRY.1 {
        Some(theme::problem())
    } else if share >= WORRY.0 {
        Some(theme::warn())
    } else {
        None
    }
}

pub fn fill_bar<M>(share: Option<f64>, tip: impl Into<String>) -> Node<M> {
    fill_bar_as(share, tip, true)
}

pub fn fill_bar_good<M>(share: Option<f64>, tip: impl Into<String>) -> Node<M> {
    fill_bar_as(share, tip, false)
}

pub fn fill_colour(share: f64, worried: bool) -> Option<Colour> {
    if worried { worry(share) } else { None }
}

fn fill_bar_as<M>(share: Option<f64>, tip: impl Into<String>, worried: bool) -> Node<M> {
    let tall = theme::sheet().fill_bar;
    drawing((120.0, tall), tip, move |canvas, pen| {
        let mut s = Sketch::strip(canvas, pen, tall);
        let (wide, track) = (s.wide(), s.track());
        s.rect((0.0, 0.0, wide, tall), tall / 2.0, track);
        if let Some(share) = share {
            let colour = fill_colour(share, worried).unwrap_or_else(|| s.highlight());
            let across = (wide * share.clamp(0.0, 1.0) as f32).max(tall).min(wide);
            s.rect((0.0, 0.0, across, tall), tall / 2.0, colour);
        }
    }).progress(share.map(|v| v.clamp(0.0, 1.0) as f32)).width(Length::Fill).height(Length::Fixed(tall))
}

pub fn standard_ring<M>(share: Option<f32>, across: f32) -> Node<M> {
    let figure = share.map(|v| format!("{}%", (v.clamp(0.0, 1.0) * 100.0).round()));
    standard_ring_with(share, figure, across)
}

pub fn standard_ring_with<M>(share: Option<f32>, figure: Option<String>, across: f32) -> Node<M> {
    let ring = drawing((across, across), "", move |canvas, pen| {
        let mut s = Sketch::fit(canvas, pen, (84.0, 84.0));
        let track = s.track();
        s.ring((42.0, 42.0), 34.0, 15.0, track);
        let hl = s.highlight();
        match share {
            Some(v) => s.arc((42.0, 42.0), 34.0, 15.0, 0.0, v, hl),
            None => {
                static START: std::sync::OnceLock<std::time::Instant> = std::sync::OnceLock::new();
                const TURN_MS: u128 = 1200;
                let from = if super::motion::reduced() { 0.0 } else { (START.get_or_init(std::time::Instant::now).elapsed().as_millis() % TURN_MS) as f32 / TURN_MS as f32 };
                s.arc((42.0, 42.0), 34.0, 15.0, from, 0.3, hl);
            }
        }
    })
    .progress(share)
    .called(crate::t!("kit-progress"));
    match figure {
        Some(f) => row(vec![super::widget::title(f), ring]).gap(crate::theme::gaps().roomy).centred(),
        None => ring,
    }
}

pub fn time_pie<M>(share: f32, across: f32, colours: impl Fn(&theme::Theme) -> (Colour, Colour) + 'static) -> Node<M> {
    drawing((across, across), "", move |canvas, pen| {
        let (disc, pie) = colours(pen.theme);
        let mut s = Sketch::fit(canvas, pen, (84.0, 84.0));
        s.circle((42.0, 42.0), 40.0, disc);
        s.pie((42.0, 42.0), 40.0, share, pie);
    })
}

pub fn picture<M>(icon: impl Into<String>, name: impl Into<String>, art: Node<M>) -> Node<M> {
    let b = theme::bento();
    let (p, words) = (b.picture, b.words);
    card(column(vec![tile_head(icon, name, Vec::new()), art.height(Length::Fill)]).gap(crate::theme::gaps().large))
        .height(Length::Fill).beside().span_grows(p.0, p.1, words)
}

pub fn stat<M>(icon: impl Into<String>, name: impl Into<String>, number: impl Into<String>, words: impl Into<String>,
               vis: Option<Node<M>>) -> Node<M> {
    let b = theme::bento();
    let (f, least) = (b.figure, b.figure_words);
    let mut all = vec![tile_head(icon, name, Vec::new()), space().height(Length::Fill), figure(number)];
    all.extend(vis);
    all.push(space().height(Length::Fill));
    all.push(note(words));
    card(column(all).gap(crate::theme::gaps().roomy)).height(Length::Fill).beside().span_grows(f.0, f.1, least)
}

fn radio_dot<M>(on: bool) -> Node<M> {
    let side = theme::sheet().radio;
    drawing((side, side), "", move |canvas, pen| {
        let mut s = Sketch::fit(canvas, pen, (side, side));
        let track = s.track();
        let outer = crate::theme::sheet().corner_of(crate::theme::Part::Radio);
        s.rect((0.0, 0.0, side, side), outer, track);
        if on {
            let hl = s.highlight();
            s.rect((side / 4.0, side / 4.0, side / 2.0, side / 2.0), outer / 2.0, hl);
        }
    })
}

pub fn tick_box<M>(on: bool) -> Node<M> {
    let side = theme::sheet().tick;
    drawing((side, side), "", move |canvas, pen| {
        let mut s = Sketch::fit(canvas, pen, (side, side));
        let corner = crate::theme::sheet().corner_of(crate::theme::Part::Tick);
        let back = if on { s.highlight() } else { s.track() };
        s.rect((0.0, 0.0, side, side), corner, back);
        if on {
            let ink = s.words_on(back);
            let at = |x: f32, y: f32| (x * side / 18.0, y * side / 18.0);
            s.line(at(4.5, 9.5), at(7.8, 12.8), side / 9.0, ink);
            s.line(at(7.8, 12.8), at(13.5, 5.5), side / 9.0, ink);
        }
    })
}

pub fn radio_group<M: Clone + 'static>(options: Vec<(String, String)>, chosen: usize, choose: impl Fn(usize) -> M) -> Node<M> {
    let rows = options.into_iter().enumerate().map(|(i, (name, sub))| {
        let said = if sub.is_empty() { name.clone() } else { format!("{name}. {sub}") };
        let mut words = vec![super::widget::form_label(name)];
        if !sub.is_empty() {
            words.push(note(sub));
        }
        let row = row(vec![radio_dot(i == chosen), column(words).gap(crate::theme::gaps().small).width(Length::Fill)]).gap(crate::theme::gaps().roomy).centred();
        card(row).flat().width(Length::Fill).on_press(choose(i)).called(said).radio(i == chosen)
    }).collect();
    column(rows).gap(crate::theme::gaps().medium)
}

pub fn empty_state<M>(sentence: impl Into<String>, next: Option<Node<M>>) -> Node<M> {
    let mut all = vec![note(sentence)];
    all.extend(next);
    column(all).gap(crate::theme::gaps().roomy)
}

pub fn loading_state<M>(height: f32) -> Node<M> {
    row(vec![space(), standard_ring(None, 40.0), space()]).height(Length::Fixed(height))
}

pub fn empty_picture<M>(icon: impl Into<String>, sentence: impl Into<String>, next: Option<Node<M>>) -> Node<M> {
    let mut all = vec![icon_image(icon, 48.0, super::widget::Tone::Calm), note(sentence)];
    all.extend(next);
    column(all).gap(crate::theme::gaps().roomy).centred()
}

pub fn skeleton<M>(lines: usize) -> Node<M> {
    let k = theme::sheet();
    let (line, gap, last) = (k.skeleton_line, k.skeleton_gap, k.skeleton_last);
    let tall = lines as f32 * (line + gap) - gap;
    drawing((120.0, tall.max(line)), "", move |canvas, pen| {
        let mut s = Sketch::strip(canvas, pen, tall.max(line));
        let (wide, track) = (s.wide(), s.track());
        let corner = crate::theme::sheet().corner_of(crate::theme::Part::Chip);
        for i in 0..lines {
            let w = if i + 1 == lines && lines > 1 { wide * last } else { wide };
            s.rect((0.0, i as f32 * (line + gap), w, line), corner, track);
        }
    }).width(Length::Fill).called(crate::t!("kit-loading")).busy()
}

pub fn error_state<M>(words: impl Into<String>, retry: Node<M>) -> Node<M> {
    let said = note(words).tone(super::widget::Tone::Problem);
    row(vec![said.width(Length::Fill), retry]).gap(crate::theme::gaps().roomy).centred()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_fullness_is_a_worry_only_from_nearly_full() {
        assert!(worry(0.0).is_none() && worry(WORRY.0 - 0.01).is_none());
        assert!(worry(WORRY.0).is_some() && worry(WORRY.1 - 0.01).is_some());
        assert_ne!(worry(WORRY.0).map(|c| c.css()), worry(WORRY.1).map(|c| c.css()));
        assert!(worry(1.5).is_some());
    }

    #[test]
    fn a_full_bar_where_more_is_better_is_not_the_worry_colour() {
        assert!(fill_colour(1.0, false).is_none());
        assert!(fill_colour(1.0, true).is_some());
        assert!(fill_colour(0.5, true).is_none());
    }
}
