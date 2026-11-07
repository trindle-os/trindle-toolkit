//! One drawing API: a `Painter` records a frame's drawing as a list for the processor (or later the GPU) to play.
#![allow(dead_code)]

pub(crate) mod cpu;
mod damage;
pub(crate) mod gpu;
mod drawer;
mod list;
mod link;
pub(crate) mod log;
pub(crate) mod remote;
pub(crate) mod service;
pub(crate) mod wire;

use std::rc::Rc;

use tiny_skia::{FillRule, IntRect, Path, Pixmap, PixmapMut, Stroke};

use crate::colour::Colour;
use crate::shadow::Shadows;
use crate::theme::Fall;
use crate::widget::Rect;

pub(crate) use damage::{changes, Seen};
pub(crate) use drawer::{gpu_for, way_for, Draws, Drawing, Painting};
pub use drawer::{drawn_by, DrawnBy, Why};
#[cfg(test)]
pub(crate) use drawer::here;
pub(crate) use list::{Glyph, How, Op};

#[derive(Default)]
pub struct Painter<'a> {
    pub(crate) ops: Vec<Op<'a>>,
    pub(crate) marks: Vec<damage::Mark>,
}

impl<'a> Painter<'a> {
    pub fn fill(&mut self, r: Rect, colour: Colour) {
        self.ops.push(Op::Fill { r, colour });
    }

    pub fn round(&mut self, r: Rect, radii: [f32; 4], colour: Colour) {
        self.ops.push(Op::Round { r, radii, colour });
    }

    pub fn line(&mut self, from: (f32, f32), to: (f32, f32), width: f32, colour: Colour) {
        self.ops.push(Op::Line { from, to, width, colour });
    }

    pub fn fill_path(&mut self, path: Path, rule: FillRule, colour: Colour) {
        self.ops.push(Op::Path { path, how: How::Fill(rule), colour });
    }

    pub fn stroke_path(&mut self, path: Path, stroke: Stroke, colour: Colour) {
        self.ops.push(Op::Path { path, how: How::Stroke(stroke), colour });
    }

    pub(crate) fn glyphs(&mut self, run: Vec<Glyph>, colour: Colour, table: Rc<[u8; 256]>) {
        self.ops.push(Op::Glyphs { run, colour, table });
    }

    pub(crate) fn image(&mut self, id: u64, pixmap: Rc<Pixmap>, x: i32, y: i32) {
        self.ops.push(Op::Image { id, pixmap, x, y });
    }

    pub(crate) fn shadow(&mut self, r: Rect, radius: f32, falls: &[Fall], colour: Colour) {
        self.ops.push(Op::Shadow { r, radius, falls: falls.to_vec(), colour });
    }

    pub(crate) fn shadow_inside(&mut self, r: Rect, radius: f32, fall: Fall, colour: Colour) {
        self.ops.push(Op::Inside { r, radius, fall, colour });
    }

    pub fn push_clip(&mut self, at: IntRect, radius: f32) {
        self.ops.push(Op::Push { at, opacity: 1.0, radius });
    }

    pub fn push_layer(&mut self, at: IntRect, opacity: f32) {
        self.ops.push(Op::Push { at, opacity, radius: 0.0 });
    }

    pub fn pop(&mut self) {
        self.ops.push(Op::Pop);
    }

    pub(crate) fn mark(&mut self, key: u64, bounds: Option<IntRect>) {
        self.marks.push(damage::Mark { at: self.ops.len(), key, bounds });
    }

    pub fn pixels(&mut self, at: IntRect, draw: impl FnOnce(&mut PixmapMut) + 'a) {
        self.ops.push(Op::Pixels { at, draw: Box::new(draw) });
    }

    pub fn play(self, canvas: &mut PixmapMut) {
        SHADOWS.with(|s| cpu::play(self.ops, canvas, s));
    }
}

thread_local! {
    static SHADOWS: Shadows = Shadows::default();
}

pub(crate) fn played<'a>(canvas: &mut PixmapMut, shadows: &Shadows, record: impl FnOnce(&mut Painter<'a>)) {
    let mut p = Painter::default();
    record(&mut p);
    cpu::play(p.ops, canvas, shadows);
}

pub(crate) fn new_id() -> u64 {
    static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);
    NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
}
