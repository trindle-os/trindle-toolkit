//! Plays a recorded list with OpenGL ES 3.0: every edge, letter and shadow from the masks the processor makes, so the two agree within two levels.

use std::collections::hash_map::RandomState;
use std::collections::HashMap;
use std::hash::{DefaultHasher, Hash, Hasher};
use std::os::fd::OwnedFd;
use std::rc::Rc;
use std::time::Instant;

use tiny_skia::{FillRule, IntRect, Path, PathBuilder, Pixmap, Stroke, Transform};

use super::list::{How, Op};
use super::log;
use crate::colour::{paint, Colour};
use crate::shadow::{Made, Shadows};
use crate::sharp::{Mask, MaskKey};
use crate::sys::gbm::{Device, Dmabuf};
use crate::sys::gl::{self, Context, Instances, Texel};
use crate::widget::Rect;

mod store;

pub(crate) const START: u32 = 512;
pub(crate) const SIDE: u32 = 2048;
const GROUPS: usize = 6;
const SETTLE_AFTER: u8 = 8;
const ROWS: usize = 33;
const FILL: f32 = 0.0;
const COVER: f32 = 1.0;
const PICTURE: f32 = 2.0;
const SHADOW: f32 = 3.0;
const NONE: [f32; 4] = [0.0; 4];
const NO_MASK: [f32; 4] = [-1.0, 0.0, 0.0, 0.0];
const BATCH_MOST: usize = 1 << 16;
const LOOSE_MOST: usize = 64 << 20;
const POOL_MOST: usize = 8;
const POOL_MOST_BYTES: u64 = 128 << 20;
const AREA_MOST: f64 = 64.0e6;
const SHARE_MOST: usize = 1 << 20;
const ISLAND_KEPT: u64 = 3840 * 2160;

const VERTEX: &str = "#version 300 es
layout(location = 0) in vec4 q;
layout(location = 1) in vec4 m;
layout(location = 2) in vec4 n;
layout(location = 3) in vec4 c;
layout(location = 4) in vec4 i;
layout(location = 5) in vec4 s;
uniform vec2 size;
flat out vec4 vq;
flat out vec4 vm;
flat out vec4 vn;
flat out vec4 vc;
flat out vec4 vi;
flat out vec4 vs;
void main() {
    int k = gl_VertexID;
    vec2 corner = vec2(k == 1 || k == 4 || k == 5 ? 1.0 : 0.0, k == 2 || k == 3 || k == 5 ? 1.0 : 0.0);
    gl_Position = vec4(mix(q.xy, q.zw, corner) / size * 2.0 - 1.0, 0.0, 1.0);
    vq = q; vm = m; vn = n; vc = c; vi = i; vs = s;
}
";

const FRAGMENT: &str = "#version 300 es
precision highp float;
precision highp int;
uniform sampler2D masks;
uniform sampler2D pictures;
uniform sampler2D tables;
flat in vec4 vq;
flat in vec4 vm;
flat in vec4 vn;
flat in vec4 vc;
flat in vec4 vi;
flat in vec4 vs;
out vec4 o;
int pick(int u, int mid, int extra) { return u < mid ? u : (u < mid + extra ? mid : u - extra); }
int byte(vec4 t) { return int(t.r * 255.0 + 0.5); }
int div255(int v) { v += 128; return (v + (v >> 8)) >> 8; }
// A see-through colour laid over d at `cover` the way tiny-skia's 8-bit blend rounds it, floor((d * (255 - c) + x + 255) / 256)
// for c the colour's alpha scaled by the cover and x 256 times its colour so scaled and rounded up: alpha (c + 1) / 256 makes
// the blend's d * (1 - alpha) exactly d * (255 - c) / 256, and (x + 127.5) / 256 its rounding that floor. An opaque colour's
// rounding is within a level of the blend's own, so only these pay for it.
void soft(int cover) {
    ivec4 x = ((ivec4(vc * 255.0 + 0.5) * cover + 255) >> 8) * 256;
    int c = x.a / 256;
    if (c == 0) discard;
    o = vec4((vec3(x.rgb) + 127.5) / 65280.0, float(c + 1) / 256.0);
}
void main() {
    ivec2 at = ivec2(gl_FragCoord.xy);
    int kind = int(vn.w);
    if (kind == 0) {
        vec2 p = vec2(at);
        vec2 a = clamp(min(p + 1.0, vs.zw) - max(p, vs.xy), 0.0, 1.0);
        if (vc.a < 1.0) soft(int(a.x * a.y * 255.0 + 0.5)); else o = vc * (a.x * a.y);
        return;
    }
    ivec2 u = at - ivec2(vq.xy);
    ivec2 t = ivec2(pick(u.x, int(vm.z), int(vn.x)), pick(u.y, int(vm.w), int(vn.y)));
    if (kind == 3) {
        if (u.x >= int(vs.x) && u.x < int(vs.z) && u.y >= int(vs.y) && u.y < int(vs.w)) discard;
        int v = byte(texelFetch(masks, ivec2(vm.xy) + t / int(vn.z), 0));
        ivec4 k = ivec4(vc);
        int a = v * k.a / 255;
        o = vec4(vec3(k.rgb * a / 255), float(a)) / 255.0;
        return;
    }
    float cover = 1.0;
    if (vm.x >= 0.0) {
        int v = byte(texelFetch(masks, ivec2(vm.xy) + t, 0));
        if (vi.z >= 0.0) {
            int a = div255(byte(texelFetch(tables, ivec2(v, int(vi.z)), 0)) * int(vc.a));
            o = vec4(vc.rgb * float(a) / 65025.0, float(a) / 255.0);
            return;
        }
        if (kind == 1 && vc.a < 1.0) {
            soft(v);
            return;
        }
        cover = float(v) / 255.0;
    }
    o = kind == 2 ? texelFetch(pictures, ivec2(vi.xy) + u, 0) * (vc.a * cover) : vc * cover;
}
";

pub(crate) struct Target {
    tex: u32,
    fbo: u32,
    pub(crate) w: u32,
    pub(crate) h: u32,
}

#[derive(Clone, Copy)]
struct Spot {
    u: u32,
    v: u32,
    w: u32,
    h: u32,
    tex: u32,
}

struct Atlas {
    tex: u32,
    texel: Texel,
    shelves: Vec<[u32; 3]>,
    top: u32,
    side: u32,
    max: u32,
}

impl Atlas {
    fn take(&mut self, w: u32, h: u32) -> Option<(u32, u32)> {
        if let Some(s) = self.shelves.iter_mut().find(|s| h <= s[1] && s[1] <= h + h / 2 + 2 && s[2] + w <= self.side) {
            s[2] += w;
            return Some((s[2] - w, s[0]));
        }
        (self.top + h <= self.side && w <= self.side).then(|| {
            self.shelves.push([self.top, h, w]);
            self.top += h;
            (0, self.top - h)
        })
    }
}

struct Level {
    fbo: u32,
    size: (u32, u32),
    off: (i32, i32),
    clip: [i32; 4],
    room: (u32, u32),
    layer: Option<(Target, IntRect, f32, f32)>,
}

pub(crate) struct Store {
    masks: Atlas,
    pictures: Atlas,
    glyphs: HashMap<MaskKey, (Rc<Mask>, Spot)>,
    shapes: HashMap<[u32; 12], Spot>,
    paths: HashMap<u64, Spot>,
    shades: HashMap<*const Made, (Rc<Made>, Spot)>,
    images: HashMap<u64, Spot>,
    held: Vec<u32>,
    shared_bytes: usize,
    levels: Vec<Level>,
}

struct Letters {
    atlas: Atlas,
    known: HashMap<[u64; 2], Spot>,
    keys: [RandomState; 2],
}

pub(crate) struct Gpu {
    gl: Context,
    size_at: i32,
    quads: Instances,
    batch: Vec<f32>,
    bound: (u32, u32),
    masks: Atlas,
    pictures: Atlas,
    tables: u32,
    rows: Vec<[u8; 256]>,
    glyphs: HashMap<MaskKey, (Rc<Mask>, Spot)>,
    shapes: HashMap<[u32; 12], Spot>,
    paths: HashMap<u64, Spot>,
    shades: HashMap<*const Made, (Rc<Made>, Spot)>,
    images: HashMap<u64, Spot>,
    held: Vec<u32>,
    shared_bytes: usize,
    letters: Option<Letters>,
    strict: bool,
    area: f64,
    lost: bool,
    pool: Vec<Target>,
    loose: Vec<u32>,
    loose_bytes: usize,
    scratch: Option<Pixmap>,
    island: Option<Spot>,
    levels: Vec<Level>,
    until_settle: u8,
    #[cfg(test)]
    pub(crate) resets: [u32; 2],
    #[cfg(test)]
    pub(crate) growths: [u32; 2],
}

impl Gpu {
    pub(crate) fn new() -> Option<Gpu> {
        Gpu::with(Context::new()?)
    }

    pub(crate) fn on(device: Rc<Device>) -> Option<Gpu> {
        Gpu::with(Context::on_gbm(device)?)
    }

    fn with(gl: Context) -> Option<Gpu> {
        let program = gl.program(VERTEX, FRAGMENT).ok()?;
        gl.use_program(program);
        for (n, name) in ["masks", "pictures", "tables"].iter().enumerate() {
            gl.uniform_int(gl.uniform_location(program, name), n as i32);
        }
        let atlas = |texel| Atlas { tex: gl.texture(texel, START, START), texel, shelves: Vec::new(), top: 0, side: START, max: SIDE };
        let (masks, pictures) = (atlas(Texel::R8), atlas(Texel::Rgba8));
        let tables = gl.texture(Texel::R8, 256, ROWS as u32);
        Some(Gpu {
            size_at: gl.uniform_location(program, "size"), quads: gl.instances(GROUPS), batch: Vec::new(),
            bound: (masks.tex, pictures.tex), masks, pictures, tables, rows: Vec::new(),
            glyphs: HashMap::new(), shapes: HashMap::new(), paths: HashMap::new(), shades: HashMap::new(),
            images: HashMap::new(), held: Vec::new(), shared_bytes: 0, letters: None, strict: false, area: 0.0, lost: false,
            pool: Vec::new(), loose: Vec::new(), loose_bytes: 0, scratch: None, island: None,
            levels: Vec::new(), until_settle: SETTLE_AFTER, gl,
            #[cfg(test)]
            resets: [0; 2],
            #[cfg(test)]
            growths: [0; 2],
        })
    }

    pub(crate) fn renderer(&self) -> String {
        self.gl.string(gl::GL_RENDERER)
    }

    pub(crate) fn target(&self, w: u32, h: u32) -> Option<Target> {
        let (tex, fbo) = (self.gl.texture(Texel::Rgba8, w, h), self.gl.framebuffer());
        self.gl.bind_framebuffer(gl::GL_FRAMEBUFFER, fbo);
        self.gl.framebuffer_texture(gl::GL_FRAMEBUFFER, gl::GL_COLOR_ATTACHMENT0, gl::GL_TEXTURE_2D, tex, 0);
        if self.gl.framebuffer_status(gl::GL_FRAMEBUFFER) != gl::GL_FRAMEBUFFER_COMPLETE {
            self.gl.delete_framebuffer(fbo);
            self.gl.delete_texture(tex);
            return None;
        }
        self.gl.disable(gl::GL_SCISSOR_TEST);
        self.gl.clear_colour(0.0, 0.0, 0.0, 0.0);
        self.gl.clear(gl::GL_COLOR_BUFFER_BIT);
        self.gl.enable(gl::GL_SCISSOR_TEST);
        Some(Target { tex, fbo, w, h })
    }

    pub(crate) fn target_of(&self, d: &Dmabuf, modifier: Option<u64>) -> Option<Target> {
        let tex = self.gl.import(d, modifier)?;
        let fbo = self.gl.framebuffer();
        self.gl.bind_framebuffer(gl::GL_FRAMEBUFFER, fbo);
        self.gl.framebuffer_texture(gl::GL_FRAMEBUFFER, gl::GL_COLOR_ATTACHMENT0, gl::GL_TEXTURE_2D, tex, 0);
        let t = Target { tex, fbo, w: d.width, h: d.height };
        if self.gl.framebuffer_status(gl::GL_FRAMEBUFFER) != gl::GL_FRAMEBUFFER_COMPLETE {
            self.free(t);
            return None;
        }
        Some(t)
    }

    pub(crate) fn free(&self, t: Target) {
        self.gl.delete_framebuffer(t.fbo);
        self.gl.delete_texture(t.tex);
    }

    pub(crate) fn current(&self) -> bool {
        self.gl.make_current()
    }

    pub(crate) fn fence(&self) -> Option<OwnedFd> {
        self.gl.fence()
    }

    pub(crate) fn put(&self, t: &Target, pix: &Pixmap) {
        self.gl.upload(t.tex, Texel::Rgba8, (0, 0, t.w, t.h), pix.data(), t.w as usize);
    }

    pub(crate) fn read(&self, t: &Target) -> Option<Pixmap> {
        self.gl.bind_framebuffer(gl::GL_FRAMEBUFFER, t.fbo);
        let px = self.gl.read_rgba(0, 0, t.w.try_into().ok()?, t.h.try_into().ok()?);
        Pixmap::from_vec(px, tiny_skia::IntSize::from_wh(t.w, t.h)?)
    }

    pub(crate) fn finish(&self) {
        self.gl.finish();
    }

    pub(crate) fn error(&self) -> u32 {
        self.gl.error()
    }

    pub(crate) fn play(&mut self, ops: Vec<Op>, target: &Target, shadows: &Shadows) {
        self.play_in(ops, target, shadows, None);
    }

    pub(crate) fn play_part(&mut self, ops: Vec<Op>, target: &Target, shadows: &Shadows, part: [i32; 4]) {
        self.play_in(ops, target, shadows, Some(part));
    }

    fn play_in(&mut self, ops: Vec<Op>, target: &Target, shadows: &Shadows, part: Option<[i32; 4]>) {
        self.begin(target, part);
        self.run(&mut ops.into_iter(), shadows, None);
        self.end();
    }

    pub(crate) fn begin(&mut self, target: &Target, part: Option<[i32; 4]>) {
        let size = (target.w, target.h);
        let whole = [0, 0, size.0 as i32, size.1 as i32];
        let clip = part.map_or(whole, |p| [p[0].max(0), p[1].max(0), p[2].min(whole[2]), p[3].min(whole[3])]);
        self.levels = vec![Level { fbo: target.fbo, size, off: (0, 0), clip, room: size, layer: None }];
        self.resume();
        if part.is_some() {
            self.gl.clear_colour(0.0, 0.0, 0.0, 0.0);
            self.gl.clear(gl::GL_COLOR_BUFFER_BIT);
        }
    }

    pub(crate) fn resume(&mut self) {
        self.gl.enable(gl::GL_BLEND);
        self.gl.blend_func(gl::GL_ONE, gl::GL_ONE_MINUS_SRC_ALPHA);
        self.gl.enable(gl::GL_SCISSOR_TEST);
        self.bound = (self.masks.tex, self.pictures.tex);
        self.bind_all();
        self.apply();
    }

    pub(crate) fn run<'a>(&mut self, ops: &mut impl Iterator<Item = Op<'a>>, shadows: &Shadows, until: Option<Instant>) -> bool {
        self.area = 0.0;
        for op in ops.by_ref() {
            match op {
                Op::Push { at, opacity, radius } => self.push(at, opacity, radius),
                Op::Pop => self.pop(),
                op => self.draw(op, shadows),
            }
            if self.lost {
                return true;
            }
            if until.is_some_and(|t| self.area >= AREA_MOST || Instant::now() >= t) {
                self.flush();
                return false;
            }
        }
        true
    }

    pub(crate) fn end(&mut self) {
        while self.levels.len() > 1 {
            self.pop();
        }
        self.flush();
        if self.until_settle > 0 {
            self.until_settle -= 1;
            if self.until_settle == 0 {
                self.gl.finish();
                crate::sys::lean::settle();
            }
        }
        self.free_loose();
        self.trim_pool();
        self.trim_island();
    }

    fn trim_island(&mut self) {
        if self.island.is_some_and(|s| u64::from(s.w) * u64::from(s.h) > ISLAND_KEPT) {
            if let Some(s) = self.island.take() {
                self.gl.delete_texture(s.tex);
                self.bound.1 = u32::MAX;
            }
        }
    }

    pub(crate) fn abandon(&mut self) -> bool {
        self.batch.clear();
        for l in std::mem::take(&mut self.levels) {
            if let Some((t, ..)) = l.layer {
                self.pool.push(t);
            }
        }
        self.free_loose();
        self.trim_pool();
        std::mem::take(&mut self.lost)
    }

    pub(crate) fn lost(&self) -> bool {
        self.lost
    }

    fn free_loose(&mut self) {
        self.loose_bytes = 0;
        for tex in std::mem::take(&mut self.loose) {
            self.gl.delete_texture(tex);
        }
    }

    fn trim_pool(&mut self) {
        let bytes = |p: &[Target]| p.iter().map(|t| u64::from(t.w) * u64::from(t.h) * 4).sum::<u64>();
        while self.pool.len() > POOL_MOST || (self.pool.len() > 1 && bytes(&self.pool) > POOL_MOST_BYTES) {
            let t = self.pool.remove(0);
            self.free(t);
        }
    }

    fn bind_all(&mut self) {
        for (unit, tex) in [(0, self.bound.0), (1, self.bound.1), (2, self.tables)] {
            self.gl.active_texture(gl::GL_TEXTURE0 + unit);
            self.gl.bind_texture(gl::GL_TEXTURE_2D, tex);
        }
    }

    fn bind(&mut self, mask: u32, picture: u32) {
        if self.bound != (mask, picture) {
            self.flush();
            self.bound = (mask, picture);
            self.bind_all();
        }
    }

    fn flush(&mut self) {
        if !self.batch.is_empty() {
            self.gl.draw_instances(&self.quads, &self.batch);
            self.batch.clear();
        }
    }

    fn apply(&self) {
        let Some(l) = self.levels.last() else { return };
        self.gl.bind_framebuffer(gl::GL_FRAMEBUFFER, l.fbo);
        self.gl.viewport(0, 0, l.size.0 as i32, l.size.1 as i32);
        self.gl.uniform_2f(self.size_at, l.size.0 as f32, l.size.1 as f32);
        let c = l.clip;
        self.gl.scissor(c[0], c[1], (c[2] - c[0]).max(0), (c[3] - c[1]).max(0));
    }

    fn quad(&mut self, groups: [[f32; 4]; GROUPS]) {
        self.batch.extend(groups.iter().flatten());
        if self.strict {
            if let Some(l) = self.levels.last() {
                let ([x0, y0, x1, y1], c) = (groups[0], l.clip.map(|v| v as f32));
                self.area += (f64::from(x1.min(c[2]) - x0.max(c[0])).max(0.0)) * (f64::from(y1.min(c[3]) - y0.max(c[1])).max(0.0));
            }
        }
        if self.batch.len() >= BATCH_MOST {
            self.flush();
        }
    }

    fn push(&mut self, at: IntRect, opacity: f32, radius: f32) {
        self.flush();
        let Some(top) = self.levels.last() else { return };
        let room = (at.width(), at.height());
        let level = if opacity >= 1.0 && radius <= 0.0 {
            let off = (top.off.0 + at.x(), top.off.1 + at.y());
            let c = top.clip;
            let clip = [c[0].max(off.0), c[1].max(off.1), c[2].min(off.0 + room.0 as i32), c[3].min(off.1 + room.1 as i32)];
            Level { fbo: top.fbo, size: top.size, off, clip, room, layer: None }
        } else {
            let found = self.pool.iter().position(|t| (t.w, t.h) == room);
            let t = match found {
                Some(n) => Some(self.pool.swap_remove(n)),
                None => self.target(room.0, room.1),
            };
            match t {
                Some(t) => Level { fbo: t.fbo, size: room, off: (0, 0), clip: [0, 0, room.0 as i32, room.1 as i32], room, layer: Some((t, at, opacity, radius)) },
                None => Level { fbo: top.fbo, size: top.size, off: top.off, clip: [0; 4], room, layer: None },
            }
        };
        let fresh = level.layer.is_some();
        self.levels.push(level);
        self.apply();
        if fresh {
            self.gl.clear_colour(0.0, 0.0, 0.0, 0.0);
            self.gl.clear(gl::GL_COLOR_BUFFER_BIT);
            self.area += f64::from(room.0) * f64::from(room.1);
        }
    }

    fn pop(&mut self) {
        if self.levels.len() < 2 {
            return;
        }
        self.flush();
        let Some(Level { layer, .. }) = self.levels.pop() else { return };
        self.apply();
        let Some((t, at, opacity, radius)) = layer else { return };
        let off = self.levels.last().map_or((0, 0), |l| l.off);
        let (x, y) = (off.0 + at.x(), off.1 + at.y());
        let q = [x as f32, y as f32, (x + t.w as i32) as f32, (y + t.h as i32) as f32];
        let shape = Rect { x: at.x() as f32, y: at.y() as f32, w: t.w as f32, h: t.h as f32 };
        let room = self.levels.last().map_or((0, 0), |l| l.room);
        let mask = if radius > 0.0 { self.shape(shape, [radius; 4], off, room) } else { None };
        let (q, from, m, n) = match mask {
            Some((s, p, mid, stretch)) => (
                [p.0, p.1, p.0 + (s.w + stretch.0) as f32, p.1 + (s.h + stretch.1) as f32],
                [p.0 - x as f32, p.1 - y as f32, -1.0, 0.0],
                [s.u as f32, s.v as f32, mid.0 as f32, mid.1 as f32],
                [stretch.0 as f32, stretch.1 as f32, 1.0, PICTURE],
            ),
            None => (q, [0.0, 0.0, -1.0, 0.0], NO_MASK, [0.0, 0.0, 1.0, PICTURE]),
        };
        let mask_tex = mask.map_or(self.bound.0, |s| s.0.tex);
        self.bind(mask_tex, t.tex);
        self.quad([q, m, n, [0.0, 0.0, 0.0, opacity], from, NONE]);
        self.bind(self.bound.0, self.pictures.tex);
        self.pool.push(t);
        self.trim_pool();
    }

    fn draw(&mut self, op: Op, shadows: &Shadows) {
        let Some(l) = self.levels.last() else { return };
        let (off, room) = (l.off, l.room);
        if l.clip[0] >= l.clip[2] || l.clip[1] >= l.clip[3] {
            if let Op::Pixels { at, draw } = op {
                self.island(at, draw, off, room, false);
            }
            return;
        }
        match op {
            Op::Fill { r, colour } => {
                let r = Rect { x: r.x + off.0 as f32, y: r.y + off.1 as f32, ..r };
                if tiny_skia::Rect::from_xywh(r.x, r.y, r.w, r.h).is_none() {
                    return;
                }
                let q = [r.x.floor(), r.y.floor(), (r.x + r.w).ceil(), (r.y + r.h).ceil()];
                self.quad([q, NONE, [0.0, 0.0, 1.0, FILL], premultiplied(colour), NONE, [r.x, r.y, r.x + r.w, r.y + r.h]]);
            }
            Op::Round { r, radii, colour } => {
                let Some((s, at, mid, stretch)) = self.shape(r, radii, off, room) else { return };
                self.bind(s.tex, self.bound.1);
                let q = [at.0, at.1, at.0 + (s.w + stretch.0) as f32, at.1 + (s.h + stretch.1) as f32];
                self.quad([q, [s.u as f32, s.v as f32, mid.0 as f32, mid.1 as f32], [stretch.0 as f32, stretch.1 as f32, 1.0, COVER], premultiplied(colour), [0.0, 0.0, -1.0, 0.0], NONE]);
            }
            Op::Line { from, to, width, colour } => {
                let mut b = PathBuilder::new();
                b.move_to(from.0, from.1);
                b.line_to(to.0, to.1);
                if let Some(path) = b.finish() {
                    self.path(&path, &How::Stroke(Stroke { width, ..Stroke::default() }), colour, off, room);
                }
            }
            Op::Path { path, how, colour } => self.path(&path, &how, colour, off, room),
            Op::Glyphs { run, colour, table } => {
                let row = self.row(&table) as f32;
                let ink = colour.to_skia().to_color_u8();
                let c = [ink.red() as f32, ink.green() as f32, ink.blue() as f32, (colour.a.clamp(0.0, 1.0) * 255.0).round()];
                for g in run {
                    let (left, top, ..) = g.mask.parts();
                    let Some(s) = self.glyph(g.key, &g.mask) else { continue };
                    self.bind(s.tex, self.bound.1);
                    let (x, y) = ((g.x + left + off.0) as f32, (g.y + top + off.1) as f32);
                    self.quad([[x, y, x + s.w as f32, y + s.h as f32], [s.u as f32, s.v as f32, s.w as f32, s.h as f32], [0.0, 0.0, 1.0, COVER], c, [0.0, 0.0, row, 0.0], NONE]);
                }
            }
            Op::Image { id, pixmap, x, y } => {
                let Some(s) = self.image(id, &pixmap) else { return };
                self.bind(self.bound.0, s.tex);
                let (x, y) = ((x + off.0) as f32, (y + off.1) as f32);
                self.quad([[x, y, x + s.w as f32, y + s.h as f32], NO_MASK, [0.0, 0.0, 1.0, PICTURE], [0.0, 0.0, 0.0, 1.0], [s.u as f32, s.v as f32, -1.0, 0.0], NONE]);
            }
            Op::Shadow { r, radius, falls, colour } => self.shadow(shadows.plan(false, r, radius, &falls), colour, off),
            Op::Inside { r, radius, fall, colour } => self.shadow(shadows.plan(true, r, radius, &[fall]), colour, off),
            Op::Pixels { at, draw } => self.island(at, draw, off, room, true),
            Op::Laid { x, y, pixmap } => self.lay(pixmap.data(), pixmap.width() as usize, (x + off.0, y + off.1), (pixmap.width(), pixmap.height())),
            Op::Push { .. } | Op::Pop => {}
        }
    }

    fn place(&mut self, texel: Texel, w: u32, h: u32, data: &[u8], stride: usize) -> Option<Spot> {
        if w == 0 || h == 0 {
            return None;
        }
        let mut at = self.atlas(texel).take(w, h);
        while at.is_none() && self.atlas(texel).side < self.atlas(texel).max && self.grow(texel) {
            at = self.atlas(texel).take(w, h);
        }
        if at.is_none() && w <= self.atlas(texel).side && h <= self.atlas(texel).side {
            #[cfg(test)]
            {
                self.resets[usize::from(texel != Texel::R8)] += 1;
            }
            if texel == Texel::R8 {
                log::say(log::Kind::Masks, || format!("letter and shape atlas full, {} letters and {} shapes dropped", self.glyphs.len(), self.shapes.len() + self.paths.len() + self.shades.len()));
            } else {
                log::say(log::Kind::Pictures, || format!("picture atlas full, {} pictures dropped", self.images.len()));
            }
            self.flush();
            self.empty(texel);
            at = self.atlas(texel).take(w, h);
        }
        let (tex, (u, v)) = match at {
            Some(at) => (self.atlas(texel).tex, at),
            None => {
                let bytes = w as usize * h as usize * if texel == Texel::R8 { 1 } else { 4 };
                if self.loose_bytes + bytes > LOOSE_MOST && !self.loose.is_empty() {
                    self.flush();
                    self.free_loose();
                    self.bound = (self.masks.tex, self.pictures.tex);
                    self.bind_all();
                }
                self.loose_bytes += bytes;
                let tex = self.gl.texture(texel, w, h);
                self.loose.push(tex);
                (tex, (0, 0))
            }
        };
        self.gl.upload(tex, texel, (u, v, w, h), data, stride);
        Some(Spot { u, v, w, h, tex })
    }

    fn atlas(&mut self, texel: Texel) -> &mut Atlas {
        if texel == Texel::R8 { &mut self.masks } else { &mut self.pictures }
    }

    fn grow(&mut self, texel: Texel) -> bool {
        self.flush();
        let (old, side, used, max) = { let a = self.atlas(texel); (a.tex, a.side, a.top, a.max) };
        let side_now = (side * 2).min(max);
        let tex = self.gl.texture(texel, side_now, side_now);
        if used > 0 && !self.gl.copy_texture(old, tex, (side, used)) {
            self.gl.delete_texture(tex);
            self.atlas(texel).max = side;
            self.apply();
            return false;
        }
        self.gl.delete_texture(old);
        let a = self.atlas(texel);
        (a.tex, a.side) = (tex, side_now);
        #[cfg(test)]
        {
            self.growths[usize::from(texel != Texel::R8)] += 1;
        }
        let spots: Vec<&mut Spot> = if texel == Texel::R8 {
            self.glyphs.values_mut().map(|v| &mut v.1).chain(self.shapes.values_mut()).chain(self.paths.values_mut()).chain(self.shades.values_mut().map(|v| &mut v.1)).collect()
        } else {
            self.images.values_mut().collect()
        };
        for s in spots.into_iter().filter(|s| s.tex == old) {
            s.tex = tex;
        }
        if texel == Texel::R8 && self.bound.0 == old {
            self.bound.0 = tex;
        } else if texel != Texel::R8 && self.bound.1 == old {
            self.bound.1 = tex;
        }
        self.bind_all();
        self.apply();
        log::say(if texel == Texel::R8 { log::Kind::Masks } else { log::Kind::Pictures }, || format!("{} atlas grown to {side_now} x {side_now}", if texel == Texel::R8 { "letter and shape" } else { "picture" }));
        true
    }

    fn empty(&mut self, texel: Texel) {
        if texel == Texel::R8 {
            self.masks.shelves.clear();
            self.masks.top = 0;
            let tex = self.masks.tex;
            self.glyphs.retain(|_, g| g.1.tex != tex);
            self.shapes.clear();
            self.paths.clear();
            self.shades.clear();
        } else {
            self.pictures.shelves.clear();
            self.pictures.top = 0;
            self.images.clear();
        }
    }

    fn kept(&self, s: Spot) -> bool {
        s.tex == self.masks.tex || s.tex == self.pictures.tex
    }

    fn glyph(&mut self, key: MaskKey, mask: &Rc<Mask>) -> Option<Spot> {
        if self.strict {
            let found = self.glyphs.get(&key).map(|g| g.1);
            let (_, _, w, h, _) = mask.parts();
            self.lost |= found.is_none() && w > 0 && h > 0;
            return found;
        }
        if let Some((kept, s)) = self.glyphs.get(&key) {
            if Rc::ptr_eq(kept, mask) || kept.parts() == mask.parts() {
                return Some(*s);
            }
        }
        let (_, _, w, h, cover) = mask.parts();
        let s = self.place(Texel::R8, w as u32, h as u32, cover, w)?;
        if self.kept(s) {
            self.glyphs.insert(key, (mask.clone(), s));
        }
        Some(s)
    }

    fn image(&mut self, id: u64, pix: &Pixmap) -> Option<Spot> {
        if let Some(s) = self.images.get(&id) {
            return Some(*s);
        }
        if self.strict {
            self.lost = true;
            return None;
        }
        let s = self.place(Texel::Rgba8, pix.width(), pix.height(), pix.data(), pix.width() as usize)?;
        if self.kept(s) {
            self.images.insert(id, s);
        }
        Some(s)
    }

    fn row(&mut self, table: &[u8; 256]) -> usize {
        if let Some(n) = self.rows.iter().position(|t| t == table) {
            return n;
        }
        if self.rows.len() == ROWS {
            self.flush();
            self.rows.clear();
        }
        self.rows.push(*table);
        let n = self.rows.len() - 1;
        self.gl.upload(self.tables, Texel::R8, (0, n as u32, 256, 1), table, 256);
        n
    }

    fn shape(&mut self, r: Rect, radii: [f32; 4], off: (i32, i32), room: (u32, u32)) -> Option<(Spot, (f32, f32), (u32, u32), (u32, u32))> {
        let at = (r.x.floor(), r.y.floor());
        let placed = (at.0 + off.0 as f32, at.1 + off.1 as f32);
        let full = ((r.x + r.w).ceil() - at.0, (r.y + r.h).ceil() - at.1);
        let corner = radii.iter().fold(0.0f32, |a, &b| a.max(b)).min(r.w / 2.0).min(r.h / 2.0).max(0.0).ceil() as u32 + 2;
        let (fw, fh) = (full.0 as u32, full.1 as u32);
        let cut = at.0 < 0.0 || at.1 < 0.0 || at.0 + fw as f32 > room.0 as f32 || at.1 + fh as f32 > room.1 as f32;
        let stretch = if cut { (0, 0) } else { (fw.saturating_sub(2 * corner), fh.saturating_sub(2 * corner)) };
        let mid = (corner - 1, corner - 1);
        let seen = |from: f32, len: u32, edge: u32| ((-from).max(0.0) as u32, (edge as f32 - from).clamp(0.0, len as f32) as u32);
        let (across, down) = (seen(at.0, fw, room.0), seen(at.1, fh, room.1));
        if cut && (across.0 >= across.1 || down.0 >= down.1) {
            return None;
        }
        let [a, b, c, d, e, f, g, h] = [r.x, r.y, r.w, r.h, radii[0], radii[1], radii[2], radii[3]].map(f32::to_bits);
        let key = if cut { [a, b, c, d, e, f, g, h, across.0, across.1, down.0, down.1] } else { [a, b, c, d, e, f, g, h, 0, 0, 0, 0] };
        let placed = if cut { (placed.0 + across.0 as f32, placed.1 + down.0 as f32) } else { placed };
        if let Some(s) = self.shapes.get(&key) {
            return Some((*s, placed, mid, stretch));
        }
        let path = super::cpu::corners(r, radii)?;
        let (mw, mh) = if cut { (across.1 - across.0, down.1 - down.0) } else { (fw - stretch.0, fh - stretch.1) };
        let mut cover = vec![0u8; mw as usize * mh as usize];
        let white = paint(Colour { r: 1.0, g: 1.0, b: 1.0, a: 1.0 });
        if cut {
            let mut pix = Pixmap::new(room.0, room.1)?;
            pix.fill_path(&path, &white, FillRule::Winding, Transform::identity(), None);
            for y in down.0..down.1 {
                let from = ((at.1 as i64 + y as i64) * room.0 as i64 + at.0 as i64 + across.0 as i64) as usize * 4;
                let to = (y - down.0) as usize * mw as usize;
                for (n, px) in pix.data()[from..from + mw as usize * 4].chunks_exact(4).enumerate() {
                    cover[to + n] = px[3];
                }
            }
        }
        let split = |m: u32, s: u32| if s == 0 { [(0, m, 0), (m, m, m)] } else { [(0, corner, 0), (corner, m, corner + s)] };
        for (c0, c1, from_x) in split(mw, stretch.0).into_iter().filter(|_| !cut) {
            for (r0, r1, from_y) in split(mh, stretch.1) {
                let Some(mut pix) = Pixmap::new(c1 - c0, r1 - r0) else { continue };
                let move_by = Transform::from_translate(-(at.0 + from_x as f32), -(at.1 + from_y as f32));
                pix.fill_path(&path, &white, FillRule::Winding, move_by, None);
                for (y, line) in pix.data().chunks_exact(4 * (c1 - c0) as usize).enumerate() {
                    let start = (r0 as usize + y) * mw as usize + c0 as usize;
                    for (to, px) in cover[start..start + (c1 - c0) as usize].iter_mut().zip(line.chunks_exact(4)) {
                        *to = px[3];
                    }
                }
            }
        }
        let s = self.place(Texel::R8, mw, mh, &cover, mw as usize)?;
        if self.kept(s) {
            self.shapes.insert(key, s);
        }
        Some((s, placed, mid, stretch))
    }

    fn cover_of(&mut self, pix: &Pixmap) -> Option<Spot> {
        let cover: Vec<u8> = pix.data().chunks_exact(4).map(|p| p[3]).collect();
        self.place(Texel::R8, pix.width(), pix.height(), &cover, pix.width() as usize)
    }

    fn path(&mut self, path: &Path, how: &How, colour: Colour, off: (i32, i32), room: (u32, u32)) {
        let b = path.bounds();
        let margin = match how {
            How::Fill(_) => 2.0,
            How::Stroke(s) => s.width.max(1.0) * s.miter_limit.max(1.0) + 2.0,
        };
        let x0 = (b.left() - margin).floor().max(0.0);
        let y0 = (b.top() - margin).floor().max(0.0);
        let x1 = (b.right() + margin).ceil().min(room.0 as f32);
        let y1 = (b.bottom() + margin).ceil().min(room.1 as f32);
        if x1 <= x0 || y1 <= y0 {
            return;
        }
        let mut h = DefaultHasher::new();
        for p in path.points() {
            (p.x.to_bits(), p.y.to_bits()).hash(&mut h);
        }
        for v in path.verbs() {
            std::mem::discriminant(v).hash(&mut h);
        }
        match how {
            How::Fill(rule) => (0u8, *rule as u8).hash(&mut h),
            How::Stroke(s) => (1u8, s.width.to_bits(), s.miter_limit.to_bits(), s.line_cap as u8, s.line_join as u8, format!("{:?}", s.dash)).hash(&mut h),
        }
        [x0, y0, x1, y1].map(f32::to_bits).hash(&mut h);
        let key = h.finish();
        let s = match self.paths.get(&key) {
            Some(s) => *s,
            None => {
                let Some(mut pix) = Pixmap::new((x1 - x0) as u32, (y1 - y0) as u32) else { return };
                let (white, move_by) = (paint(Colour { r: 1.0, g: 1.0, b: 1.0, a: 1.0 }), Transform::from_translate(-x0, -y0));
                match how {
                    How::Fill(rule) => pix.fill_path(path, &white, *rule, move_by, None),
                    How::Stroke(stroke) => pix.stroke_path(path, &white, stroke, move_by, None),
                }
                let Some(s) = self.cover_of(&pix) else { return };
                if self.kept(s) {
                    self.paths.insert(key, s);
                }
                s
            }
        };
        self.bind(s.tex, self.bound.1);
        let (x, y) = (x0 + off.0 as f32, y0 + off.1 as f32);
        self.quad([[x, y, x + s.w as f32, y + s.h as f32], [s.u as f32, s.v as f32, s.w as f32, s.h as f32], [0.0, 0.0, 1.0, COVER], premultiplied(colour), [0.0, 0.0, -1.0, 0.0], NONE]);
    }

    fn shadow(&mut self, plan: Option<(Rc<Made>, (i32, i32), (usize, usize), Option<usize>)>, colour: Colour, off: (i32, i32)) {
        let Some((made, at, size, hollow)) = plan else { return };
        let (w, h, inner, boxed, step, cover) = made.parts();
        let key = Rc::as_ptr(&made);
        let s = match self.shades.get(&key) {
            Some((_, s)) => *s,
            None => {
                let across = w.div_ceil(step);
                let Some(s) = self.place(Texel::R8, across as u32, (cover.len() / across) as u32, cover, across) else { return };
                if self.kept(s) {
                    self.shades.insert(key, (made.clone(), s));
                }
                s
            }
        };
        let extra = (size.0.saturating_sub(boxed.0), size.1.saturating_sub(boxed.1));
        let mid = (inner.0 + boxed.0 / 2, inner.1 + boxed.1 / 2);
        let (x, y) = ((at.0 - inner.0 as i32 + off.0) as f32, (at.1 - inner.1 as i32 + off.1) as f32);
        let hole = hollow.map_or(NONE, |r| [inner.0 as f32, (inner.1 + r) as f32, (inner.0 + size.0) as f32, (inner.1 + size.1).saturating_sub(r) as f32]);
        let c = colour.to_skia().premultiply().to_color_u8();
        self.bind(s.tex, self.bound.1);
        self.quad([
            [x, y, x + (w + extra.0) as f32, y + (h + extra.1) as f32],
            [s.u as f32, s.v as f32, mid.0 as f32, mid.1 as f32],
            [extra.0 as f32, extra.1 as f32, step as f32, SHADOW],
            [c.red() as f32, c.green() as f32, c.blue() as f32, c.alpha() as f32],
            NONE,
            hole,
        ]);
    }

    fn island(&mut self, at: IntRect, draw: Box<dyn FnOnce(&mut tiny_skia::PixmapMut) + '_>, off: (i32, i32), room: (u32, u32), show: bool) {
        self.flush();
        if self.scratch.as_ref().is_none_or(|p| (p.width(), p.height()) != room) {
            self.scratch = Pixmap::new(room.0, room.1);
        }
        let Some(pix) = self.scratch.as_mut() else { return };
        let x0 = at.x().clamp(0, room.0 as i32) as usize;
        let x1 = (at.x() + at.width() as i32).clamp(0, room.0 as i32) as usize;
        let y0 = at.y().clamp(0, room.1 as i32) as usize;
        let y1 = (at.y() + at.height() as i32).clamp(0, room.1 as i32) as usize;
        let stride = room.0 as usize * 4;
        for row in pix.data_mut().chunks_exact_mut(stride).take(y1).skip(y0) {
            row[x0 * 4..x1 * 4].fill(0);
        }
        draw(&mut pix.as_mut());
        let (w, h) = ((x1 - x0) as u32, (y1 - y0) as u32);
        if !show || w == 0 || h == 0 {
            return;
        }
        let Some(pix) = self.scratch.take() else { return };
        self.lay(&pix.data()[(y0 * room.0 as usize + x0) * 4..], room.0 as usize, (x0 as i32 + off.0, y0 as i32 + off.1), (w, h));
        self.scratch = Some(pix);
    }

    fn lay(&mut self, data: &[u8], stride: usize, (x, y): (i32, i32), (w, h): (u32, u32)) {
        let Some(l) = self.levels.last() else { return };
        let c = l.clip;
        if x >= c[2] || y >= c[3] || x + w as i32 <= c[0] || y + h as i32 <= c[1] || c[0] >= c[2] || c[1] >= c[3] {
            return;
        }
        let (x0, y0) = (x.max(c[0]), y.max(c[1]));
        let (w, h) = (((x + w as i32).min(c[2]) - x0) as u32, ((y + h as i32).min(c[3]) - y0) as u32);
        let data = &data[((y0 - y) as usize * stride + (x0 - x) as usize) * 4..];
        let (x, y) = (x0, y0);
        let fits = self.island.is_some_and(|s| s.w >= w && s.h >= h);
        if !fits {
            let old = self.island.take();
            if let Some(s) = old {
                self.gl.delete_texture(s.tex);
                self.bound.1 = u32::MAX;
            }
            let (w, h) = old.map_or((w, h), |s| {
                let round = (s.w.max(w), s.h.max(h));
                let area = |(w, h): (u32, u32)| u64::from(w) * u64::from(h);
                if area(round) <= 2 * (area((s.w, s.h)) + area((w, h))) { round } else { (w, h) }
            });
            self.island = Some(Spot { u: 0, v: 0, w, h, tex: self.gl.texture(Texel::Rgba8, w, h) });
        }
        let Some(island) = self.island else { return };
        self.gl.upload(island.tex, Texel::Rgba8, (0, 0, w, h), data, stride);
        self.bind(self.bound.0, island.tex);
        let (x, y) = (x as f32, y as f32);
        self.quad([[x, y, x + w as f32, y + h as f32], NO_MASK, [0.0, 0.0, 1.0, PICTURE], [0.0, 0.0, 0.0, 1.0], [0.0, 0.0, -1.0, 0.0], NONE]);
        self.flush();
    }
}

pub(crate) fn agrees(gpu: &mut Gpu) -> bool {
    let ink = Colour { r: 0.9, g: 0.4, b: 0.1, a: 0.8 };
    let record = |p: &mut super::Painter| {
        p.fill(Rect { x: 0.0, y: 0.0, w: 32.0, h: 32.0 }, Colour { r: 0.95, g: 0.93, b: 0.9, a: 1.0 });
        p.shadow(Rect { x: 6.0, y: 6.0, w: 18.0, h: 14.0 }, 4.0, &[crate::theme::Fall { y: 2.0, blur: 4.0, spread: 0.0, alpha: 0.4 }], Colour { r: 0.0, g: 0.0, b: 0.0, a: 1.0 });
        p.round(Rect { x: 6.5, y: 6.0, w: 18.0, h: 14.0 }, [5.0; 4], ink);
        p.line((2.0, 28.0), (30.0, 22.0), 1.5, ink);
        if let Some(at) = IntRect::from_xywh(8, 10, 16, 16) {
            p.push_layer(at, 0.5);
            p.fill(Rect { x: 2.0, y: 2.0, w: 9.0, h: 9.0 }, Colour { r: 0.1, g: 0.3, b: 0.8, a: 1.0 });
            p.pop();
            p.pixels(at, move |c| {
                if let Some(r) = tiny_skia::Rect::from_xywh(at.x() as f32, at.y() as f32, 16.0, 16.0) {
                    c.fill_rect(r, &paint(Colour { r: 0.1, g: 0.5, b: 0.2, a: 0.5 }), Transform::identity(), None);
                }
            });
        }
    };
    let shadows = Shadows::default();
    let (Some(mut want), Some(t)) = (Pixmap::new(32, 32), gpu.target(32, 32)) else { return false };
    let mut p = super::Painter::default();
    record(&mut p);
    super::cpu::play(p.ops, &mut want.as_mut(), &shadows);
    let mut p = super::Painter::default();
    record(&mut p);
    gpu.play(p.ops, &t, &shadows);
    let got = gpu.read(&t);
    gpu.free(t);
    gpu.error() == 0 && got.is_some_and(|got| got.data().iter().zip(want.data()).all(|(a, b)| a.abs_diff(*b) <= 2))
}

fn premultiplied(c: Colour) -> [f32; 4] {
    let p = c.to_skia().premultiply();
    [p.red(), p.green(), p.blue(), p.alpha()]
}

#[cfg(test)]
mod tests;
