//! Charts drawn in the sheet's colours: line, area, columns, sparkline, pie and ring, each a node that takes numbers and a name.

use std::cell::Cell;
use std::f32::consts::{FRAC_PI_2, TAU};
use std::rc::Rc;

use super::colour::Colour;
use super::theme::{self, mix, Palette};
use super::tiles::Sketch;
use super::widget::{drawing, Length, Node, Pointer};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Kind {
    Line,
    Area,
    Columns,
    Spark,
    Pie,
    Ring,
}

impl Kind {
    fn parts(self) -> bool {
        matches!(self, Kind::Pie | Kind::Ring)
    }
}

const PAD: f32 = 4.0;
const TOP: f32 = 18.0;
const UNITS: f32 = 100.0;
const ROUND: f32 = 46.0;
const HOLE: f32 = 30.0;
const GAP: f32 = 0.012;

pub struct Chart {
    kind: Kind,
    name: String,
    values: Vec<f32>,
    labels: Vec<String>,
    hover: Option<usize>,
    tall: f32,
}

impl Chart {
    fn new(kind: Kind, name: impl Into<String>, values: &[f32]) -> Chart {
        let tall = if kind == Kind::Spark { 32.0 } else { 160.0 };
        let values = values.iter().map(|v| if v.is_finite() { *v } else { 0.0 }).collect();
        Chart { kind, name: name.into(), values, labels: Vec::new(), hover: None, tall }
    }

    pub fn line(name: impl Into<String>, values: &[f32]) -> Chart {
        Chart::new(Kind::Line, name, values)
    }

    pub fn area(name: impl Into<String>, values: &[f32]) -> Chart {
        Chart::new(Kind::Area, name, values)
    }

    pub fn columns(name: impl Into<String>, values: &[f32]) -> Chart {
        Chart::new(Kind::Columns, name, values)
    }

    pub fn spark(name: impl Into<String>, values: &[f32]) -> Chart {
        Chart::new(Kind::Spark, name, values)
    }

    pub fn pie(name: impl Into<String>, values: &[f32]) -> Chart {
        Chart::new(Kind::Pie, name, values)
    }

    pub fn ring(name: impl Into<String>, values: &[f32]) -> Chart {
        Chart::new(Kind::Ring, name, values)
    }

    pub fn labels(mut self, labels: Vec<String>) -> Chart {
        self.labels = labels;
        self
    }

    pub fn hover(mut self, at: Option<usize>) -> Chart {
        self.hover = at.filter(|i| *i < self.values.len());
        self
    }

    pub fn tall(mut self, points: f32) -> Chart {
        self.tall = points.clamp(16.0, 2000.0);
        self
    }

    pub fn node<M: 'static>(self) -> Node<M> {
        self.build(None)
    }

    pub fn hovering<M: 'static>(self, call: impl Fn(Option<usize>) -> M + 'static) -> Node<M> {
        self.build(Some(Box::new(call)))
    }

    pub fn summary(&self) -> String {
        let (name, n) = (self.name.as_str(), self.values.len());
        if n == 0 {
            return crate::t!("kit-chart-empty", name = name);
        }
        if self.kind.parts() {
            let shares = shares(&self.values);
            let said: Vec<String> = shares.iter().enumerate().map(|(i, s)| format!("{} {}%", self.label(i), (s * 100.0).round())).collect();
            let args = [("name", name.into()), ("n", (n as f64).into()), ("parts", said.join(", ").into())];
            return if self.kind == Kind::Pie { crate::lang::t_with("kit-chart-pie", &args) } else { crate::lang::t_with("kit-chart-ring", &args) };
        }
        let (low, high) = self.values.iter().fold((f32::MAX, f32::MIN), |(l, h), v| (l.min(*v), h.max(*v)));
        let args = [("name", name.into()), ("n", (n as f64).into()), ("first", num(self.values[0]).into()),
                    ("last", num(self.values[n - 1]).into()), ("low", num(low).into()), ("high", num(high).into())];
        match self.kind {
            Kind::Line => crate::lang::t_with("kit-chart-line", &args),
            Kind::Area => crate::lang::t_with("kit-chart-area", &args),
            Kind::Columns => crate::lang::t_with("kit-chart-columns", &args),
            _ => crate::lang::t_with("kit-chart-spark", &args),
        }
    }

    fn label(&self, i: usize) -> String {
        self.labels.get(i).cloned().unwrap_or_else(|| crate::t!("kit-chart-part", n = i + 1))
    }

    fn build<M: 'static>(self, call: Option<Box<dyn Fn(Option<usize>) -> M>>) -> Node<M> {
        let said = self.summary();
        let (kind, tall) = (self.kind, self.tall);
        let size = Rc::new(Cell::new((0.0f32, 0.0f32)));
        let (seen, n) = (size.clone(), self.values.len());
        let (values, shares_of) = (self.values.clone(), shares(&self.values));
        let (labels, hover) = (self.labels.clone(), self.hover);
        let node = drawing::<M>((if kind.parts() { tall } else { 120.0 }, tall), said, move |canvas, pen| {
            seen.set((canvas.width() as f32 / pen.scale, canvas.height() as f32 / pen.scale));
            if kind.parts() {
                let mut s = Sketch::fit(canvas, pen, (UNITS, UNITS));
                paint_parts(&mut s, kind, &shares_of, hover);
            } else {
                let mut s = Sketch::strip(canvas, pen, tall);
                let shown = hover.filter(|_| kind != Kind::Spark).map(|i| match labels.get(i) {
                    Some(l) => format!("{l}: {}", num(values[i])),
                    None => num(values[i]),
                });
                paint_xy(&mut s, kind, tall, &values, hover, shown.as_deref());
            }
        });
        let node = if kind.parts() { node } else { node.width(Length::Fill) };
        let Some(call) = call else { return node };
        let last = Cell::new(None);
        let shares = shares(&self.values);
        node.on_hover().on_pointer_when(move |p| {
            let (w, h) = size.get();
            let at = match p {
                Pointer::Hover(x, y) => if kind.parts() { part_at(&shares, (x, y), (w, h), kind) } else { column_at(x, n, kind, plot(w, h, kind)) },
                Pointer::Leave => None,
                _ => return None,
            };
            (last.replace(at) != at).then(|| call(at))
        })
    }
}

fn num(v: f32) -> String {
    crate::lang::format::digits(&format!("{}", (v * 100.0).round() / 100.0))
}

fn shares(values: &[f32]) -> Vec<f32> {
    let total: f32 = values.iter().map(|v| v.max(0.0)).sum();
    values.iter().map(|v| if total > 0.0 { v.max(0.0) / total } else { 0.0 }).collect()
}

fn plot(w: f32, h: f32, kind: Kind) -> (f32, f32, f32, f32) {
    let top = if kind == Kind::Spark { PAD } else { TOP };
    (PAD, top, (w - 2.0 * PAD).max(1.0), (h - top - PAD).max(1.0))
}

fn x_of(i: usize, n: usize, kind: Kind, p: (f32, f32, f32, f32)) -> f32 {
    match kind {
        Kind::Columns => p.0 + p.2 * (i as f32 + 0.5) / n as f32,
        _ if n < 2 => p.0 + p.2 / 2.0,
        _ => p.0 + p.2 * i as f32 / (n - 1) as f32,
    }
}

fn column_at(x: f32, n: usize, kind: Kind, p: (f32, f32, f32, f32)) -> Option<usize> {
    if n == 0 {
        return None;
    }
    let t = ((x - p.0) / p.2).clamp(0.0, 1.0);
    Some(match kind {
        Kind::Columns => ((t * n as f32) as usize).min(n - 1),
        _ if n < 2 => 0,
        _ => (t * (n - 1) as f32).round() as usize,
    })
}

fn part_at(shares: &[f32], at: (f32, f32), (w, h): (f32, f32), kind: Kind) -> Option<usize> {
    let k = (w.min(h) / UNITS).max(f32::EPSILON);
    let (dx, dy) = ((at.0 - w / 2.0) / k, (at.1 - h / 2.0) / k);
    let r = dx.hypot(dy);
    if r > ROUND || (kind == Kind::Ring && r < HOLE) {
        return None;
    }
    let turn = (dy.atan2(dx) + FRAC_PI_2).rem_euclid(TAU) / TAU;
    let mut end = 0.0;
    shares.iter().position(|s| {
        end += s;
        *s > 0.0 && turn < end
    })
}

fn colour_of(s: &Sketch, i: usize) -> Colour {
    let sheet = theme::sheet();
    if let Some(safe) = theme::safe_now().or_else(|| sheet.colour_blind_safe.then(|| sheet.safe[usize::from(crate::common::style::light())])) {
        return mix(s.wash(Palette::Grey, 0.0), safe.series[i % 6], (1.0 - 0.3 * (i / 6) as f32).max(0.3));
    }
    let pair = theme::pair();
    let base = [s.highlight(), pair[0], pair[1]][i % 3];
    mix(s.wash(Palette::Grey, 0.0), base, (1.0 - 0.3 * (i / 3) as f32).max(0.3))
}

fn paint_parts(s: &mut Sketch, kind: Kind, shares: &[f32], hover: Option<usize>) {
    let c = UNITS / 2.0;
    let live = shares.iter().filter(|v| **v > 0.0).count();
    if live == 0 {
        let track = s.track();
        if kind == Kind::Ring {
            s.ring((c, c), (ROUND + HOLE) / 2.0, ROUND - HOLE, track);
        } else {
            s.circle((c, c), ROUND, track);
        }
        return;
    }
    let gap = if live > 1 { GAP } else { 0.0 };
    let at = |t: f32, r: f32| (c + r * (TAU * t - FRAC_PI_2).cos(), c + r * (TAU * t - FRAC_PI_2).sin());
    let mut start = 0.0;
    for (i, share) in shares.iter().enumerate() {
        let g = if *share > gap { gap } else { 0.0 };
        let (from, to) = (start + g / 2.0, start + share - g / 2.0);
        start += share;
        if *share <= 0.0 {
            continue;
        }
        let outer = if hover == Some(i) { ROUND + 3.0 } else { ROUND };
        let steps = ((96.0 * share).ceil() as usize).max(2);
        let arc = |r: f32| (0..=steps).map(|j| at(from + (to - from) * j as f32 / steps as f32, r)).collect::<Vec<_>>();
        let mut shape = arc(outer);
        if kind == Kind::Ring {
            shape.extend(arc(HOLE).into_iter().rev());
        } else {
            shape.push((c, c));
        }
        let fill = colour_of(s, i);
        s.shape(&shape, fill);
    }
    if let Some(i) = hover {
        let (spot, words) = if kind == Kind::Ring {
            ((c, c), format!("{}%", (shares[i] * 100.0).round()))
        } else {
            let mid = shares[..i].iter().sum::<f32>() + shares[i] / 2.0;
            (at(mid, ROUND * 0.62), format!("{}%", (shares[i] * 100.0).round()))
        };
        let ink = if kind == Kind::Ring { s.words() } else { s.words_on(colour_of(s, i)) };
        s.text(&words, spot, 11.0, ink);
    }
}

fn scale_ends(kind: Kind, values: &[f32]) -> (f32, f32) {
    let (mut lo, mut hi) = values.iter().fold((f32::MAX, f32::MIN), |(l, h), v| (l.min(*v), h.max(*v)));
    if matches!(kind, Kind::Area | Kind::Columns) {
        lo = lo.min(0.0);
        hi = hi.max(0.0);
    }
    if (hi - lo).abs() < f32::EPSILON {
        if matches!(kind, Kind::Line | Kind::Spark) { lo -= 1.0; hi += 1.0 } else { hi = lo + 1.0 }
    }
    (lo, hi)
}

fn paint_xy(s: &mut Sketch, kind: Kind, tall: f32, values: &[f32], hover: Option<usize>, shown: Option<&str>) {
    let wide = s.wide();
    let n = values.len();
    let p = plot(wide, tall, kind);
    let (px, py, pw, ph) = p;
    let (lo, hi) = scale_ends(kind, values);
    let y = |v: f32| py + ph - (v - lo) / (hi - lo) * ph;
    let (track, hl) = (s.track(), s.highlight());
    if kind != Kind::Spark {
        for f in [0.0, 0.5, 1.0] {
            s.line((px, py + ph * f), (px + pw, py + ph * f), 1.0, track);
        }
    }
    if n == 0 {
        return;
    }
    let dim = mix(s.wash(Palette::Grey, 0.0), hl, 0.5);
    match kind {
        Kind::Columns => {
            let cw = (pw / n as f32 * 0.6).clamp(2.0, 28.0);
            let zero = y(0.0);
            for (i, v) in values.iter().enumerate() {
                let (top, bottom) = (y(*v).min(zero), y(*v).max(zero));
                let fill = if hover.is_none_or(|h| h == i) { hl } else { dim };
                s.rect((x_of(i, n, kind, p) - cw / 2.0, top, cw, (bottom - top).max(2.0)), 3.0f32.min(cw / 2.0), fill);
            }
        }
        _ => {
            let line = theme::sheet().chart_line;
            let points: Vec<(f32, f32)> = values.iter().enumerate().map(|(i, v)| (x_of(i, n, kind, p), y(*v))).collect();
            if kind == Kind::Area && n > 1 {
                let base = y(0.0);
                let mut shape = vec![(points[0].0, base)];
                shape.extend(&points);
                shape.push((points[n - 1].0, base));
                let wash = mix(s.wash(Palette::Grey, 0.0), hl, 0.25);
                s.shape(&shape, wash);
            }
            for w in points.windows(2) {
                s.line(w[0], w[1], line, hl);
            }
            if n == 1 || kind == Kind::Spark {
                s.circle(points[n - 1], line * 1.4, hl);
            }
            if let Some(i) = hover {
                s.line((points[i].0, py), (points[i].0, py + ph), 1.0, track);
                s.circle(points[i], line * 1.8, hl);
            }
        }
    }
    if let (Some(i), Some(words)) = (hover, shown) {
        let ink = s.words();
        let x = x_of(i, n, kind, p).clamp(48.0, (wide - 48.0).max(48.0));
        s.text(words, (x, TOP / 2.0), 12.0, ink);
    }
}
