//! Text: system fonts, shaping, stand-in fonts, bidi, line breaks and painting.

use std::cell::{OnceCell, RefCell};
use std::collections::HashMap;
use std::ops::Range;
use std::rc::Rc;
use std::sync::Arc;

use fontdb::{Database, Family, Query, Stretch, Style as Slant, Weight, ID};
use skrifa::instance::{Location, LocationRef, Size};
use skrifa::outline::{DrawSettings, OutlinePen};
use skrifa::raw::TableProvider;
use skrifa::{FontRef, GlyphId, MetadataProvider};
use tiny_skia::{FillRule, Path, PathBuilder, PixmapMut, PixmapPaint, Transform};
use unicode_bidi::{BidiClass, BidiInfo, Level};
use unicode_linebreak::BreakOpportunity;

use super::colour::Colour;
use super::emoji::{self, Picture};
use super::paint::cpu::play;
use super::paint::{Glyph, Painter};
use super::shadow::Shadows;
use super::sharp::{self, Hinting, MaskKey, Sharp};

mod fonts_list;
pub use fonts_list::FontFamily;

pub const FAMILY: &str = "Trindle Sans";
pub const BUILT_IN: &str = "Noto Sans";
pub const BUILT_IN_MONO: &str = "Noto Sans Mono";
const BUILT_IN_FAMILIES: [&str; 3] = [BUILT_IN, BUILT_IN_MONO, "Noto Sans Arabic"];
const BUILT_IN_FILES: [&[u8]; 8] = [
    include_bytes!("../share/fonts/NotoSans-Regular.ttf"),
    include_bytes!("../share/fonts/NotoSans-Bold.ttf"),
    include_bytes!("../share/fonts/NotoSans-Italic.ttf"),
    include_bytes!("../share/fonts/NotoSans-BoldItalic.ttf"),
    include_bytes!("../share/fonts/NotoSansMono-Regular.ttf"),
    include_bytes!("../share/fonts/NotoSansMono-Bold.ttf"),
    include_bytes!("../share/fonts/NotoSansArabic-Regular.ttf"),
    include_bytes!("../share/fonts/NotoSansArabic-Bold.ttf"),
];
pub const MONOSPACE: &str = "monospace";

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Align {
    #[default]
    Start,
    Centre,
    End,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Style {
    pub family: Option<String>,
    pub size: f32,
    pub weight: u16,
    pub italic: bool,
    pub align: Align,
    pub spacing: f32,
}

impl Style {
    pub fn new(size: f32) -> Style {
        Style { family: None, size, weight: 400, italic: false, align: Align::Start, spacing: 0.0 }
    }

    pub fn monospace(size: f32) -> Style {
        Style { family: Some(MONOSPACE.to_string()), ..Style::new(size) }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Placed {
    font: ID,
    pub glyph: u16,
    pub x: f32,
    pub y: f32,
    pub at: usize,
    pub rtl: bool,
}

#[derive(Clone, Debug)]
pub struct Line {
    pub range: Range<usize>,
    pub shown_end: usize,
    pub x: f32,
    pub top: f32,
    pub baseline: f32,
    pub height: f32,
    pub width: f32,
    pub glyphs: Vec<Placed>,
}

#[derive(Clone, Debug)]
pub struct Layout {
    pub width: f32,
    pub height: f32,
    pub lines: Vec<Line>,
    size: f32,
    weight: f32,
    lean: bool,
    pub cut: bool,
    whole: Rc<str>,
}

impl Layout {
    pub fn widest(&self) -> f32 {
        self.lines.iter().map(|l| l.width).fold(0.0, f32::max)
    }

    pub fn left(&self) -> f32 {
        self.lines.iter().map(|l| l.x).fold(f32::INFINITY, f32::min)
    }

    #[cfg(any(test, feature = "testing"))]
    pub fn whole(&self) -> &Rc<str> {
        &self.whole
    }

    pub fn cut(&self, room: f32, dots: &Layout) -> Layout {
        let (Some(line), Some(tail)) = (self.lines.first(), dots.lines.first()) else { return self.clone() };
        if self.width <= room {
            return self.clone();
        }
        let (mut kept, mut x, width) = cut_line(line, (room - dots.width).max(0.0), tail);
        if line.glyphs.first().is_some_and(|g| g.rtl) {
            for g in &mut kept {
                g.x += line.x - x;
            }
            x = line.x;
        }
        let mut out = self.clone();
        out.cut = true;
        out.lines.truncate(1);
        let line = &mut out.lines[0];
        (line.glyphs, line.x, line.width) = (kept, x, width);
        out.width = x + width;
        out
    }

    pub fn cut_after(&self, lines: usize, room: f32, dots: &Layout) -> Layout {
        let Some(tail) = dots.lines.first() else { return self.clone() };
        if lines == 0 || self.lines.len() <= lines {
            return self.clone();
        }
        let mut out = self.clone();
        out.cut = true;
        out.lines.truncate(lines);
        let line = &mut out.lines[lines - 1];
        let (kept, x, width) = cut_line(line, (room - dots.width).max(0.0), tail);
        (line.glyphs, line.x, line.width) = (kept, x, width);
        out.height = line.top + line.height;
        out.width = out.lines.iter().map(|l| l.x + l.width).fold(0.0, f32::max);
        out
    }

    pub fn spaced(&self, gap: f32) -> Layout {
        let mut out = self.clone();
        for line in &mut out.lines {
            let mut order: Vec<usize> = (0..line.glyphs.len()).collect();
            order.sort_by(|&a, &b| line.glyphs[a].x.total_cmp(&line.glyphs[b].x));
            let (mut letters, mut last) = (0usize, None);
            for i in order {
                let g = &mut line.glyphs[i];
                if last.is_some_and(|l| l != g.at) {
                    letters += 1;
                }
                last = Some(g.at);
                g.x += gap * letters as f32;
            }
            line.width += gap * letters as f32;
        }
        out.width = out.lines.iter().map(|l| l.width).fold(0.0, f32::max);
        out
    }

    pub fn hit(&self, x: f32, y: f32) -> usize {
        let Some(line) = self.lines.iter().find(|l| y < l.top + l.height).or(self.lines.last()) else { return 0 };
        let mut best = (f32::MAX, line.range.start);
        for ((i, g), next) in line.glyphs.iter().enumerate().zip(next_ats(line)) {
            let right = line.glyphs.get(i + 1).map_or(line.x + line.width, |n| n.x);
            let next = next.unwrap_or(line.shown_end);
            let (left_at, right_at) = if g.rtl { (next, g.at) } else { (g.at, next) };
            for (edge, at) in [(g.x, left_at), (right, right_at)] {
                if (edge - x).abs() < best.0 {
                    best = ((edge - x).abs(), at);
                }
            }
        }
        best.1
    }

    pub fn mark_bands(&self, ranges: &[Range<usize>]) -> Vec<(f32, f32, f32, f32)> {
        let mut out = Vec::new();
        if ranges.is_empty() {
            return out;
        }
        for line in &self.lines {
            let mut spans: Vec<(f32, f32)> = Vec::new();
            for ((i, g), next) in line.glyphs.iter().enumerate().zip(next_ats(line)) {
                let right = line.glyphs.get(i + 1).map_or(line.x + line.width, |n| n.x);
                let next = next.unwrap_or(line.shown_end.max(g.at + 1));
                if ranges.iter().any(|r| g.at < r.end && next > r.start) {
                    let (a, b) = (g.x.min(right), g.x.max(right));
                    spans.push((a, b));
                }
            }
            spans.sort_by(|a, b| a.0.total_cmp(&b.0));
            let mut joined: Vec<(f32, f32)> = Vec::new();
            for (a, b) in spans {
                match joined.last_mut() {
                    Some(last) if a <= last.1 + 0.5 => last.1 = last.1.max(b),
                    _ => joined.push((a, b)),
                }
            }
            out.extend(joined.into_iter().filter(|(a, b)| b > a).map(|(a, b)| (a, line.top, b - a, line.height)));
        }
        out
    }

    pub fn caret(&self, at: usize) -> (f32, f32, f32) {
        let Some(line) = self.lines.iter().find(|l| at < l.range.end || (at == l.range.end && at == l.shown_end))
            .or(self.lines.last()) else { return (0.0, 0.0, 0.0) };
        let first = line.glyphs.iter().enumerate().filter(|(_, g)| g.at >= at).min_by_key(|(_, g)| g.at);
        let x = match first {
            Some((i, g)) if g.rtl => line.glyphs.get(i + 1).map_or(line.x + line.width, |n| n.x),
            Some((_, g)) => g.x,
            None => match line.glyphs.iter().max_by_key(|g| g.at) {
                Some(g) if g.rtl => g.x,
                _ => line.x + line.width,
            },
        };
        (x, line.top, line.height)
    }
}

pub fn clean_marks(text: &str, ranges: &[Range<usize>]) -> Vec<Range<usize>> {
    let len = text.len();
    let mut tidy: Vec<Range<usize>> = ranges.iter().filter_map(|r| {
        if r.start >= r.end {
            return None;
        }
        let (mut a, mut b) = (r.start.min(len), r.end.min(len));
        while !text.is_char_boundary(a) {
            a -= 1;
        }
        while !text.is_char_boundary(b) {
            b += 1;
        }
        (a < b).then_some(a..b)
    }).collect();
    tidy.sort_by_key(|r| r.start);
    let mut out: Vec<Range<usize>> = Vec::new();
    for r in tidy {
        match out.last_mut() {
            Some(last) if r.start <= last.end => last.end = last.end.max(r.end),
            _ => out.push(r),
        }
    }
    out
}

const MOST_HOME_FONT: u64 = 64 << 20;

fn read_home_fonts(db: &mut Database) {
    if let Some(home) = std::env::var_os("HOME") {
        read_fonts_under(db, std::path::Path::new(&home));
    }
}

static WARM: std::sync::Mutex<Option<std::thread::JoinHandle<()>>> = std::sync::Mutex::new(None);
static SYSTEM: std::sync::OnceLock<Database> = std::sync::OnceLock::new();

fn scan() -> Database {
    let mut db = Database::new();
    db.load_system_fonts();
    read_home_fonts(&mut db);
    db
}

pub(crate) fn warm_fonts() {
    if only_built_in() {
        return;
    }
    if let Ok(mut warm) = WARM.lock() {
        if warm.is_none() && SYSTEM.get().is_none() {
            *warm = std::thread::Builder::new().name("fonts".into()).spawn(|| {
                let _ = SYSTEM.set(scan());
                sharp::arrived();
            }).ok();
        }
    }
    if !waits() {
        sharp::fc().ask(&sharp::mono_args());
        sharp::fc().ask(&sharp::render_args(FAMILY));
    }
}

static BUILT_IN_ONLY: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
use std::sync::atomic::Ordering;

#[cfg(any(test, feature = "testing"))]
pub(crate) fn built_in_mono_only() {
    BUILT_IN_ONLY.store(true, Ordering::Relaxed);
}

static ONLY_BUILT_IN: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

pub fn built_in_fonts_only() {
    ONLY_BUILT_IN.store(true, Ordering::Relaxed);
    BUILT_IN_ONLY.store(true, Ordering::Relaxed);
}

pub(crate) fn only_built_in() -> bool {
    ONLY_BUILT_IN.load(Ordering::Relaxed)
}

pub(crate) fn waits() -> bool {
    BUILT_IN_ONLY.load(Ordering::Relaxed) || cfg!(test)
}

pub(crate) fn on_fonts_ready(call: impl Fn() -> bool + Send + 'static) {
    sharp::on_arrival(call);
}

fn usable_mono(db: &Database, answer: &str) -> Option<String> {
    let name = answer.split(',').next()?.trim();
    (!BUILT_IN_FAMILIES.contains(&name) && db.faces().any(|f| f.monospaced && f.families.iter().any(|(n, _)| n == name))).then(|| name.to_string())
}

fn scanned(wait: bool) -> Option<Database> {
    scanned_from(&WARM, &SYSTEM, wait)
}

fn scanned_from(warm: &std::sync::Mutex<Option<std::thread::JoinHandle<()>>>, system: &std::sync::OnceLock<Database>, wait: bool) -> Option<Database> {
    let started = match warm.lock() {
        Ok(mut w) if system.get().is_none() && w.is_some() => match wait {
            true => w.take(),
            false => return None,
        },
        _ => None,
    };
    if let Some(h) = started {
        let _ = h.join();
    }
    Some(system.get_or_init(scan).clone())
}

fn sans_of(db: &Database) -> String {
    let own = !only_built_in() && db.faces().any(|f| f.families.iter().any(|(n, _)| n == FAMILY));
    if own { FAMILY } else { BUILT_IN }.to_string()
}

fn wanted_of(db: &Database, tags: &[String]) -> Vec<String> {
    let have: Vec<&str> = db.faces().filter_map(|f| f.families.first().map(|(n, _)| n.as_str())).collect();
    crate::lang::script::script_of(tags).map(|s| crate::lang::script::installed(s, &have)).unwrap_or_default()
}

fn built_in(f: &fontdb::FaceInfo) -> bool {
    f.families.iter().any(|(n, _)| BUILT_IN_FAMILIES.contains(&n.as_str()))
}

fn read_fonts_under(db: &mut Database, home: &std::path::Path) {
    let mut paths: Vec<std::path::PathBuf> = db.faces().filter_map(|f| match &f.source {
        fontdb::Source::File(p) if p.starts_with(home) => Some(p.clone()),
        _ => None,
    }).collect();
    paths.sort();
    paths.dedup();
    for path in paths {
        let Ok(data) = crate::common::files::open_file(&path).and_then(|f| {
            use std::io::Read;
            let mut data = Vec::new();
            f.take(MOST_HOME_FONT + 1).read_to_end(&mut data).map(|_| data)
        }) else { continue };
        if data.len() as u64 > MOST_HOME_FONT {
            continue;
        }
        let ids: Vec<ID> = db.faces().filter(|f| matches!(&f.source, fontdb::Source::File(p) if *p == path)).map(|f| f.id).collect();
        for id in ids {
            db.remove_face(id);
        }
        db.load_font_data(data);
    }
}

struct Loaded {
    data: Arc<dyn AsRef<[u8]> + Send + Sync>,
    index: u32,
    pictures: bool,
    family: String,
    hinting: Hinting,
    shaper: harfrust::Font,
    weights: RefCell<HashMap<u32, harfrust::Font>>,
}

impl Loaded {
    fn face(&self, weight: f32) -> Option<(FontRef<'_>, Location)> {
        let font = FontRef::from_index((*self.data).as_ref(), self.index).ok()?;
        let location = font.axes().location([("wght", weight)]);
        Some((font, location))
    }

    fn shaper(&self, weight: f32) -> harfrust::Font {
        let make = || self.shaper.instance_builder().variations([(harfrust::Tag::new(b"wght"), weight)]).build();
        self.weights.borrow_mut().entry(weight.to_bits()).or_insert_with(make).clone()
    }
}

struct Run {
    level: Level,
    font: ID,
    glyphs: Vec<Shaped>,
    ascent: f32,
    descent: f32,
    gap: f32,
}

impl Run {
    fn in_range(&self, shown: &Range<usize>) -> Vec<Shaped> {
        let g = &self.glyphs;
        let (from, to) = if self.level.is_rtl() {
            (g.partition_point(|x| x.at >= shown.end), g.partition_point(|x| x.at >= shown.start))
        } else {
            (g.partition_point(|x| x.at < shown.start), g.partition_point(|x| x.at < shown.end))
        };
        g[from..to].to_vec()
    }
}

#[derive(Clone, Copy)]
struct Shaped {
    glyph: u16,
    at: usize,
    advance: f32,
    dx: f32,
    dy: f32,
}

#[derive(Clone, Debug)]
pub(crate) struct Said {
    pub at: (f32, f32, f32, f32),
    pub whole: Rc<str>,
}

fn next_ats(line: &Line) -> Vec<Option<usize>> {
    let mut ats: Vec<usize> = line.glyphs.iter().map(|g| g.at).collect();
    ats.sort_unstable();
    ats.dedup();
    line.glyphs.iter().map(|g| ats.get(ats.partition_point(|&a| a <= g.at)).copied()).collect()
}

fn cut_line(line: &Line, fit: f32, tail: &Line) -> (Vec<Placed>, f32, f32) {
    let mut xs: Vec<f32> = line.glyphs.iter().map(|g| g.x).collect();
    xs.sort_by(f32::total_cmp);
    let far = line.x + line.width;
    let end = |x: f32| xs.get(xs.partition_point(|&n| n <= x)).copied().unwrap_or(far);
    let drop = line.baseline - tail.baseline;
    let tail_at = |kept: &mut Vec<Placed>, by: f32| {
        kept.extend(tail.glyphs.iter().map(|g| Placed { x: g.x + by, y: g.y + drop, ..*g }));
    };
    if line.glyphs.first().is_some_and(|g| g.rtl) {
        let mut kept: Vec<Placed> = line.glyphs.iter().copied().filter(|g| far - g.x <= fit).collect();
        let at = kept.iter().map(|g| g.x).fold(far, f32::min) - tail.width;
        tail_at(&mut kept, at);
        (kept, at, far - at)
    } else {
        let mut kept: Vec<Placed> = line.glyphs.iter().copied().filter(|g| end(g.x) <= fit).collect();
        let at = kept.iter().map(|g| end(g.x)).fold(line.x, f32::max);
        tail_at(&mut kept, at);
        (kept, line.x, at - line.x + tail.width)
    }
}

pub struct Fonts {
    db: RefCell<Database>,
    sans: RefCell<String>,
    mono: String,
    late_mono: OnceCell<String>,
    wanted: RefCell<Vec<String>>,
    pending: RefCell<Option<Vec<String>>>,
    loaded: RefCell<HashMap<ID, Rc<Loaded>>>,
    stand_ins: RefCell<HashMap<(char, u16, bool), Option<ID>>>,
    covers: RefCell<HashMap<ID, Rc<std::collections::HashSet<u32>>>>,
    stand_order: RefCell<HashMap<(bool, u16, bool), Rc<[ID]>>>,
    outlines: RefCell<HashMap<(ID, u16, u16), Option<(Rc<Path>, f32)>>>,
    pictures: RefCell<HashMap<(ID, u16, u32, u32), Option<Rc<Picture>>>>,
    sharp: Sharp,
    said: RefCell<Option<Vec<Said>>>,
}

impl Default for Fonts {
    fn default() -> Self {
        Self::new()
    }
}

impl Fonts {
    pub fn new() -> Fonts {
        Fonts::for_language(&crate::lang::choice())
    }

    pub fn for_language(tags: &[String]) -> Fonts {
        let sharp = Sharp::default();
        if only_built_in() {
            return Fonts::among_with(Database::new(), tags, sharp, None);
        }
        sharp.ask(FAMILY);
        match scanned(waits()) {
            Some(db) => Fonts::among_with(db, tags, sharp, None),
            None => {
                let fonts = Fonts::among_with(Database::new(), tags, sharp, None);
                fonts.pending.replace(Some(tags.to_vec()));
                fonts
            }
        }
    }

    fn catch_up(&self) {
        let Some(system) = SYSTEM.get().filter(|_| self.pending.borrow().is_some()) else { return };
        let Ok(mut db) = self.db.try_borrow_mut() else { return };
        let tags = self.pending.take().unwrap_or_default();
        for face in system.faces().filter(|f| !built_in(f)) {
            db.push_face_info(face.clone());
        }
        let sans = sans_of(&db);
        db.set_sans_serif_family(sans.clone());
        *self.sans.borrow_mut() = sans;
        *self.wanted.borrow_mut() = wanted_of(&db, &tags);
        self.stand_ins.borrow_mut().clear();
        self.stand_order.borrow_mut().clear();
    }

    fn db(&self) -> std::cell::Ref<'_, Database> {
        self.catch_up();
        self.db.borrow()
    }

    fn among_with(mut db: Database, tags: &[String], sharp: Sharp, mono: Option<String>) -> Fonts {
        let installed: Vec<ID> = db.faces().filter(|f| built_in(f)).map(|f| f.id).collect();
        for id in installed {
            db.remove_face(id);
        }
        for data in BUILT_IN_FILES {
            db.load_font_source(fontdb::Source::Binary(Arc::new(data)));
        }
        let mono = mono.and_then(|m| usable_mono(&db, &m)).unwrap_or_else(|| BUILT_IN_MONO.to_string());
        let sans = sans_of(&db);
        db.set_sans_serif_family(sans.clone());
        let wanted = wanted_of(&db, tags);
        Fonts {
            db: RefCell::new(db),
            sans: RefCell::new(sans),
            mono,
            late_mono: OnceCell::new(),
            wanted: RefCell::new(wanted),
            pending: RefCell::new(None),
            loaded: RefCell::default(),
            stand_ins: RefCell::default(),
            covers: RefCell::default(),
            stand_order: RefCell::default(),
            outlines: RefCell::default(),
            pictures: RefCell::default(),
            sharp,
            said: RefCell::new(None),
        }
    }

    pub fn families(&self) -> Vec<String> {
        let mut names: Vec<String> = self.db().faces().filter_map(|f| f.families.first().map(|(n, _)| n.clone())).collect();
        names.sort_by_key(|n| n.to_lowercase());
        names.dedup_by_key(|n| n.to_lowercase());
        names
    }

    pub fn monospace(&self) -> &str {
        if let Some(m) = self.late_mono.get() {
            return m;
        }
        self.catch_up();
        if waits() || self.mono != BUILT_IN_MONO || self.pending.borrow().is_some() {
            return &self.mono;
        }
        match sharp::fc().answer(&sharp::mono_args(), false) {
            Some(a) => self.late_mono.get_or_init(|| usable_mono(&self.db(), &a).unwrap_or_else(|| self.mono.clone())),
            None => &self.mono,
        }
    }

    pub(crate) fn generation(&self) -> u32 {
        sharp::arrivals()
    }

    fn chosen(&self, style: &Style) -> Option<ID> {
        let named = style.family.as_deref().map(|n| if n.eq_ignore_ascii_case(MONOSPACE) { self.monospace() } else { n }).map(Family::Name);
        let db = self.db();
        let sans = self.sans.borrow();
        let families: Vec<Family> = named.into_iter().chain([Family::Name(&sans), Family::SansSerif]).collect();
        db.query(&Query {
            families: &families,
            weight: Weight(style.weight),
            stretch: Stretch::Normal,
            style: if style.italic { Slant::Italic } else { Slant::Normal },
        })
    }

    fn load(&self, id: ID) -> Option<Rc<Loaded>> {
        if let Some(l) = self.loaded.borrow().get(&id) {
            return Some(l.clone());
        }
        let (source, index) = self.db().face(id).map(|f| (f.source.clone(), f.index))?;
        let data: Arc<dyn AsRef<[u8]> + Send + Sync> = match source {
            fontdb::Source::File(path) => Arc::new(std::fs::read(path).ok()?),
            fontdb::Source::Binary(data) | fontdb::Source::SharedFile(_, data) => data,
        };
        let pictures = FontRef::from_index((*data).as_ref(), index).is_ok_and(|f| emoji::has_pictures(&f));
        let shaper = harfrust::Font::new(data.clone(), index)?;
        let family = self.db().face(id).and_then(|f| f.families.first()).map(|(n, _)| n.clone()).unwrap_or_default();
        self.sharp.ask(&family);
        let l = Rc::new(Loaded { data, index, pictures, family, hinting: Hinting::default(), shaper, weights: RefCell::default() });
        self.loaded.borrow_mut().insert(id, l.clone());
        Some(l)
    }

    fn stand_in(&self, c: char, style: &Style) -> Option<ID> {
        if let Some(found) = self.stand_ins.borrow().get(&(c, style.weight, style.italic)) {
            return *found;
        }
        let db = self.db();
        let wanted = self.wanted.borrow();
        let picture = is_pictograph(c);
        let order = self.stand_order.borrow_mut().entry((picture, style.weight, style.italic)).or_insert_with(|| {
            let mut options: Vec<(u32, u32, bool, ID)> = db.faces().map(|f| {
                let family = f.families.first().map_or("", |(n, _)| n.as_str());
                let tier = wanted.iter().position(|w| w.eq_ignore_ascii_case(family)).map_or(wanted.len() as u32, |p| p as u32);
                let emoji = family.contains("Emoji");
                let mut cost = (f.weight.0 as i32 - style.weight as i32).unsigned_abs() / 100;
                cost += if emoji != picture { 40 } else { 0 };
                cost += if family.starts_with("Noto Sans") && !family.contains("Mono") { 0 } else { 20 };
                cost += if (f.style != Slant::Normal) != style.italic { 10 } else { 0 };
                (tier, cost, !BUILT_IN_FAMILIES.contains(&family), f.id)
            }).collect();
            options.sort_by_key(|&(tier, cost, installed, _)| (tier, cost, installed));
            options.into_iter().map(|(_, _, _, id)| id).collect()
        }).clone();
        let found = order.iter().copied().find(|&id| self.covers(&db, id).contains(&u32::from(c)));
        drop(db);
        self.stand_ins.borrow_mut().insert((c, style.weight, style.italic), found);
        found
    }

    fn covers(&self, db: &Database, id: ID) -> Rc<std::collections::HashSet<u32>> {
        if let Some(known) = self.covers.borrow().get(&id) {
            return known.clone();
        }
        let set = Rc::new(db.with_face_data(id, |data, index| {
            let mut set = std::collections::HashSet::new();
            if let Ok(font) = FontRef::from_index(data, index) {
                set.extend(font.charmap().mappings().map(|(cp, _)| cp));
            }
            set
        }).unwrap_or_default());
        self.covers.borrow_mut().insert(id, set.clone());
        set
    }

    pub fn layout(&self, text: &str, style: &Style, width: Option<f32>) -> Layout {
        let weight = style.weight as f32;
        let mut lines = Vec::new();
        let mut rtl_lines: Vec<bool> = Vec::new();
        let mut top = 0.0;
        let chosen = self.chosen(style);
        let lean = style.italic && chosen.is_some_and(|id| self.db().face(id).is_some_and(|f| f.style == Slant::Normal));
        let plain = chosen
            .and_then(|id| self.load(id))
            .and_then(|l| l.face(weight).map(|(f, at)| metrics(&f, &at, style.size)))
            .unwrap_or((style.size * 0.8, style.size * 0.2, 0.0));
        let mut start = 0;
        for para in text.split('\n') {
            let range = start..start + para.len();
            start = range.end + 1;
            let bidi = BidiInfo::new(para, None);
            let rtl = bidi.paragraphs.first().is_some_and(|p| p.level.is_rtl());
            let runs = self.shape(para, &bidi, chosen, style);
            let mut advance = vec![0.0f32; para.len() + 1];
            for g in runs.iter().flat_map(|r| &r.glyphs) {
                advance[g.at] += g.advance;
            }
            for piece in break_lines(para, &advance, width) {
                let line = set_line(&runs, para, &piece, range.start, top, plain);
                top += line.height;
                lines.push(line);
                rtl_lines.push(rtl);
            }
        }
        let widest = lines.iter().map(|l| l.width).fold(0.0, f32::max);
        let boxed = width.unwrap_or(widest);
        align_lines(&mut lines, &rtl_lines, boxed, style.align);
        Layout { width: boxed, height: top, lines, size: style.size, weight, lean, cut: false, whole: Rc::from(text) }
    }

    fn shape(&self, para: &str, bidi: &BidiInfo, chosen: Option<ID>, style: &Style) -> Vec<Run> {
        let weight = style.weight as f32;
        let main = chosen.and_then(|id| self.load(id));
        let main_map = main.as_ref().and_then(|l| l.face(weight)).map(|(f, _)| f.charmap());
        let mut pieces: Vec<(Range<usize>, Level, ID)> = Vec::new();
        for (i, c) in para.char_indices() {
            let level = bidi.levels[i];
            let joins = matches!(c, '\u{200d}' | '\u{fe00}'..='\u{fe0f}' | '\u{e0100}'..='\u{e01ef}');
            let follows = joins || c.is_whitespace() || c.is_control() || bidi.original_classes[i] == BidiClass::NSM;
            let as_emoji = is_pictograph(c) && para[i + c.len_utf8()..].starts_with('\u{fe0f}');
            let font = match (&main_map, chosen) {
                _ if joins => pieces.last().map(|p| p.2).or(chosen),
                (Some(map), Some(id)) if !as_emoji && map.map(c).is_some() => Some(id),
                _ if follows => pieces.last().map(|p| p.2).or(chosen),
                _ => self.stand_in(c, style).or(chosen),
            };
            let Some(font) = font else { continue };
            match pieces.last_mut() {
                Some(p) if p.1 == level && p.2 == font && p.0.end == i => p.0.end = i + c.len_utf8(),
                _ => pieces.push((i..i + c.len_utf8(), level, font)),
            }
        }
        pieces.into_iter().filter_map(|(range, level, font)| {
            let loaded = self.load(font)?;
            let (face, at) = loaded.face(weight)?;
            let shaper = loaded.shaper(weight);
            let scale = style.size / shaper.units_per_em() as f32;
            let mut out = harfrust::Buffer::new();
            out.push_str(&para[range.clone()]);
            out.set_direction(if level.is_rtl() { harfrust::Direction::RightToLeft } else { harfrust::Direction::LeftToRight });
            out.guess_segment_properties();
            harfrust::shape(&harfrust::ShaperFont::new(&shaper), &mut out, harfrust::ShapeOptions::new()).ok()?;
            let mut last = None;
            let glyphs = out.glyph_infos().iter().zip(out.glyph_positions()).map(|(info, pos)| {
                let first = last != Some(info.cluster);
                last = Some(info.cluster);
                Shaped {
                    glyph: info.glyph_id as u16,
                    at: range.start + info.cluster as usize,
                    advance: pos.x_advance as f32 * scale + if first { style.spacing } else { 0.0 },
                    dx: pos.x_offset as f32 * scale,
                    dy: pos.y_offset as f32 * scale,
                }
            }).collect();
            let (ascent, descent, gap) = metrics(&face, &at, style.size);
            Some(Run { level, font, glyphs, ascent, descent, gap })
        }).collect()
    }

    fn picture(&self, loaded: &Loaded, g: &Placed, size: f32, weight: f32, ink: Colour) -> Option<Rc<Picture>> {
        let opaque = Colour { a: 1.0, ..ink };
        let c = opaque.to_skia().to_color_u8();
        let key = (g.font, g.glyph, size.to_bits(), u32::from_be_bytes([c.red(), c.green(), c.blue(), 255]));
        if let Some(p) = self.pictures.borrow().get(&key) {
            return p.clone();
        }
        let p = loaded.face(weight).and_then(|(f, at)| emoji::picture(&f, &at, GlyphId::new(g.glyph.into()), size, opaque)).map(Rc::new);
        self.pictures.borrow_mut().insert(key, p.clone());
        p
    }

    pub(crate) fn listen(&self) {
        *self.said.borrow_mut() = Some(Vec::new());
    }

    pub(crate) fn heard(&self) -> Vec<Said> {
        self.said.borrow_mut().take().unwrap_or_default()
    }

    pub fn draw(&self, canvas: &mut PixmapMut, layout: &Layout, x: f32, y: f32, colour: Colour) {
        let mut painter = Painter::default();
        self.paint(&mut painter, layout, x, y, colour);
        play(painter.ops, canvas, &Shadows::default());
    }

    pub(crate) fn paint(&self, painter: &mut Painter, layout: &Layout, x: f32, y: f32, colour: Colour) {
        if layout.cut {
            if let Some(said) = self.said.borrow_mut().as_mut() {
                said.push(Said { at: (x, y, layout.width, layout.height), whole: layout.whole.clone() });
            }
        }
        #[cfg(any(test, feature = "testing"))]
        for line in layout.lines.iter().filter(|l| !l.glyphs.is_empty()) {
            noted(painter, colour, (x + line.x, y + line.top, line.width, line.height));
        }
        let table = self.sharp.table(colour);
        let mut run = Vec::new();
        for line in &layout.lines {
            let baseline = (y + line.baseline).round();
            for g in &line.glyphs {
                let at = (x + g.x, baseline + (g.y - line.baseline));
                if !self.sharp_glyph(&mut run, layout, g, at) {
                    if !run.is_empty() {
                        painter.glyphs(std::mem::take(&mut run), colour, table.clone());
                    }
                    self.glyph(painter, layout, g, at, Transform::identity(), colour);
                }
            }
        }
        if !run.is_empty() {
            painter.glyphs(run, colour, table);
        }
    }

    fn sharp_glyph(&self, run: &mut Vec<Glyph>, layout: &Layout, g: &Placed, at: (f32, f32)) -> bool {
        let Some(loaded) = self.load(g.font) else { return true };
        if loaded.pictures {
            return false;
        }
        let key = MaskKey {
            font: g.font,
            glyph: g.glyph,
            size: layout.size.to_bits(),
            weight: layout.weight as u16,
            lean: layout.lean,
        };
        if let Some(mask) = self.sharp.mask((*loaded.data).as_ref(), loaded.index, &loaded.family, &loaded.hinting, key) {
            run.push(Glyph { key, mask, x: (at.0 + 0.5).floor() as i32, y: (at.1 + 0.5).floor() as i32 });
        }
        true
    }

    pub fn draw_on_arc(&self, canvas: &mut PixmapMut, layout: &Layout, centre: (f32, f32), radius: f32,
                       mid_angle: f32, colour: Colour) {
        let mut painter = Painter::default();
        self.paint_on_arc(&mut painter, layout, centre, radius, mid_angle, colour);
        play(painter.ops, canvas, &Shadows::default());
    }

    pub(crate) fn paint_on_arc(&self, painter: &mut Painter, layout: &Layout, centre: (f32, f32), radius: f32,
                               mid_angle: f32, colour: Colour) {
        let Some(line) = layout.lines.first() else { return };
        if radius <= 0.0 {
            return;
        }
        #[cfg(any(test, feature = "testing"))]
        noted(painter, colour, (centre.0 - radius - line.height, centre.1 - radius - line.height,
                                2.0 * (radius + line.height), 2.0 * (radius + line.height)));
        let lower = mid_angle.sin() > 0.15;
        let dir = if lower { -1.0 } else { 1.0 };
        let start = mid_angle - dir * line.width / radius / 2.0;
        let drop = line.baseline - (line.top + line.height / 2.0);
        for g in &line.glyphs {
            let end = line.glyphs.iter().map(|n| n.x).filter(|&nx| nx > g.x).fold(line.x + line.width, f32::min);
            let half = (end - g.x) / 2.0;
            let along = g.x - line.x + half;
            let a = start + dir * along / radius;
            let place = Transform::from_rotate((a + dir * std::f32::consts::FRAC_PI_2).to_degrees())
                .post_translate(centre.0 + radius * a.cos(), centre.1 + radius * a.sin());
            self.glyph(painter, layout, g, (-half, drop + (g.y - line.baseline)), place, colour);
        }
    }

    fn glyph(&self, painter: &mut Painter, layout: &Layout, g: &Placed, at: (f32, f32), place: Transform, colour: Colour) {
        let Some(loaded) = self.load(g.font) else { return };
        let weight = layout.weight;
        let (gx, gy) = at;
        if loaded.pictures {
            if let Some(p) = self.picture(&loaded, g, layout.size, weight, colour) {
                let (px, py) = ((gx + p.left).round() as i32, (gy + p.top).round() as i32);
                let shown = tiny_skia::Rect::from_xywh(px as f32, py as f32, p.pixmap.width() as f32, p.pixmap.height() as f32)
                    .and_then(|r| r.transform(place)).and_then(|r| r.round_out());
                if let Some(shown) = shown {
                    let paint = PixmapPaint { opacity: colour.a, ..PixmapPaint::default() };
                    painter.pixels(shown, move |c| c.draw_pixmap(px, py, p.pixmap.as_ref(), &paint, place, None));
                }
                return;
            }
        }
        let key = (g.font, g.glyph, weight as u16);
        let cached = self.outlines.borrow().get(&key).cloned();
        let found = cached.unwrap_or_else(|| {
            let found = loaded.face(weight).and_then(|(f, at)| {
                let upem = f.head().ok()?.units_per_em() as f32;
                Some((Rc::new(outline(&f, &at, GlyphId::new(g.glyph.into()))?), upem))
            });
            self.outlines.borrow_mut().insert(key, found.clone());
            found
        });
        let Some((outline, upem)) = found else { return };
        let s = layout.size / upem;
        let lean = if layout.lean { s * 0.2 } else { 0.0 };
        let at = Transform::from_row(s, 0.0, lean, -s, gx, gy).post_concat(place);
        if let Some(path) = (*outline).clone().transform(at) {
            painter.fill_path(path, FillRule::Winding, colour);
        }
    }
}

#[cfg(any(test, feature = "testing"))]
fn noted(painter: &mut Painter, colour: Colour, at: (f32, f32, f32, f32)) {
    let (x0, y0) = (at.0.floor() as i32, at.1.floor() as i32);
    let (x1, y1) = ((at.0 + at.2).ceil() as i32, (at.1 + at.3).ceil() as i32);
    if let Some(box_) = tiny_skia::IntRect::from_ltrb(x0, y0, x1, y1) {
        painter.pixels(box_, move |c| super::theme::drawn::ink(c, super::theme::drawn::Kind::Words, colour, at));
    }
}

fn metrics(font: &FontRef, at: &Location, size: f32) -> (f32, f32, f32) {
    let m = font.metrics(Size::new(size), at);
    (m.ascent, -m.descent, m.leading)
}

pub(super) fn outline(font: &FontRef, at: &Location, glyph: GlyphId) -> Option<tiny_skia::Path> {
    let mut pen = Pen(PathBuilder::new());
    font.outline_glyphs().get(glyph)?.draw(DrawSettings::unhinted(Size::unscaled(), LocationRef::from(at)), &mut pen).ok()?;
    pen.0.finish()
}

fn is_pictograph(c: char) -> bool {
    matches!(c, '\u{1f000}'..='\u{1faff}' | '\u{2600}'..='\u{27bf}' | '\u{2b00}'..='\u{2bff}')
}

fn break_lines(para: &str, advance: &[f32], width: Option<f32>) -> Vec<Range<usize>> {
    let mut before = Vec::with_capacity(advance.len() + 1);
    before.push(0.0);
    before.extend(advance.iter().scan(0.0, |sum, a| {
        *sum += a;
        Some(*sum)
    }));
    let wide = |r: Range<usize>| -> f32 {
        let end = r.start + para[r.clone()].trim_end().len();
        before[end] - before[r.start]
    };
    let mut lines = Vec::new();
    let mut start = 0;
    let mut fits = None;
    for (at, kind) in unicode_linebreak::linebreaks(para) {
        if let Some(max) = width {
            if wide(start..at) > max {
                if let Some(f) = fits {
                    lines.push(start..f);
                    start = f;
                }
                while wide(start..at) > max {
                    let mut cut = start;
                    for (i, c) in para[start..at].char_indices() {
                        let next = start + i + c.len_utf8();
                        if cut > start && wide(start..next) > max {
                            break;
                        }
                        cut = next;
                    }
                    if cut >= at {
                        break;
                    }
                    lines.push(start..cut);
                    start = cut;
                }
            }
        }
        fits = Some(at);
        if kind == BreakOpportunity::Mandatory && at < para.len() {
            lines.push(start..at);
            start = at;
            fits = None;
        }
    }
    lines.push(start..para.len());
    lines
}

pub(crate) fn reorder<T>(parts: &mut [(Level, T)]) {
    let Some(top) = parts.iter().map(|p| p.0.number()).max() else { return };
    let lowest_odd = parts.iter().map(|p| p.0.number()).min().map_or(1, |m| m | 1);
    for level in (lowest_odd..=top).rev() {
        let mut i = 0;
        while i < parts.len() {
            if parts[i].0.number() < level {
                i += 1;
                continue;
            }
            let from = i;
            while i < parts.len() && parts[i].0.number() >= level {
                i += 1;
            }
            parts[from..i].reverse();
        }
    }
}

struct Pen(PathBuilder);

impl OutlinePen for Pen {
    fn move_to(&mut self, x: f32, y: f32) {
        self.0.move_to(x, y)
    }
    fn line_to(&mut self, x: f32, y: f32) {
        self.0.line_to(x, y)
    }
    fn quad_to(&mut self, x1: f32, y1: f32, x: f32, y: f32) {
        self.0.quad_to(x1, y1, x, y)
    }
    fn curve_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, x: f32, y: f32) {
        self.0.cubic_to(x1, y1, x2, y2, x, y)
    }
    fn close(&mut self) {
        self.0.close()
    }
}

fn set_line(runs: &[Run], para: &str, piece: &Range<usize>, para_start: usize, top: f32, plain: (f32, f32, f32)) -> Line {
    let shown = piece.start..piece.start + para[piece.clone()].trim_end().len();
    let mut parts: Vec<(Level, (&Run, Vec<Shaped>))> = runs.iter()
        .map(|r| (r.level, (r, r.in_range(&shown))))
        .filter(|(_, (_, g))| !g.is_empty())
        .collect();
    reorder(&mut parts);
    let used = || parts.iter().map(|(_, (r, _))| *r);
    let ascent = used().map(|r| r.ascent).fold(plain.0, f32::max);
    let descent = used().map(|r| r.descent).fold(plain.1, f32::max);
    let gap = used().map(|r| r.gap).fold(plain.2, f32::max);
    let baseline = top + ascent;
    let mut pen = 0.0;
    let mut glyphs = Vec::new();
    for (_, (run, shaped)) in &parts {
        for g in shaped {
            glyphs.push(Placed { font: run.font, glyph: g.glyph, x: pen + g.dx, y: baseline - g.dy, at: para_start + g.at, rtl: run.level.is_rtl() });
            pen += g.advance;
        }
    }
    Line {
        range: para_start + piece.start..para_start + piece.end,
        shown_end: para_start + shown.end,
        x: 0.0,
        top,
        baseline,
        height: ascent + descent + gap,
        width: pen,
        glyphs,
    }
}

fn align_lines(lines: &mut [Line], rtl_lines: &[bool], boxed: f32, align: Align) {
    for (line, &rtl) in lines.iter_mut().zip(rtl_lines) {
        let room = boxed - line.width;
        line.x = match (align, rtl) {
            (Align::Start, false) | (Align::End, true) => 0.0,
            (Align::Start, true) | (Align::End, false) => room,
            (Align::Centre, _) => room / 2.0,
        };
        for g in &mut line.glyphs {
            g.x += line.x;
        }
    }
}

#[cfg(test)]
mod tests;
