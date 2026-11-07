//! The drawing operations a `Painter` records, in pixels of the layer they are drawn on.

use std::rc::Rc;

use tiny_skia::{FillRule, IntRect, Path, Pixmap, PixmapMut, Stroke};

use crate::colour::Colour;
use crate::sharp::{Mask, MaskKey};
use crate::theme::Fall;
use crate::widget::Rect;

pub(crate) struct Glyph {
    pub(crate) key: MaskKey,
    pub(crate) mask: Rc<Mask>,
    pub(crate) x: i32,
    pub(crate) y: i32,
}

pub(crate) enum How {
    Fill(FillRule),
    Stroke(Stroke),
}

pub(crate) enum Op<'a> {
    Fill { r: Rect, colour: Colour },
    Round { r: Rect, radii: [f32; 4], colour: Colour },
    Line { from: (f32, f32), to: (f32, f32), width: f32, colour: Colour },
    Path { path: Path, how: How, colour: Colour },
    Glyphs { run: Vec<Glyph>, colour: Colour, table: Rc<[u8; 256]> },
    Image { id: u64, pixmap: Rc<Pixmap>, x: i32, y: i32 },
    Shadow { r: Rect, radius: f32, falls: Vec<Fall>, colour: Colour },
    Inside { r: Rect, radius: f32, fall: Fall, colour: Colour },
    Push { at: IntRect, opacity: f32, radius: f32 },
    Pop,
    Pixels { at: IntRect, draw: Box<dyn FnOnce(&mut PixmapMut) + 'a> },
    Laid { x: i32, y: i32, pixmap: Pixmap },
}
