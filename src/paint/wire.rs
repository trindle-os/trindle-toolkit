//! A frame's drawing list as bytes: `Encoder` writes it, `Decoder` reads it back, trusting nothing it is given.

use std::collections::{HashMap, HashSet};
use std::fmt;
use std::rc::{Rc, Weak};
use std::sync::atomic::{AtomicU64, Ordering};

use tiny_skia::{FillRule, IntRect, IntSize, LineCap, LineJoin, PathBuilder, PathSegment, Pixmap, Stroke};

use super::{Glyph, How, Op};
use crate::colour::Colour;
use crate::sharp::{Mask, MaskKey};
use crate::theme::Fall;
use crate::widget::Rect;

const MAGIC: &[u8; 4] = b"TDRW";
const VERSION: u16 = 1;
const RESET: u16 = 1;

pub(crate) const MAX_CONTROL: usize = 4 << 20;
pub(crate) const MAX_PIXEL_BYTES: usize = 40 << 20;
const MAX_OPS: usize = 1 << 18;
const MAX_UPLOADS: usize = 1 << 16;
const MAX_RUN: usize = 1 << 14;
const MAX_FRAME_GLYPHS: usize = 1 << 18;
const MAX_PATH_SEGMENTS: usize = 1 << 14;
const MAX_FRAME_SEGMENTS: usize = 1 << 20;
const MAX_FALLS: usize = 8;
const MAX_SHADOW_PIXELS: u64 = 1 << 21;
pub(crate) const MAX_FRAME_SHADOW_PIXELS: u64 = 1 << 22;
const MAX_DEPTH: usize = 32;
const MAX_SIDE: u64 = 16384;
const MAX_PIXELS: u64 = 1 << 25;
const MAX_LAYER_PIXELS: u64 = 1 << 26;
const MAX_MASK_SIDE: u64 = 4096;
const MAX_MASKS: usize = 1 << 14;
const MAX_MASK_BYTES: usize = 8 << 20;
const MAX_IMAGES: usize = 1 << 12;
const MAX_IMAGE_BYTES: usize = 64 << 20;
const MAX_TABLES: usize = 1 << 10;
const MAX_COORD: i64 = 1 << 24;
const MAX_F: f32 = 1_048_576.0;
const MAX_SIDE_F: f32 = MAX_SIDE as f32;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Error {
    Short,
    Version,
    TooBig(&'static str),
    Bad(&'static str),
    Pixels,
    Dash,
    Full,
    Failed,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::Short => f.write_str("the frame ends too soon"),
            Error::Version => f.write_str("a version of the frame format not known here"),
            Error::TooBig(what) => write!(f, "too much: {what}"),
            Error::Bad(what) => write!(f, "not valid: {what}"),
            Error::Pixels => f.write_str("a pixels op cannot be sent"),
            Error::Dash => f.write_str("a dashed stroke cannot be sent"),
            Error::Full => f.write_str("the uploads kept are full"),
            Error::Failed => f.write_str("an earlier frame failed"),
        }
    }
}

impl std::error::Error for Error {}

type Res<T> = Result<T, Error>;

pub(crate) struct Frame {
    pub(crate) control: Vec<u8>,
    pub(crate) pixels: Vec<u8>,
}

fn check_size(w: u64, h: u64) -> Res<(u32, u32)> {
    if w == 0 || h == 0 || w > MAX_SIDE || h > MAX_SIDE || w * h > MAX_PIXELS {
        return Err(Error::TooBig("picture size"));
    }
    Ok((w as u32, h as u32))
}

fn limit(v: f32, most: f32) -> Res<f32> {
    if v.is_finite() && v.abs() <= most {
        Ok(v)
    } else {
        Err(Error::Bad("number"))
    }
}

#[derive(Default)]
struct Nest(Vec<u64>);

impl Nest {
    fn push(&mut self, pixels: u64) -> Res<()> {
        if self.0.len() >= MAX_DEPTH || self.0.iter().sum::<u64>() + pixels > MAX_LAYER_PIXELS {
            return Err(Error::TooBig("layers"));
        }
        self.0.push(pixels);
        Ok(())
    }

    fn pop(&mut self) {
        self.0.pop();
    }
}

struct W(Vec<u8>);

impl W {
    fn uv(&mut self, mut v: u64) {
        while v >= 0x80 {
            self.0.push(v as u8 | 0x80);
            v >>= 7;
        }
        self.0.push(v as u8);
    }

    fn iv(&mut self, v: i32) -> Res<()> {
        if i64::from(v).abs() > MAX_COORD {
            return Err(Error::Bad("place"));
        }
        let v = i64::from(v);
        self.uv(((v << 1) ^ (v >> 63)) as u64);
        Ok(())
    }

    fn f(&mut self, v: f32) -> Res<()> {
        self.0.extend_from_slice(&limit(v, MAX_F)?.to_le_bytes());
        Ok(())
    }

    fn s(&mut self, v: f32) -> Res<()> {
        self.0.extend_from_slice(&limit(v, MAX_SIDE_F)?.to_le_bytes());
        Ok(())
    }

    fn rect(&mut self, r: Rect, side: bool) -> Res<()> {
        self.f(r.x)?;
        self.f(r.y)?;
        if side {
            self.s(r.w)?;
            self.s(r.h)
        } else {
            self.f(r.w)?;
            self.f(r.h)
        }
    }

    fn colour(&mut self, c: Colour) -> Res<()> {
        [c.r, c.g, c.b, c.a].into_iter().try_for_each(|v| self.f(if v.is_nan() { v } else { v.clamp(0.0, 1.0) }))
    }

    fn fall(&mut self, f: Fall) -> Res<()> {
        self.s(f.y)?;
        self.s(f.blur)?;
        self.s(f.spread)?;
        self.f(f.alpha)
    }

    fn size(&mut self, w: u32, h: u32) -> Res<()> {
        check_size(u64::from(w), u64::from(h))?;
        self.uv(u64::from(w));
        self.uv(u64::from(h));
        Ok(())
    }

    fn int_rect(&mut self, at: IntRect) -> Res<()> {
        self.iv(at.x())?;
        self.iv(at.y())?;
        self.size(at.width(), at.height())
    }
}

struct Rd<'a>(&'a [u8]);

impl<'a> Rd<'a> {
    fn take(&mut self, n: usize) -> Res<&'a [u8]> {
        if n > self.0.len() {
            return Err(Error::Short);
        }
        let (head, rest) = self.0.split_at(n);
        self.0 = rest;
        Ok(head)
    }

    fn u8(&mut self) -> Res<u8> {
        Ok(self.take(1)?[0])
    }

    fn uv(&mut self) -> Res<u64> {
        let mut v = 0u64;
        for shift in (0..64).step_by(7) {
            let b = self.u8()?;
            if shift == 63 && b > 1 {
                break;
            }
            v |= u64::from(b & 0x7f) << shift;
            if b & 0x80 == 0 {
                return Ok(v);
            }
        }
        Err(Error::Bad("number"))
    }

    fn uv_to(&mut self, most: u64) -> Res<u64> {
        let v = self.uv()?;
        if v > most {
            return Err(Error::TooBig("number"));
        }
        Ok(v)
    }

    fn count(&mut self, most: usize, each: usize) -> Res<usize> {
        let n = self.uv_to(most as u64)? as usize;
        if n * each > self.0.len() {
            return Err(Error::Short);
        }
        Ok(n)
    }

    fn iv(&mut self) -> Res<i32> {
        let u = self.uv()?;
        let v = (u >> 1) as i64 ^ -((u & 1) as i64);
        if !(-MAX_COORD..=MAX_COORD).contains(&v) {
            return Err(Error::Bad("place"));
        }
        Ok(v as i32)
    }

    fn f(&mut self) -> Res<f32> {
        limit(f32::from_le_bytes(self.take(4)?.try_into().map_err(|_| Error::Short)?), MAX_F)
    }

    fn s(&mut self) -> Res<f32> {
        limit(f32::from_le_bytes(self.take(4)?.try_into().map_err(|_| Error::Short)?), MAX_SIDE_F)
    }

    fn rect(&mut self, side: bool) -> Res<Rect> {
        let (x, y) = (self.f()?, self.f()?);
        let (w, h) = if side { (self.s()?, self.s()?) } else { (self.f()?, self.f()?) };
        Ok(Rect { x, y, w, h })
    }

    fn colour(&mut self) -> Res<Colour> {
        let c = Colour { r: self.f()?, g: self.f()?, b: self.f()?, a: self.f()? };
        if [c.r, c.g, c.b, c.a].iter().any(|v| !(0.0..=1.0).contains(v)) {
            return Err(Error::Bad("colour"));
        }
        Ok(c)
    }

    fn fall(&mut self) -> Res<Fall> {
        Ok(Fall { y: self.s()?, blur: self.s()?, spread: self.s()?, alpha: self.f()? })
    }

    fn size(&mut self) -> Res<(u32, u32)> {
        let w = self.uv()?;
        check_size(w, self.uv()?)
    }

    fn int_rect(&mut self) -> Res<IntRect> {
        let (x, y) = (self.iv()?, self.iv()?);
        let (w, h) = self.size()?;
        IntRect::from_xywh(x, y, w, h).ok_or(Error::Bad("box"))
    }
}

const FILL: u8 = 0;
const ROUND: u8 = 1;
const LINE: u8 = 2;
const PATH: u8 = 3;
const GLYPHS: u8 = 4;
const IMAGE: u8 = 5;
const SHADOW: u8 = 6;
const INSIDE: u8 = 7;
const PUSH: u8 = 8;
const POP: u8 = 9;
const LAID: u8 = 10;

const UP_MASK: u8 = 0;
const UP_IMAGE: u8 = 1;
const UP_TABLE: u8 = 2;

const HOW_WINDING: u8 = 0;
const HOW_EVEN_ODD: u8 = 1;
const HOW_STROKE: u8 = 2;

impl W {
    fn how(&mut self, how: &How) -> Res<()> {
        match how {
            How::Fill(FillRule::Winding) => self.0.push(HOW_WINDING),
            How::Fill(FillRule::EvenOdd) => self.0.push(HOW_EVEN_ODD),
            How::Stroke(s) => {
                if s.dash.is_some() {
                    return Err(Error::Dash);
                }
                self.0.push(HOW_STROKE);
                self.f(s.width)?;
                self.f(s.miter_limit)?;
                self.0.push(s.line_cap as u8);
                self.0.push(s.line_join as u8);
            }
        }
        Ok(())
    }

    fn path(&mut self, path: &tiny_skia::Path) -> Res<()> {
        if path.len() > MAX_PATH_SEGMENTS {
            return Err(Error::TooBig("path"));
        }
        self.uv(path.len() as u64);
        for seg in path.segments() {
            let points: &[tiny_skia::Point] = match &seg {
                PathSegment::MoveTo(a) | PathSegment::LineTo(a) => &[*a],
                PathSegment::QuadTo(a, b) => &[*a, *b],
                PathSegment::CubicTo(a, b, c) => &[*a, *b, *c],
                PathSegment::Close => &[],
            };
            self.0.push(match seg {
                PathSegment::MoveTo(_) => 0,
                PathSegment::LineTo(_) => 1,
                PathSegment::QuadTo(..) => 2,
                PathSegment::CubicTo(..) => 3,
                PathSegment::Close => 4,
            });
            points.iter().try_for_each(|p| self.f(p.x).and_then(|()| self.f(p.y)))?;
        }
        Ok(())
    }

    fn shadow(&mut self, r: Rect, radius: f32, falls: &[Fall], colour: Colour) -> Res<()> {
        if falls.len() > MAX_FALLS {
            return Err(Error::TooBig("shadows"));
        }
        self.0.push(SHADOW);
        self.rect(r, true)?;
        self.s(radius)?;
        self.colour(colour)?;
        self.uv(falls.len() as u64);
        falls.iter().try_for_each(|f| self.fall(*f))
    }

    fn inside(&mut self, r: Rect, radius: f32, fall: Fall, colour: Colour) -> Res<()> {
        self.0.push(INSIDE);
        self.rect(r, true)?;
        self.s(radius)?;
        self.fall(fall)?;
        self.colour(colour)
    }
}

struct Out {
    uploads: W,
    n_uploads: u64,
    upload_pixels: Vec<u8>,
    ops: W,
    n_ops: u64,
    laid_pixels: Vec<u8>,
    nest: Nest,
}

#[derive(Default)]
pub(crate) struct Encoder {
    masks: HashMap<usize, (u32, Weak<Mask>)>,
    mask_bytes: usize,
    tables: HashMap<usize, (u32, Weak<[u8; 256]>)>,
    images: HashSet<u64>,
    image_bytes: usize,
    reset: bool,
}

impl Encoder {
    pub(crate) fn start_again(&mut self) {
        self.reset = true;
    }

    pub(crate) fn frame(&mut self, ops: &[Op]) -> Res<Frame> {
        let mut reset = self.reset;
        loop {
            if reset {
                self.masks.clear();
                self.tables.clear();
                self.images.clear();
                (self.mask_bytes, self.image_bytes) = (0, 0);
            }
            match self.build(ops, reset) {
                Ok(frame) => {
                    self.reset = false;
                    return Ok(frame);
                }
                Err(Error::Full) if !reset => reset = true,
                Err(e) => {
                    self.reset = true;
                    return Err(if e == Error::Full { Error::TooBig("frame") } else { e });
                }
            }
        }
    }

    fn build(&mut self, ops: &[Op], reset: bool) -> Res<Frame> {
        if ops.len() > MAX_OPS {
            return Err(Error::TooBig("ops"));
        }
        let mut out = Out {
            uploads: W(Vec::new()),
            n_uploads: 0,
            upload_pixels: Vec::new(),
            ops: W(Vec::new()),
            n_ops: 0,
            laid_pixels: Vec::new(),
            nest: Nest::default(),
        };
        for op in ops {
            self.op(&mut out, op)?;
            out.n_ops += 1;
        }
        let mut control = Vec::with_capacity(16 + out.uploads.0.len() + out.ops.0.len());
        control.extend_from_slice(MAGIC);
        control.extend_from_slice(&VERSION.to_le_bytes());
        control.extend_from_slice(&(if reset { RESET } else { 0 }).to_le_bytes());
        let mut head = W(control);
        head.uv(out.n_uploads);
        head.0.extend_from_slice(&out.uploads.0);
        head.uv(out.n_ops);
        head.0.extend_from_slice(&out.ops.0);
        let mut pixels = out.upload_pixels;
        pixels.extend_from_slice(&out.laid_pixels);
        if head.0.len() > MAX_CONTROL || pixels.len() > MAX_PIXEL_BYTES {
            return Err(Error::TooBig("frame"));
        }
        Ok(Frame { control: head.0, pixels })
    }

    fn mask(&mut self, out: &mut Out, m: &Rc<Mask>) -> Res<u32> {
        let at = Rc::as_ptr(m) as usize;
        if let Some((id, _)) = self.masks.get(&at) {
            return Ok(*id);
        }
        let (left, top, w, h, cover) = m.parts();
        if w as u64 > MAX_MASK_SIDE || h as u64 > MAX_MASK_SIDE {
            return Err(Error::TooBig("letter size"));
        }
        if self.masks.len() >= MAX_MASKS || self.mask_bytes + cover.len() > MAX_MASK_BYTES {
            return Err(Error::Full);
        }
        let id = self.masks.len() as u32;
        out.uploads.0.push(UP_MASK);
        out.uploads.uv(u64::from(id));
        out.uploads.iv(left)?;
        out.uploads.iv(top)?;
        out.uploads.uv(w as u64);
        out.uploads.uv(h as u64);
        out.upload_pixels.extend_from_slice(cover);
        out.n_uploads += 1;
        self.mask_bytes += cover.len();
        self.masks.insert(at, (id, Rc::downgrade(m)));
        Ok(id)
    }

    fn table(&mut self, out: &mut Out, t: &Rc<[u8; 256]>) -> Res<u32> {
        let at = Rc::as_ptr(t) as usize;
        if let Some((id, _)) = self.tables.get(&at) {
            return Ok(*id);
        }
        if self.tables.len() >= MAX_TABLES {
            return Err(Error::Full);
        }
        let id = self.tables.len() as u32;
        out.uploads.0.push(UP_TABLE);
        out.uploads.uv(u64::from(id));
        out.uploads.0.extend_from_slice(&**t);
        out.n_uploads += 1;
        self.tables.insert(at, (id, Rc::downgrade(t)));
        Ok(id)
    }

    fn image(&mut self, out: &mut Out, id: u64, pixmap: &Pixmap) -> Res<()> {
        if self.images.contains(&id) {
            return Ok(());
        }
        let len = pixmap.data().len();
        if self.images.len() >= MAX_IMAGES || self.image_bytes + len > MAX_IMAGE_BYTES {
            return Err(Error::Full);
        }
        out.uploads.0.push(UP_IMAGE);
        out.uploads.uv(id);
        out.uploads.size(pixmap.width(), pixmap.height())?;
        out.upload_pixels.extend_from_slice(pixmap.data());
        out.n_uploads += 1;
        self.image_bytes += len;
        self.images.insert(id);
        Ok(())
    }

    fn glyphs(&mut self, out: &mut Out, run: &[Glyph], colour: Colour, table: &Rc<[u8; 256]>) -> Res<()> {
        if run.len() > MAX_RUN {
            return Err(Error::TooBig("letters"));
        }
        let table = self.table(out, table)?;
        out.ops.0.push(GLYPHS);
        out.ops.colour(colour)?;
        out.ops.uv(u64::from(table));
        out.ops.uv(run.len() as u64);
        for g in run {
            let id = self.mask(out, &g.mask)?;
            out.ops.uv(u64::from(id));
            out.ops.iv(g.x)?;
            out.ops.iv(g.y)?;
        }
        Ok(())
    }

    fn op(&mut self, out: &mut Out, op: &Op) -> Res<()> {
        match op {
            Op::Fill { r, colour } => {
                out.ops.0.push(FILL);
                out.ops.rect(*r, false)?;
                out.ops.colour(*colour)
            }
            Op::Round { r, radii, colour } => {
                out.ops.0.push(ROUND);
                out.ops.rect(*r, false)?;
                radii.iter().try_for_each(|v| out.ops.f(*v))?;
                out.ops.colour(*colour)
            }
            Op::Line { from, to, width, colour } => {
                out.ops.0.push(LINE);
                [from.0, from.1, to.0, to.1, *width].into_iter().try_for_each(|v| out.ops.f(v))?;
                out.ops.colour(*colour)
            }
            Op::Path { path, how, colour } => {
                out.ops.0.push(PATH);
                out.ops.how(how)?;
                out.ops.colour(*colour)?;
                out.ops.path(path)
            }
            Op::Glyphs { run, colour, table } => self.glyphs(out, run, *colour, table),
            Op::Image { id, pixmap, x, y } => {
                self.image(out, *id, pixmap)?;
                out.ops.0.push(IMAGE);
                out.ops.uv(*id);
                out.ops.iv(*x)?;
                out.ops.iv(*y)
            }
            Op::Shadow { r, radius, falls, colour } => out.ops.shadow(*r, *radius, falls, *colour),
            Op::Inside { r, radius, fall, colour } => out.ops.inside(*r, *radius, *fall, *colour),
            Op::Push { at, opacity, radius } => {
                out.nest.push(u64::from(at.width()) * u64::from(at.height()))?;
                out.ops.0.push(PUSH);
                out.ops.int_rect(*at)?;
                out.ops.f(*opacity)?;
                out.ops.f(*radius)
            }
            Op::Pop => {
                out.nest.pop();
                out.ops.0.push(POP);
                Ok(())
            }
            Op::Pixels { .. } => Err(Error::Pixels),
            Op::Laid { x, y, pixmap } => {
                out.ops.0.push(LAID);
                out.ops.iv(*x)?;
                out.ops.iv(*y)?;
                out.ops.size(pixmap.width(), pixmap.height())?;
                out.laid_pixels.extend_from_slice(pixmap.data());
                Ok(())
            }
        }
    }
}

fn fresh() -> u64 {
    static NEXT: AtomicU64 = AtomicU64::new(1);
    NEXT.fetch_add(1, Ordering::Relaxed)
}

#[derive(Default)]
pub(crate) struct Decoder {
    masks: HashMap<u32, (MaskKey, Rc<Mask>)>,
    mask_bytes: usize,
    tables: HashMap<u32, Rc<[u8; 256]>>,
    images: HashMap<u64, (u64, Rc<Pixmap>)>,
    image_bytes: usize,
    failed: bool,
    new_masks: Vec<u32>,
    new_images: Vec<u64>,
    was_reset: bool,
    layer_most: Option<u64>,
    hollow_picture: Option<Rc<Pixmap>>,
}

pub(crate) struct News {
    pub(crate) reset: bool,
    pub(crate) masks: Vec<(MaskKey, Rc<Mask>, Rc<Mask>)>,
    pub(crate) images: Vec<(u64, Rc<Pixmap>)>,
}

#[derive(Default)]
struct Used {
    glyphs: usize,
    segments: usize,
    nest: Nest,
}

fn check_shadow(inside: bool, r: Rect, radius: f32, falls: &[Fall]) -> Res<()> {
    if crate::shadow::mask_pixels(inside, r, radius, falls) > MAX_SHADOW_PIXELS {
        return Err(Error::TooBig("shadows"));
    }
    Ok(())
}

impl Decoder {
    pub(crate) fn hold_to(&mut self, w: u32, h: u32) {
        self.layer_most = Some((u64::from(w) * u64::from(h) * 4).clamp(1 << 20, MAX_PIXELS));
    }

    pub(crate) fn news(&mut self) -> Option<News> {
        let stand_in = match &self.hollow_picture {
            Some(p) => p.clone(),
            None => Rc::new(Pixmap::new(1, 1)?),
        };
        self.hollow_picture = Some(stand_in.clone());
        let masks = std::mem::take(&mut self.new_masks).into_iter().filter_map(|id| {
            let (key, mask) = self.masks.get_mut(&id)?;
            let hollow = Rc::new(mask.hollow());
            Some((*key, std::mem::replace(mask, hollow.clone()), hollow))
        }).collect();
        let images = std::mem::take(&mut self.new_images).into_iter().filter_map(|id| {
            let (name, pixmap) = self.images.get_mut(&id)?;
            Some((*name, std::mem::replace(pixmap, stand_in.clone())))
        }).collect();
        Some(News { reset: std::mem::take(&mut self.was_reset), masks, images })
    }

    pub(crate) fn frame(&mut self, control: &[u8], pixels: &[u8]) -> Res<Vec<Op<'static>>> {
        if self.failed {
            return Err(Error::Failed);
        }
        let ops = self.read(control, pixels);
        self.failed = ops.is_err();
        ops
    }

    fn read(&mut self, control: &[u8], pixels: &[u8]) -> Res<Vec<Op<'static>>> {
        if control.len() > MAX_CONTROL || pixels.len() > MAX_PIXEL_BYTES {
            return Err(Error::TooBig("frame"));
        }
        let (mut r, mut px) = (Rd(control), Rd(pixels));
        if r.take(4)? != MAGIC {
            return Err(Error::Bad("not a frame"));
        }
        if u16::from_le_bytes([r.u8()?, r.u8()?]) != VERSION {
            return Err(Error::Version);
        }
        let flags = u16::from_le_bytes([r.u8()?, r.u8()?]);
        if flags & !RESET != 0 {
            return Err(Error::Bad("flags"));
        }
        if flags & RESET != 0 {
            *self = Decoder { was_reset: true, layer_most: self.layer_most, hollow_picture: self.hollow_picture.take(), ..Decoder::default() };
        }
        for _ in 0..r.count(MAX_UPLOADS, 4)? {
            self.upload(&mut r, &mut px)?;
        }
        let mut used = Used::default();
        let mut ops = Vec::new();
        for _ in 0..r.count(MAX_OPS, 1)? {
            ops.push(self.op(&mut r, &mut px, &mut used)?);
        }
        if !r.0.is_empty() || !px.0.is_empty() {
            return Err(Error::Bad("bytes left over"));
        }
        Ok(ops)
    }

    fn upload(&mut self, r: &mut Rd, px: &mut Rd) -> Res<()> {
        match r.u8()? {
            UP_MASK => {
                let id = r.uv_to(u64::from(u32::MAX))? as u32;
                let (left, top) = (r.iv()?, r.iv()?);
                let (w, h) = (r.uv_to(MAX_MASK_SIDE)? as usize, r.uv_to(MAX_MASK_SIDE)? as usize);
                if self.masks.contains_key(&id) {
                    return Err(Error::Bad("letter id used twice"));
                }
                if self.masks.len() >= MAX_MASKS || self.mask_bytes + w * h > MAX_MASK_BYTES {
                    return Err(Error::TooBig("letters kept"));
                }
                let cover = Box::from(px.take(w * h)?);
                self.mask_bytes += w * h;
                let name = fresh();
                let key = MaskKey { font: fontdb::ID::dummy(), glyph: (name >> 32) as u16, size: name as u32, weight: 0, lean: false };
                self.masks.insert(id, (key, Rc::new(Mask::from_parts(left, top, w, h, cover))));
                self.new_masks.push(id);
            }
            UP_IMAGE => {
                let id = r.uv()?;
                let (w, h) = r.size()?;
                let len = w as usize * h as usize * 4;
                if self.images.contains_key(&id) {
                    return Err(Error::Bad("picture id used twice"));
                }
                if self.images.len() >= MAX_IMAGES || self.image_bytes + len > MAX_IMAGE_BYTES {
                    return Err(Error::TooBig("pictures kept"));
                }
                let pixmap = pixmap(px, w, h)?;
                self.image_bytes += len;
                self.images.insert(id, (fresh(), Rc::new(pixmap)));
                self.new_images.push(id);
            }
            UP_TABLE => {
                let id = r.uv_to(u64::from(u32::MAX))? as u32;
                if self.tables.contains_key(&id) {
                    return Err(Error::Bad("table id used twice"));
                }
                if self.tables.len() >= MAX_TABLES {
                    return Err(Error::TooBig("tables kept"));
                }
                let table = <[u8; 256]>::try_from(r.take(256)?).map_err(|_| Error::Short)?;
                self.tables.insert(id, Rc::new(table));
            }
            _ => return Err(Error::Bad("upload")),
        }
        Ok(())
    }

    fn glyphs(&self, r: &mut Rd, used: &mut Used) -> Res<Op<'static>> {
        let colour = r.colour()?;
        let table = r.uv_to(u64::from(u32::MAX))? as u32;
        let table = self.tables.get(&table).ok_or(Error::Bad("table not sent"))?.clone();
        let n = r.count(MAX_RUN, 3)?;
        used.glyphs += n;
        if used.glyphs > MAX_FRAME_GLYPHS {
            return Err(Error::TooBig("letters"));
        }
        let mut run = Vec::with_capacity(n);
        for _ in 0..n {
            let id = r.uv_to(u64::from(u32::MAX))? as u32;
            let (key, mask) = self.masks.get(&id).map(|(k, m)| (*k, m.clone())).ok_or(Error::Bad("letter not sent"))?;
            run.push(Glyph { key, mask, x: r.iv()?, y: r.iv()? });
        }
        Ok(Op::Glyphs { run, colour, table })
    }

    fn op(&mut self, r: &mut Rd, px: &mut Rd, used: &mut Used) -> Res<Op<'static>> {
        Ok(match r.u8()? {
            FILL => Op::Fill { r: r.rect(false)?, colour: r.colour()? },
            ROUND => {
                let rect = r.rect(false)?;
                Op::Round { r: rect, radii: [r.f()?, r.f()?, r.f()?, r.f()?], colour: r.colour()? }
            }
            LINE => {
                let (from, to) = ((r.f()?, r.f()?), (r.f()?, r.f()?));
                Op::Line { from, to, width: r.f()?, colour: r.colour()? }
            }
            PATH => path(r, used)?,
            GLYPHS => self.glyphs(r, used)?,
            IMAGE => {
                let id = r.uv()?;
                let (id, pixmap) = self.images.get(&id).map(|(n, p)| (*n, p.clone())).ok_or(Error::Bad("picture not sent"))?;
                Op::Image { id, pixmap, x: r.iv()?, y: r.iv()? }
            }
            SHADOW => {
                let (rect, radius, colour) = (r.rect(true)?, r.s()?, r.colour()?);
                let n = r.count(MAX_FALLS, 16)?;
                let falls = (0..n).map(|_| r.fall()).collect::<Res<Vec<_>>>()?;
                check_shadow(false, rect, radius, &falls)?;
                Op::Shadow { r: rect, radius, falls, colour }
            }
            INSIDE => {
                let (rect, radius, fall) = (r.rect(true)?, r.s()?, r.fall()?);
                check_shadow(true, rect, radius, &[fall])?;
                Op::Inside { r: rect, radius, fall, colour: r.colour()? }
            }
            PUSH => {
                let at = r.int_rect()?;
                let pixels = u64::from(at.width()) * u64::from(at.height());
                used.nest.push(pixels)?;
                if self.layer_most.is_some_and(|most| pixels > most || used.nest.0.iter().sum::<u64>() > 2 * most) {
                    return Err(Error::TooBig("layers"));
                }
                Op::Push { at, opacity: r.f()?, radius: r.f()? }
            }
            POP => {
                used.nest.pop();
                Op::Pop
            }
            LAID => {
                let (x, y) = (r.iv()?, r.iv()?);
                let (w, h) = r.size()?;
                Op::Laid { x, y, pixmap: pixmap(px, w, h)? }
            }
            _ => return Err(Error::Bad("op")),
        })
    }
}

fn how(r: &mut Rd) -> Res<How> {
    Ok(match r.u8()? {
        HOW_WINDING => How::Fill(FillRule::Winding),
        HOW_EVEN_ODD => How::Fill(FillRule::EvenOdd),
        HOW_STROKE => {
            let (width, miter_limit) = (r.f()?, r.f()?);
            let line_cap = match r.u8()? {
                0 => LineCap::Butt,
                1 => LineCap::Round,
                2 => LineCap::Square,
                _ => return Err(Error::Bad("line cap")),
            };
            let line_join = match r.u8()? {
                0 => LineJoin::Miter,
                1 => LineJoin::MiterClip,
                2 => LineJoin::Round,
                3 => LineJoin::Bevel,
                _ => return Err(Error::Bad("line join")),
            };
            How::Stroke(Stroke { width, miter_limit, line_cap, line_join, dash: None })
        }
        _ => return Err(Error::Bad("path kind")),
    })
}

fn path(r: &mut Rd, used: &mut Used) -> Res<Op<'static>> {
    let how = how(r)?;
    let colour = r.colour()?;
    let n = r.count(MAX_PATH_SEGMENTS, 1)?;
    used.segments += n;
    if used.segments > MAX_FRAME_SEGMENTS {
        return Err(Error::TooBig("path segments"));
    }
    let mut b = PathBuilder::new();
    for _ in 0..n {
        match r.u8()? {
            0 => b.move_to(r.f()?, r.f()?),
            1 => b.line_to(r.f()?, r.f()?),
            2 => b.quad_to(r.f()?, r.f()?, r.f()?, r.f()?),
            3 => b.cubic_to(r.f()?, r.f()?, r.f()?, r.f()?, r.f()?, r.f()?),
            4 => b.close(),
            _ => return Err(Error::Bad("path segment")),
        }
    }
    Ok(Op::Path { path: b.finish().ok_or(Error::Bad("path"))?, how, colour })
}

fn pixmap(px: &mut Rd, w: u32, h: u32) -> Res<Pixmap> {
    let mut bytes = px.take(w as usize * h as usize * 4)?.to_vec();
    for p in bytes.chunks_exact_mut(4) {
        let a = p[3];
        p[..3].iter_mut().for_each(|c| *c = (*c).min(a));
    }
    Pixmap::from_vec(bytes, IntSize::from_wh(w, h).ok_or(Error::Bad("picture size"))?).ok_or(Error::Bad("picture"))
}

#[cfg(test)]
mod tests;
