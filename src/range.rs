//! A range slider: two handles on one line, the low end and the high end of a span.

use std::cell::Cell;
use std::rc::Rc;

use tiny_skia::{FillRule, Stroke, Transform};

use crate::colour::paint;
use crate::input::{Input, Keysym};
use crate::theme;
use crate::ui::{knob, rounded};
use crate::role::Role;
use crate::widget::{drawing, Length, Node, Pointer, Rect};

const PAGE: f32 = 10.0;

#[derive(Clone, Debug)]
pub struct Range {
    pub low: f32,
    pub high: f32,
    pub min: f32,
    pub max: f32,
    pub step: f32,
    pub lit: usize,
    seen: Rc<Cell<f32>>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum RangeMsg {
    Press(f32),
    Drag(f32),
    Step(f32),
    End(bool),
    Light(usize),
}

impl Range {
    pub fn new(low: f32, high: f32, min: f32, max: f32, step: f32) -> Self {
        let (min, max) = if min <= max { (min, max) } else { (max, min) };
        let mut r = Range { low: min, high: max, min, max, step, lit: 0, seen: Rc::new(Cell::new(0.0)) };
        r.low = r.fit(low.min(high));
        r.high = r.fit(high.max(low));
        r
    }

    fn fit(&self, v: f32) -> f32 {
        let v = v.clamp(self.min, self.max);
        if self.step <= 0.0 {
            return v;
        }
        ((self.min + ((v - self.min) / self.step).round() * self.step) * 1e5).round() / 1e5
    }

    fn value_at(&self, along: f32) -> f32 {
        self.fit(self.min + along.clamp(0.0, 1.0) * (self.max - self.min))
    }

    fn put(&mut self, v: f32) {
        if self.lit == 0 {
            self.low = self.fit(v).min(self.high);
        } else {
            self.high = self.fit(v).max(self.low);
        }
    }

    pub fn update(&mut self, m: RangeMsg) -> bool {
        let was = (self.low, self.high);
        match m {
            RangeMsg::Press(along) => {
                let v = self.value_at(along);
                self.lit = if (v - self.low).abs() < (v - self.high).abs() || (self.low == self.high && v < self.low) { 0 } else { 1 };
                self.put(v);
            }
            RangeMsg::Drag(along) => self.put(self.value_at(along)),
            RangeMsg::Step(n) => self.put(if self.lit == 0 { self.low } else { self.high } + n * if self.step > 0.0 { self.step } else { (self.max - self.min) / 100.0 }),
            RangeMsg::End(most) => self.put(if most { self.max } else { self.min }),
            RangeMsg::Light(i) => self.lit = i.min(1),
        }
        (self.low, self.high) != was
    }

    pub fn node<M: 'static>(&self, tip: &str, wrap: impl Fn(RangeMsg) -> M + Clone + 'static) -> Node<M> {
        let (low, high, min, max, lit) = (self.low, self.high, self.min, self.max, self.lit);
        let share = move |v: f32| if max > min { ((v - min) / (max - min)).clamp(0.0, 1.0) } else { 0.0 };
        let (seen, seen1) = (self.seen.clone(), self.seen.clone());
        let along = move |x: f32| {
            let k = theme::sheet().knob;
            (x - k / 2.0) / (seen1.get() - k).max(1.0)
        };
        let (w1, w2) = (wrap.clone(), wrap);
        drawing((theme::SLIDER_MIN_WIDTH, theme::sheet().switch.1.max(theme::TARGET_LEAST)), "", move |canvas, pen| {
            let (t, s) = (pen.theme, pen.scale);
            seen.set(canvas.width() as f32 / s);
            let (w, h) = (canvas.width() as f32, canvas.height() as f32);
            let (k, track) = ((theme::sheet().knob * s).round(), (theme::sheet().track * s).round().max(1.0));
            let mid = (h / 2.0).round();
            let corner = theme::sheet().corner_of(theme::Part::Slider) * s;
            let line = Rect { x: k / 2.0 - track / 2.0, y: mid - track / 2.0, w: w - k + track, h: track };
            let fill = |canvas: &mut tiny_skia::PixmapMut, r: Rect, c| {
                if let Some(path) = rounded(r, corner) {
                    canvas.fill_path(&path, &paint(c), FillRule::Winding, Transform::identity(), None);
                }
            };
            fill(canvas, line, t.control);
            let (a, b) = (share(low) * (w - k), share(high) * (w - k));
            fill(canvas, Rect { x: a + k / 2.0, w: b - a, ..line }, t.soft);
            for (i, x) in [a, b].into_iter().enumerate() {
                let ring = Rect { x, y: mid - k / 2.0, w: k, h: k };
                knob(canvas, ring, t, s);
                if i == lit {
                    let width = (theme::sheet().hover_edge * s).max(1.0);
                    if let Some(path) = rounded(ring.inset(width / 2.0), (k - width) / 2.0) {
                        canvas.stroke_path(&path, &paint(t.accent), &Stroke { width, ..Stroke::default() }, Transform::identity(), None);
                    }
                }
            }
        })
        .width(Length::Fill)
        .role(Role::Range { low: f64::from(low), high: f64::from(high), min: f64::from(min), max: f64::from(max), lit })
        .called(tip)
        .on_pointer_when(move |p| match p {
            Pointer::Down(x, _) => Some(w1(RangeMsg::Press(along(x)))),
            Pointer::Move(x, _) => Some(w1(RangeMsg::Drag(along(x)))),
            _ => None,
        })
        .on_keys(move |i| {
            let Input::Key { sym, mods, .. } = *i else { return None };
            let m = match sym {
                Keysym::Left | Keysym::Down | Keysym::KP_Left | Keysym::KP_Down => RangeMsg::Step(-1.0),
                Keysym::Right | Keysym::Up | Keysym::KP_Right | Keysym::KP_Up => RangeMsg::Step(1.0),
                Keysym::Page_Down | Keysym::KP_Page_Down => RangeMsg::Step(-PAGE),
                Keysym::Page_Up | Keysym::KP_Page_Up => RangeMsg::Step(PAGE),
                Keysym::Home | Keysym::KP_Home => RangeMsg::End(false),
                Keysym::End | Keysym::KP_End => RangeMsg::End(true),
                Keysym::Tab if !mods.shift && lit == 0 => RangeMsg::Light(1),
                Keysym::ISO_Left_Tab | Keysym::Tab if mods.shift && lit == 1 => RangeMsg::Light(0),
                _ => return None,
            };
            Some(w2(m))
        })
    }
}
