//! Letters drawn sharp as GTK 4 draws them: hinted up and down, placed on whole pixels as a browser places them, exact coverage; fontconfig's hinting and smoothing choices are honoured.

use std::cell::{OnceCell, RefCell};
use std::collections::HashMap;
use std::io::Read;
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Arc, Condvar, Mutex, OnceLock};
use std::rc::Rc;
use std::time::{Duration, Instant};

use ab_glyph_rasterizer::{point, Point, Rasterizer};
use skrifa::instance::{Location, LocationRef, Size};
use skrifa::outline::{DrawSettings, Engine, GlyphStyles, HintingInstance, HintingOptions, OutlineGlyphFormat,
                      OutlinePen, SmoothMode, Target};
use skrifa::{FontRef, GlyphId, MetadataProvider};
use tiny_skia::PixmapMut;

use super::colour::Colour;

const GAMMA: f32 = 1.2;
const CONTRAST: f32 = 0.2;
const MOST_MASKS: usize = 4096;
const MOST_HINTERS: usize = 48;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct Render {
    pub(super) hint: bool,
    pub(super) smooth: bool,
}

impl Default for Render {
    fn default() -> Self {
        Render { hint: true, smooth: true }
    }
}

pub(super) struct Mask {
    left: i32,
    top: i32,
    width: usize,
    height: usize,
    cover: Box<[u8]>,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub(super) struct MaskKey {
    pub(super) font: fontdb::ID,
    pub(super) glyph: u16,
    pub(super) size: u32,
    pub(super) weight: u16,
    pub(super) lean: bool,
}

#[derive(Default)]
pub(super) struct Hinting {
    styles: OnceCell<GlyphStyles>,
}

#[derive(Default)]
pub(super) struct Sharp {
    masks: RefCell<HashMap<MaskKey, Option<Rc<Mask>>>>,
    hinters: RefCell<HashMap<(fontdb::ID, u16, u32), Option<Rc<HintingInstance>>>>,
    tables: RefCell<HashMap<u8, Rc<[u8; 256]>>>,
}

impl Sharp {
    pub(super) fn mask(&self, data: &[u8], index: u32, family: &str, hinting: &Hinting, key: MaskKey) -> Option<Rc<Mask>> {
        if let Some(m) = self.masks.borrow().get(&key) {
            return m.clone();
        }
        let known = self.render(family);
        let m = self.draw_mask(data, index, known.unwrap_or_default(), hinting, key).map(Rc::new);
        if known.is_none() {
            return m;
        }
        let mut masks = self.masks.borrow_mut();
        if masks.len() >= MOST_MASKS {
            masks.clear();
        }
        masks.insert(key, m.clone());
        m
    }

    fn draw_mask(&self, data: &[u8], index: u32, render: Render, hinting: &Hinting, key: MaskKey) -> Option<Mask> {
        let font = FontRef::from_index(data, index).ok()?;
        let outlines = font.outline_glyphs();
        let glyph = outlines.get(GlyphId::new(key.glyph as u32))?;
        let location = font.axes().location([("wght", key.weight as f32)]);
        let size = Size::new(f32::from_bits(key.size));
        let mut pen = Gather::default();
        let hinter = if render.hint { self.hinter(&outlines, &location, hinting, key) } else { None };
        match &hinter {
            Some(h) => glyph.draw(DrawSettings::hinted(h, false), &mut pen).ok()?,
            None => glyph.draw(DrawSettings::unhinted(size, LocationRef::from(&location)), &mut pen).ok()?,
        };
        pen.close();
        pen.fill(if key.lean { 0.2 } else { 0.0 }, render.smooth)
    }

    fn hinter(&self, outlines: &skrifa::outline::OutlineGlyphCollection, location: &Location, hinting: &Hinting,
              key: MaskKey) -> Option<Rc<HintingInstance>> {
        let at = (key.font, key.weight, key.size);
        if let Some(h) = self.hinters.borrow().get(&at) {
            return h.clone();
        }
        let engine = match outlines.format() {
            Some(OutlineGlyphFormat::Cff | OutlineGlyphFormat::Cff2) => Engine::Interpreter,
            _ => Engine::Auto(Some(hinting.styles.get_or_init(|| GlyphStyles::new(outlines)).clone())),
        };
        let target = Target::Smooth { mode: SmoothMode::Light, symmetric_rendering: true, preserve_linear_metrics: true };
        let options = HintingOptions { engine, target };
        let h = HintingInstance::new(outlines, Size::new(f32::from_bits(key.size)), location, options).ok().map(Rc::new);
        let mut hinters = self.hinters.borrow_mut();
        if hinters.len() >= MOST_HINTERS {
            hinters.clear();
        }
        hinters.insert(at, h.clone());
        h
    }

    pub(super) fn ask(&self, family: &str) {
        if super::text::only_built_in() {
            return;
        }
        fc().ask(&render_args(family));
    }

    fn render(&self, family: &str) -> Option<Render> {
        if super::text::only_built_in() {
            return Some(Render::default());
        }
        fc().answer(&render_args(family), super::text::waits()).map(|a| read_fontconfig(&a))
    }

    pub(super) fn table(&self, colour: Colour) -> Rc<[u8; 256]> {
        let luma = |v: f32| v.clamp(0.0, 1.0).powf(GAMMA);
        let lum = (0.2126 * luma(colour.r) + 0.7152 * luma(colour.g) + 0.0722 * luma(colour.b)).powf(1.0 / GAMMA);
        let step = (lum * 32.0).round() as u8;
        if let Some(t) = self.tables.borrow().get(&step) {
            return t.clone();
        }
        let t = Rc::new(correction(step as f32 / 32.0));
        self.tables.borrow_mut().insert(step, t.clone());
        t
    }
}

const ANSWER_WITHIN: Duration = Duration::from_millis(1500);

pub(super) fn answer_within(child: &mut Child, limit: Duration) -> Option<String> {
    let start = Instant::now();
    loop {
        match child.try_wait() {
            Ok(None) if start.elapsed() < limit => std::thread::sleep(Duration::from_millis(2)),
            Ok(None) => {
                let _ = child.kill();
                let _ = child.wait();
                return None;
            }
            _ => break,
        }
    }
    let mut answer = String::new();
    child.stdout.take()?.read_to_string(&mut answer).ok()?;
    Some(answer)
}

const FC_MATCH: &str = "/usr/bin/fc-match";

pub(super) fn render_args(family: &str) -> Vec<String> {
    let mut pattern = String::new();
    for c in family.chars() {
        if matches!(c, '-' | ':' | ',' | '\\') {
            pattern.push('\\');
        }
        pattern.push(c);
    }
    vec!["-f".into(), "%{hinting}|%{hintstyle}|%{antialias}".into(), pattern]
}

pub(super) fn mono_args() -> Vec<String> {
    vec!["-f".into(), "%{family}".into(), "monospace".into()]
}

#[derive(Clone)]
pub(super) struct Lookups(Arc<Inner>);

struct Inner {
    program: String,
    done: Mutex<HashMap<Vec<String>, Option<String>>>,
    failed: Mutex<HashMap<Vec<String>, std::time::Instant>>,
    arrived: Condvar,
}

const RETRY_AFTER: Duration = Duration::from_secs(5);

static ARRIVALS: AtomicU32 = AtomicU32::new(0);
static LISTENERS: Mutex<Vec<Box<dyn Fn() -> bool + Send>>> = Mutex::new(Vec::new());

pub(super) fn arrivals() -> u32 {
    ARRIVALS.load(Ordering::Relaxed)
}

pub(super) fn arrived() {
    ARRIVALS.fetch_add(1, Ordering::Relaxed);
    if let Ok(mut all) = LISTENERS.lock() {
        all.retain(|call| call());
    }
}

pub(super) fn on_arrival(call: impl Fn() -> bool + Send + 'static) {
    if let Ok(mut all) = LISTENERS.lock() {
        all.push(Box::new(call));
    }
}

pub(super) fn fc() -> &'static Lookups {
    static FC: OnceLock<Lookups> = OnceLock::new();
    FC.get_or_init(|| Lookups::new(FC_MATCH))
}

impl Lookups {
    pub(super) fn new(program: &str) -> Lookups {
        Lookups(Arc::new(Inner { program: program.to_string(), done: Mutex::default(), failed: Mutex::default(), arrived: Condvar::new() }))
    }

    pub(super) fn ask(&self, args: &[String]) {
        if self.0.failed.lock().is_ok_and(|f| f.get(args).is_some_and(|at| at.elapsed() < RETRY_AFTER)) {
            return;
        }
        let Ok(mut done) = self.0.done.lock() else { return };
        if done.contains_key(args) {
            return;
        }
        done.insert(args.to_vec(), None);
        drop(done);
        let (me, own) = (self.0.clone(), args.to_vec());
        let run = std::thread::Builder::new().name("fontconfig".into()).spawn(move || {
            let answer = Command::new(&me.program).args(&own).stdin(Stdio::null()).stdout(Stdio::piped()).stderr(Stdio::null()).spawn().ok()
                .and_then(|mut child| answer_within(&mut child, ANSWER_WITHIN));
            me.finish(own, answer);
        });
        if run.is_err() {
            self.0.finish(args.to_vec(), None);
        }
    }

    pub(super) fn answer(&self, args: &[String], wait: bool) -> Option<String> {
        self.ask(args);
        let mut done = self.0.done.lock().ok()?;
        loop {
            match done.get(args) {
                Some(Some(a)) => return Some(a.clone()),
                Some(None) if wait => done = self.0.arrived.wait(done).ok()?,
                _ => return None,
            }
        }
    }
}

impl Inner {
    fn finish(&self, args: Vec<String>, answer: Option<String>) {
        let arrived_now = answer.is_some();
        if let Ok(mut done) = self.done.lock() {
            match answer {
                Some(a) => done.insert(args, Some(a)),
                None => {
                    if let Ok(mut failed) = self.failed.lock() {
                        failed.insert(args.clone(), std::time::Instant::now());
                    }
                    done.remove(&args)
                }
            };
        }
        self.arrived.notify_all();
        if arrived_now {
            arrived();
        }
    }
}

fn read_fontconfig(answer: &str) -> Render {
    let mut parts = answer.trim().split('|');
    let (hinting, style, smooth) = (parts.next().unwrap_or(""), parts.next().unwrap_or(""), parts.next().unwrap_or(""));
    Render { hint: hinting != "False" && style != "0", smooth: smooth != "False" }
}

fn correction(src: f32) -> [u8; 256] {
    let dst = 1.0 - src;
    let (lin_src, lin_dst) = (src.powf(GAMMA), dst.powf(GAMMA));
    let contrast = CONTRAST * lin_dst;
    let mut table = [0u8; 256];
    for (i, out) in table.iter_mut().enumerate() {
        let raw = i as f32 / 255.0;
        let a = raw + (1.0 - raw) * contrast * raw;
        let v = if (src - dst).abs() < 1.0 / 256.0 {
            a
        } else {
            let lin = lin_src * a + (1.0 - a) * lin_dst;
            (lin.powf(1.0 / GAMMA) - dst) / (src - dst)
        };
        *out = (v.clamp(0.0, 1.0) * 255.0).round() as u8;
    }
    table
}

pub(super) fn blit(canvas: &mut PixmapMut, mask: &Mask, x: i32, y: i32, colour: Colour, table: &[u8; 256]) {
    let (cw, ch) = (canvas.width() as i32, canvas.height() as i32);
    let ink = colour.to_skia().to_color_u8();
    let ink = [ink.red() as u32, ink.green() as u32, ink.blue() as u32];
    let alpha = (colour.a.clamp(0.0, 1.0) * 255.0).round() as u32;
    let data = canvas.data_mut();
    let (x0, y0) = (x + mask.left, y + mask.top);
    let cols = (0.max(-x0) as usize)..(mask.width.min((cw - x0).max(0) as usize));
    for row in (0.max(-y0) as usize)..(mask.height.min((ch - y0).max(0) as usize)) {
        let line = &mask.cover[row * mask.width..(row + 1) * mask.width];
        let at = ((y0 + row as i32) * cw + x0) as isize;
        for col in cols.clone() {
            let a = div255(table[line[col] as usize] as u32 * alpha);
            if a == 0 {
                continue;
            }
            let i = (at + col as isize) as usize * 4;
            let keep = 255 - a;
            for k in 0..3 {
                data[i + k] = (div255(ink[k] * a) + div255(data[i + k] as u32 * keep)).min(255) as u8;
            }
            data[i + 3] = (a + div255(data[i + 3] as u32 * keep)).min(255) as u8;
        }
    }
}

impl Mask {
    pub(super) fn from_parts(left: i32, top: i32, width: usize, height: usize, cover: Box<[u8]>) -> Mask {
        debug_assert_eq!(cover.len(), width * height);
        Mask { left, top, width, height, cover }
    }

    pub(super) fn hollow(&self) -> Mask {
        Mask { left: self.left, top: self.top, width: self.width, height: self.height, cover: Box::new([]) }
    }

    pub(super) fn parts(&self) -> (i32, i32, usize, usize, &[u8]) {
        (self.left, self.top, self.width, self.height, &self.cover)
    }
}

fn div255(v: u32) -> u32 {
    let v = v + 128;
    (v + (v >> 8)) >> 8
}

const MOST_MASK: f32 = 4096.0;

#[derive(Clone, Copy)]
enum Segment {
    Line([Point; 2]),
    Quad([Point; 3]),
    Cubic([Point; 4]),
}

impl Segment {
    fn points(&self) -> &[Point] {
        match self {
            Segment::Line(p) => p,
            Segment::Quad(p) => p,
            Segment::Cubic(p) => p,
        }
    }
}

#[derive(Default)]
struct Gather {
    segments: Vec<Segment>,
    start: Point,
    last: Point,
}

impl Gather {
    fn close(&mut self) {
        if self.last != self.start {
            self.segments.push(Segment::Line([self.last, self.start]));
            self.last = self.start;
        }
    }

    fn fill(&self, lean: f32, smooth: bool) -> Option<Mask> {
        let place = |p: Point| point(p.x + lean * p.y, -p.y);
        let mut all = self.segments.iter().flat_map(|s| s.points()).map(|&p| place(p));
        let first = all.next()?;
        let (mut lo, mut hi) = (first, first);
        for p in all {
            lo = point(lo.x.min(p.x), lo.y.min(p.y));
            hi = point(hi.x.max(p.x), hi.y.max(p.y));
        }
        if !((hi.x - lo.x) <= MOST_MASK && (hi.y - lo.y) <= MOST_MASK) {
            return None;
        }
        let (left, top) = (lo.x.floor() as i32, lo.y.floor() as i32);
        let (width, height) = ((hi.x.ceil() as i32 - left).max(1) as usize, (hi.y.ceil() as i32 - top).max(1) as usize);
        let at = |p: Point| {
            let p = place(p);
            point(p.x - left as f32, p.y - top as f32)
        };
        let mut raster = Rasterizer::new(width, height);
        for segment in &self.segments {
            match *segment {
                Segment::Line([a, b]) => raster.draw_line(at(a), at(b)),
                Segment::Quad([a, b, c]) => raster.draw_quad(at(a), at(b), at(c)),
                Segment::Cubic([a, b, c, d]) => raster.draw_cubic(at(a), at(b), at(c), at(d)),
            }
        }
        let mut cover = vec![0u8; width * height].into_boxed_slice();
        raster.for_each_pixel(|i, v| {
            cover[i] = if smooth { (v.clamp(0.0, 1.0) * 255.0).round() as u8 } else if v >= 0.5 { 255 } else { 0 };
        });
        Some(Mask { left, top, width, height, cover })
    }
}

impl OutlinePen for Gather {
    fn move_to(&mut self, x: f32, y: f32) {
        self.close();
        self.start = point(x, y);
        self.last = self.start;
    }

    fn line_to(&mut self, x: f32, y: f32) {
        let p = point(x, y);
        self.segments.push(Segment::Line([self.last, p]));
        self.last = p;
    }

    fn quad_to(&mut self, cx0: f32, cy0: f32, x: f32, y: f32) {
        let p = point(x, y);
        self.segments.push(Segment::Quad([self.last, point(cx0, cy0), p]));
        self.last = p;
    }

    fn curve_to(&mut self, cx0: f32, cy0: f32, cx1: f32, cy1: f32, x: f32, y: f32) {
        let p = point(x, y);
        self.segments.push(Segment::Cubic([self.last, point(cx0, cy0), point(cx1, cy1), p]));
        self.last = p;
    }

    fn close(&mut self) {
        Gather::close(self);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fontconfig_turns_hinting_and_smoothing_off_as_gtk_would() {
        assert_eq!(read_fontconfig("True|1|True\n"), Render { hint: true, smooth: true });
        assert_eq!(read_fontconfig("True|3|True"), Render { hint: true, smooth: true });
        assert_eq!(read_fontconfig("False|1|True"), Render { hint: false, smooth: true });
        assert_eq!(read_fontconfig("True|0|True"), Render { hint: false, smooth: true });
        assert_eq!(read_fontconfig("True|1|False"), Render { hint: true, smooth: false });
        assert_eq!(read_fontconfig(""), Render::default());
    }

    #[test]
    fn full_cover_stays_full_and_none_stays_none() {
        for src in [0.0, 0.3, 0.5, 0.9, 1.0] {
            let t = correction(src);
            assert_eq!((t[0], t[255]), (0, 255));
            assert!(t.windows(2).all(|w| w[0] <= w[1]), "{src}");
        }
        assert!(correction(0.9)[128] > 128);
    }

    #[test]
    fn a_square_fills_its_pixels_exactly() {
        let mut pen = Gather::default();
        pen.move_to(1.0, 0.0);
        pen.line_to(3.0, 0.0);
        pen.line_to(3.0, 2.0);
        pen.line_to(1.0, 2.0);
        pen.close();
        let m = pen.fill(0.0, true).unwrap();
        assert_eq!((m.left, m.top, m.width, m.height), (1, -2, 2, 2));
        assert!(m.cover.iter().all(|&c| c == 255));
    }

    #[test]
    fn a_first_drawing_never_waits_for_a_slow_fontconfig() {
        use std::os::unix::fs::PermissionsExt;
        let dir = std::env::temp_dir().join(format!("trindle-slow-fc-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let program = dir.join("fc-match");
        std::fs::write(&program, "#!/bin/sh\nsleep 1\necho \"$3\"\n").unwrap();
        std::fs::set_permissions(&program, std::fs::Permissions::from_mode(0o755)).unwrap();
        let slow = Lookups::new(program.to_str().unwrap());
        let args = render_args("Some Family");
        let before = arrivals();
        let began = Instant::now();
        assert_eq!(slow.answer(&args, false), None);
        assert_eq!(slow.answer(&args, false), None, "asked once, still out");
        assert!(began.elapsed() < Duration::from_millis(500), "{:?}", began.elapsed());
        assert_eq!(slow.answer(&args, true).as_deref(), Some("Some Family\n"));
        assert!(arrivals() > before);
        assert_eq!(slow.answer(&args, false).as_deref(), Some("Some Family\n"));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn a_missing_program_answers_nothing_so_the_defaults_are_used() {
        let none = Lookups::new("/nonexistent/fc-match");
        assert_eq!(none.answer(&mono_args(), true), None);
        assert_eq!(none.answer(&mono_args(), false), None, "a failure is not kept, so the next ask runs again");
        for _ in 0..100 {
            assert_eq!(none.answer(&mono_args(), true), None);
        }
        assert!(none.0.done.lock().unwrap().is_empty(), "no run is left waiting after a failure");
        assert_eq!(read_fontconfig(""), Render::default());
    }
}
