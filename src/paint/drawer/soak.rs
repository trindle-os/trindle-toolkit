//! A long run of the GPU drawer against the processor's, built to make it lose or mis-draw something: the two buffers of a window used in turn as the window does (the same slot twice now and then, an age of nought now and then, frames the window never drew between), damage kept, atlases made small so they overflow all the time, and buffers laid out linearly and as the driver chooses. Each frame's whole buffer is read back and compared with the processor's picture:

use std::cell::RefCell;
use std::rc::Rc;
use std::time::{Duration, Instant};

use tiny_skia::{IntRect, Pixmap};

use super::tests::buffer_with;
use crate::paint::remote::{Answer, Remote, Sending};
use crate::paint::service::rig::Running;
use crate::paint::here;
use super::*;
use crate::input::{Button, Input};
use crate::sharp::{Hinting, MaskKey, Sharp};
use crate::theme::Theme;
use crate::ui::Ui;
use crate::widget::{self, Node, Tone};
use crate::window::dmabuf::Ages;
use crate::paint::Op;

const INVALID: u64 = 0x00ff_ffff_ffff_ffff;
const NOTO: &[u8] = include_bytes!("../../../share/fonts/NotoSans-Regular.ttf");

struct Rng(u64);

impl Rng {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 >> 12;
        self.0 ^= self.0 << 25;
        self.0 ^= self.0 >> 27;
        self.0.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }

    fn below(&mut self, n: usize) -> usize {
        (self.next() >> 11) as usize % n.max(1)
    }

    fn chance(&mut self, per: usize) -> bool {
        self.below(1000) < per
    }

    fn range(&mut self, low: f32, high: f32) -> f32 {
        low + (self.next() >> 40) as f32 / (1u64 << 24) as f32 * (high - low)
    }
}

fn seed() -> u64 {
    std::env::var("TRINDLE_SOAK_SEED").ok().and_then(|s| s.parse().ok()).unwrap_or(20_261_031)
}

fn frames(usual: usize) -> usize {
    std::env::var("TRINDLE_SOAK_FRAMES").ok().and_then(|s| s.parse().ok()).unwrap_or(usual)
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum Layout {
    Linear,
    Named,
    Unnamed,
}

struct Buf {
    _bo: Option<gbm::Buffer>,
    d: gbm::Dmabuf,
    id: u64,
    modifier: Option<u64>,
    remote: Option<Rc<Remote>>,
}

impl Drop for Buf {
    fn drop(&mut self) {
        if let Some(r) = &self.remote {
            r.free(self.id);
        }
    }
}

struct Through(Sending);

impl Draws for Through {
    fn draw(&mut self, frame: &Frame, p: Painter, shadows: &Shadows) -> Drawn {
        match self.0.draw(frame, p, shadows) {
            Drawn::Sent { seq, parts } => match self.0.remote().block_for(seq) {
                Answer::Done => {
                    self.0.done();
                    parts.map_or(Drawn::Whole, Drawn::Parts)
                }
                Answer::Again if self.0.again() => Drawn::Again,
                Answer::Again | Answer::Gone => Drawn::Failed,
            },
            drawn => drawn,
        }
    }
}

thread_local! {
    static SERVICE: RefCell<Option<Rc<Remote>>> = const { RefCell::new(None) };
}

struct Rig {
    shared: Rc<Shared>,
    drawing: Box<dyn Draws>,
    layout: Layout,
    bufs: Vec<Buf>,
    ages: Ages,
    size: (u32, u32),
    scale: f32,
    last: usize,
    shown: Option<Vec<u8>>,
    rng: Rng,
    name: String,
    n: usize,
    count: [usize; 3],
}

thread_local! {
    static ROUNDED: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
    static STRAY: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
    static WIDEST: std::cell::Cell<u8> = const { std::cell::Cell::new(0) };
    static FAILS: std::cell::RefCell<Vec<String>> = const { std::cell::RefCell::new(Vec::new()) };
}

fn fail(what: String) {
    if std::env::var_os("TRINDLE_SOAK_SURVEY").is_none() {
        panic!("{what}");
    }
    eprintln!("FAIL {what}");
    FAILS.with_borrow_mut(|f| f.push(what));
}

struct Diff {
    first: (u32, u32, [u8; 4], [u8; 4]),
    over2: usize,
    over8: usize,
    widest: u8,
    boxed: Option<[u32; 4]>,
}

impl std::fmt::Display for Diff {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let (x, y, a, g) = self.first;
        write!(f, "first pixel that differs ({x}, {y}): processor {a:?}, GPU {g:?}; {} pixels differ by more than 2, {} by more than 8 (box {:?})", self.over2, self.over8, self.boxed)
    }
}

fn differ(want: &[u8], got: &[u8], w: usize) -> Option<Diff> {
    let mut d = Diff { first: (0, 0, [0; 4], [0; 4]), over2: 0, over8: 0, widest: 0, boxed: None };
    for (i, (a, b)) in want.chunks_exact(4).zip(got.chunks_exact(4)).enumerate() {
        let most = a.iter().zip(b).map(|(x, y)| x.abs_diff(*y)).max().unwrap_or(0);
        if most <= 2 {
            continue;
        }
        let (x, y) = ((i % w) as u32, (i / w) as u32);
        if d.over2 == 0 {
            d.first = (x, y, [a[0], a[1], a[2], a[3]], [b[0], b[1], b[2], b[3]]);
        }
        d.over2 += 1;
        d.widest = d.widest.max(most);
        if most > 8 {
            d.over8 += 1;
            d.boxed = Some(d.boxed.map_or([x, y, x, y], |b| [b[0].min(x), b[1].min(y), b[2].max(x), b[3].max(y)]));
        }
    }
    (d.over2 > 0).then_some(d)
}

fn dump(name: &str, n: usize, want: &[u8], got: &[u8], size: (u32, u32)) {
    thread_local! {
        static SAVED: RefCell<std::collections::HashMap<String, usize>> = RefCell::new(std::collections::HashMap::new());
    }
    let Some(dir) = std::env::var_os("TRINDLE_SOAK_DUMP") else { return };
    if SAVED.with_borrow_mut(|m| { let n = m.entry(name.to_string()).or_default(); *n += 1; *n > 2 }) {
        return;
    }
    let slug: String = name.chars().map(|c| if c.is_ascii_alphanumeric() { c } else { '-' }).collect();
    for (what, bytes) in [("processor", want), ("gpu", got)] {
        if let Some(pix) = tiny_skia::IntSize::from_wh(size.0, size.1).and_then(|s| Pixmap::from_vec(bytes.to_vec(), s)) {
            let _ = pix.save_png(std::path::Path::new(&dir).join(format!("{slug}-{n}-{what}.png")));
        }
    }
}

impl Rig {
    fn new(shared: &Rc<Shared>, name: &str, size: (u32, u32), scale: f32, layout: Layout, seed: u64) -> Rig {
        let mut rig = Rig {
            shared: shared.clone(), layout,
            drawing: match SERVICE.with_borrow(|s| s.clone()) {
                Some(r) => Box::new(Through(Sending::new(r))),
                None => Box::new(Drawing::new(shared.clone())),
            }, bufs: Vec::new(), ages: Ages::default(), size, scale, last: 1,
            shown: None, rng: Rng(seed | 1), name: name.to_string(), n: 0, count: [0; 3],
        };
        rig.resize(size, scale);
        rig
    }

    fn resize(&mut self, size: (u32, u32), scale: f32) {
        let mods: &[u64] = if self.layout == Layout::Linear { &[gbm::MOD_LINEAR] } else { &[] };
        let service = SERVICE.with_borrow(|s| s.clone());
        self.bufs = (0..2).map(|_| {
            let id = crate::paint::new_id();
            let (bo, d) = match &service {
                Some(r) => {
                    let own = buffer_with(&self.shared, 64, 64, &[]).1.modifier;
                    let named = [own];
                    let asked: &[u64] = if self.layout == Layout::Named && own != INVALID { &named } else { mods };
                    (None, r.buffer(id, size, asked).expect("a buffer from the service"))
                }
                None => {
                    let (bo, d) = buffer_with(&self.shared, size.0, size.1, mods);
                    (Some(bo), d)
                }
            };
            let modifier = (self.layout != Layout::Unnamed && d.modifier != INVALID).then_some(d.modifier);
            Buf { _bo: bo, d, id, modifier, remote: service.clone() }
        }).collect();
        (self.size, self.scale, self.ages, self.shown) = (size, scale, Ages::default(), None);
    }

    fn read(&self, slot: usize) -> Vec<u8> {
        let b = &self.bufs[slot];
        if self.layout == Layout::Linear {
            let raw = gbm::read_linear(&b.d).expect("the buffer read");
            let (w, p) = (b.d.width as usize, &b.d.planes[0]);
            return (0..b.d.height as usize).flat_map(|y| {
                let row = &raw[p.offset as usize + y * p.stride as usize..][..w * 4];
                row.chunks_exact(4).flat_map(|px| [px[2], px[1], px[0], px[3]])
            }).collect();
        }
        let gpu = self.shared.gpu();
        assert!(gpu.current());
        let t = gpu.target_of(&b.d, b.modifier).expect("the buffer imported to be read");
        let px = gpu.read(&t).expect("the buffer read");
        gpu.free(t);
        px.data().to_vec()
    }

    #[must_use]
    fn step(&mut self, mut play: impl FnMut(&Frame, &mut dyn Draws) -> (Pixmap, Drawn)) -> Option<(String, (u32, u32), Option<[i32; 4]>)> {
        let slot = if self.rng.chance(150) { self.last } else { 1 - self.last };
        let unknown = self.rng.chance(40);
        let age = if unknown { 0 } else { self.ages.age(slot) };
        let b = &self.bufs[slot];
        let frame = Frame { dmabuf: Some(&b.d), modifier: b.modifier, id: b.id, size: self.size, scale: self.scale, age, behind: &self.ages.behind[slot] };
        let behind = self.ages.behind[slot].len();
        let missed = self.ages.behind[slot].clone();
        let (want, drawn) = loop {
            let (want, drawn) = play(&frame, &mut *self.drawing);
            if !matches!(drawn, Drawn::Again) {
                break (want, drawn);
            }
        };
        let first = self.ages.fresh();
        let (kind, parts) = match &drawn {
            Drawn::Whole => ("whole", None),
            Drawn::Parts(p) => ("in parts", Some(p.clone())),
            _ => panic!("{}", self.context(slot, age, behind, "the GPU drawer failed (see TRINDLE_GPU_LOG=1)")),
        };
        self.count[0] += usize::from(parts.is_none());
        self.count[1] += usize::from(parts.is_some());
        self.count[2] += usize::from(age == 0);
        let told = parts.clone().filter(|_| !first);
        self.ages.shown(slot, told.as_deref());
        self.last = slot;
        self.n += 1;
        let got = self.read(slot);
        let wrong = differ(want.data(), &got, want.width() as usize).filter(|d| {
            ROUNDED.set(ROUNDED.get() + d.over2);
            STRAY.set(STRAY.get() + d.over8);
            WIDEST.set(WIDEST.get().max(d.widest));
            d.over8 > if d.widest <= 48 { 16 } else { 4 } || d.over2 > 2000
        }).map(|d| {
            dump(&self.name, self.n, want.data(), &got, self.size);
            let part = parts.as_ref().filter(|_| age > 0).and_then(|p| p.iter().chain(&missed).map(|r| [r[0], r[1], r[0] + r[2], r[1] + r[3]]).reduce(|a, r| [a[0].min(r[0]), a[1].min(r[1]), a[2].max(r[2]), a[3].max(r[3])]));
            (self.context(slot, age, behind, &format!("drawn {kind}{}; {d}", parts.as_ref().map_or(String::new(), |p| format!(" {p:?}")))), (d.first.0, d.first.1), part)
        });
        if let (Some(old), Some(parts)) = (&self.shown, &parts) {
            let w = want.width() as i32;
            let outside: Vec<(i32, i32, &[u8], &[u8])> = got.chunks_exact(4).zip(old.chunks_exact(4)).enumerate().filter_map(|(i, (a, b))| {
                let (x, y) = (i as i32 % w, i as i32 / w);
                let far = a.iter().zip(b).any(|(p, q)| p.abs_diff(*q) > 8);
                (far && !parts.iter().any(|p| x >= p[0] && y >= p[1] && x < p[0] + p[2] && y < p[1] + p[3])).then_some((x, y, a, b))
            }).collect();
            STRAY.set(STRAY.get() + outside.len().min(4));
            if let (true, Some((x, y, a, b))) = (outside.len() > 4, outside.first()) {
                fail(self.context(slot, age, behind, &format!("told only {parts:?} changed, but {} of the GPU's pixels went more than eight levels apart outside it, the first ({x}, {y}) from {b:?} to {a:?}", outside.len())));
            }
        }
        self.shown = Some(got);
        assert_eq!(self.shared.gpu().error(), 0, "{}", self.context(slot, age, behind, "a GL error"));
        wrong
    }

    fn context(&self, slot: usize, age: u32, behind: usize, what: &str) -> String {
        format!("scene {}, frame {} ({:?}, {} x {} at {}): buffer {slot}, age {age}, {behind} parts behind: {what}", self.name, self.n, self.layout, self.size.0, self.size.1, self.scale)
    }
}

fn run_ui(rig: &mut Rig, ui: &mut Ui<()>, count: usize, mut change: impl FnMut(&mut Ui<()>, usize, &mut Rng, Instant)) {
    ui.still();
    let start = Instant::now();
    for n in 0..count {
        let skipped = if rig.rng.chance(120) { 1 + rig.rng.below(3) } else { 0 };
        for k in 0..=skipped {
            let now = start + Duration::from_millis(33 * (n + k) as u64);
            let mut rng = Rng(rig.rng.next() | 1);
            change(ui, n + k, &mut rng, now);
            ui.lay_at(rig.size.0, rig.size.1);
            ui.tick(now);
        }
        let (size, scale) = (rig.size, rig.scale);
        let wrong = rig.step(|frame, drawing| {
            let mut want = Pixmap::new(size.0, size.1).unwrap();
            ui.paint(&mut want.as_mut(), scale);
            let drawn = ui.paint_by_gpu(frame, drawing);
            (want, drawn)
        });
        if let Some((what, at, part)) = wrong {
            println!("{what}");
            let count = ui.recorded(size, scale).ops.len();
            bisect(at, count, |k| {
                let last = name(&ui.recorded(size, scale).ops[k - 1]);
                let mut ops = ui.recorded(size, scale).ops;
                ops.truncate(k);
                let a = cpu_at(ops, size, at);
                let mut ops = ui.recorded(size, scale).ops;
                ops.truncate(k);
                (last, a, gpu_at(&rig.shared, ops, size, at, part))
            });
            fail(what);
        }
    }
}

fn use_it(ui: &mut Ui<()>, rng: &mut Rng, now: Instant) {
    let boxes = ui.boxes();
    if boxes.is_empty() {
        return;
    }
    let r = boxes[rng.below(boxes.len())].1;
    let (x, y) = (r.x + r.w * rng.range(0.1, 0.9), r.y + r.h * rng.range(0.1, 0.9));
    let first = rng.chance(100);
    ui.input(&if first { Input::PointerIn { x, y } } else { Input::PointerMove { x, y } }, now);
    if rng.chance(150) {
        ui.input(&Input::Press { button: Button::Left, x, y }, now);
        ui.input(&Input::Release { button: Button::Left, x, y }, now);
    }
    if rng.chance(60) {
        let sym = if rng.chance(500) { crate::input::Keysym::Tab } else { crate::input::Keysym::Escape };
        ui.input(&Input::Key { sym, text: None, mods: crate::input::Mods::default(), repeat: false }, now);
    }
}

fn looks() -> Vec<(String, Theme)> {
    let all = crate::theme::tests::every_look();
    let mut out: Vec<_> = all.into_iter().filter(|(n, _)| n.starts_with("Trindle") || n.contains("snug") || n.contains("paper")).collect();
    out.push(("chosen".into(), Theme::chosen()));
    out
}

const SIZES: [((u32, u32), f32); 5] = [((640, 480), 1.0), ((800, 600), 1.25), ((900, 560), 1.5), ((640, 400), 2.0), ((1000, 700), 1.0)];

const LAYOUTS: [Layout; 3] = [Layout::Linear, Layout::Named, Layout::Unnamed];

fn atlases(shared: &Shared, side: u32) {
    shared.gpu_mut().small_atlases(side);
}

fn gallery(shared: &Rc<Shared>, per_scene: usize, see_through: bool) {
    for (i, (name, theme)) in looks().into_iter().enumerate() {
        let (size, scale) = SIZES[i % SIZES.len()];
        atlases(shared, if i % 2 == 0 { 2048 } else { 192 });
        let mut rig = Rig::new(shared, &format!("gallery, {name}{}", if see_through { ", see-through" } else { "" }), size, scale, LAYOUTS[i % 3], seed() + i as u64);
        let mut ui = Ui::new(crate::gallery::gallery(0), theme);
        ui.see_through = see_through;
        run_ui(&mut rig, &mut ui, per_scene, |ui, n, rng, now| {
            use_it(ui, rng, now);
            if n % 17 == 16 {
                ui.set_view(crate::gallery::gallery(0));
            }
        });
    }
    atlases(shared, u32::MAX);
}

fn scrolling(shared: &Rc<Shared>, per_scene: usize) {
    for (i, (size, scale)) in SIZES.into_iter().enumerate() {
        atlases(shared, if i % 2 == 0 { 2048 } else { 200 });
        let mut rig = Rig::new(shared, "scrolling page", size, scale, LAYOUTS[i % 3], seed() + 100 + i as u64);
        let rows = 4;
        let mut ui = Ui::new(widget::scroll("soak", crate::big_screens_tests::page(size.0, size.1, scale, rows)), Theme::new(i % 2 == 0, "#fbc711"));
        run_ui(&mut rig, &mut ui, per_scene, |ui, n, rng, now| {
            let (x, y) = (rng.range(20.0, 300.0), rng.range(20.0, 300.0));
            ui.input(&Input::PointerMove { x, y }, now);
            if rng.chance(600) {
                ui.input(&Input::Scroll { x: 0.0, y: rng.range(-60.0, 160.0), steps: false }, now);
            }
            if n % 29 == 28 {
                ui.set_view(widget::scroll("soak", crate::big_screens_tests::page(size.0, size.1, scale, rows + n / 29)));
            }
        });
    }
    atlases(shared, u32::MAX);
}

fn lists(shared: &Rc<Shared>, per_scene: usize) {
    atlases(shared, 256);
    let rows = |n: usize| (0..300).map(|i| widget::item(format!("Row {i} of the list, version {}", n / 7)).icon(if i % 3 == 0 { "folder-symbolic" } else { "text-x-generic-symbolic" }).detail(format!("{} KB", i * 13))).collect::<Vec<_>>();
    let view = |n: usize| widget::row(vec![widget::list(rows(n), Some(n % 300), |_| ()).called("Rows").width(widget::Length::Fill), widget::scroll("others", widget::column((0..120).map(|i| widget::label(format!("Line {i}"))).collect()))]);
    let mut rig = Rig::new(shared, "lists", (800, 600), 1.0, Layout::Linear, seed() + 200);
    let mut ui = Ui::new(view(0), Theme::new(true, "#fbc711"));
    run_ui(&mut rig, &mut ui, per_scene, |ui, n, rng, now| {
        ui.input(&Input::PointerMove { x: rng.range(10.0, 790.0), y: rng.range(10.0, 590.0) }, now);
        if rng.chance(700) {
            ui.input(&Input::Scroll { x: 0.0, y: rng.range(-3.0, 5.0), steps: true }, now);
        }
        if n % 13 == 12 {
            ui.set_view(view(n));
        }
    });
    atlases(shared, u32::MAX);
}

const WORDS: [&str; 12] = [
    "The quick brown fox jumps over the lazy dog", "Falsches Üben von Xylophonmusik quält jeden größeren Zwerg", "Съешь же ещё этих мягких французских булок",
    "Γαζίες καὶ μυρτιὲς δὲν θὰ βρῶ πιὰ στὸ χρυσαφὶ ξέφωτο", "صِف خَلقَ خَودٍ كَمِثلِ الشَمسِ إِذ بَزَغَت", "דג סקרן שט בים מאוכזב ולפתע מצא חברה",
    "दिल्ली में सर्दी बहुत पड़ती है", "我能吞下玻璃而不伤身体 你好，世界", "吾輩は猫である。名前はまだ無い。こんにちは", "다람쥐 헌 쳇바퀴에 타고파 안녕하세요",
    "สิบเอ็ดล้านอีกสิบสองหมื่น", "Ça fait déjà près de 100 % ≠ ½ ≥ ∑ € £ ¥ ✓ ★ ☂ ♞",
];

fn words_and_pictures(shared: &Rc<Shared>, per_scene: usize, see_through: bool) {
    for (i, side) in [192u32, 256, 2048].into_iter().enumerate() {
        atlases(shared, side);
        let (size, scale) = SIZES[(i + 1) % SIZES.len()];
        let mut rig = Rig::new(shared, &format!("words and pictures, atlases {side}{}", if see_through { ", see-through" } else { "" }), size, scale, LAYOUTS[(i + 1) % 3], seed() + 300 + i as u64);
        let view = |n: usize, rng: &mut Rng| -> Node<()> {
            let line = |k: usize, rng: &mut Rng| -> Node<()> {
                let text = format!("{} {}", WORDS[(n + k + rng.below(3)) % WORDS.len()], n * 7 + k);
                match (k + n / 4) % 7 {
                    0 => widget::label(text),
                    1 => widget::heading(text),
                    2 => widget::title(text),
                    3 => widget::note(text),
                    4 => widget::headline(text),
                    5 => widget::lead(text),
                    _ => widget::page_title(text),
                }
            };
            let clock = format!("{:02}:{:02}:{:02}", (n / 3600) % 24, (n / 60) % 60, n % 60);
            let icons: Vec<Node<()>> = (0..10).map(|k| {
                let name = if k % 3 == 0 { format!("trindle-generic-{:02}", 1 + (n + k) % 12) } else { ["folder-symbolic", "edit-copy-symbolic", "list-add-symbolic"][k % 3].to_string() };
                widget::icon_image(name, [16.0, 24.0, 32.0, 48.0][(n / 5 + k) % 4], Tone::Plain)
            }).collect();
            let (pw, ph) = (40 + rng.below(160) as u32, 30 + rng.below(100) as u32);
            let tint = rng.below(200) as u8;
            let photo = Rc::new(image::DynamicImage::ImageRgba8(image::RgbaImage::from_fn(pw, ph, |x, y| image::Rgba([(x * 3) as u8, (y * 2) as u8, tint, 255]))));
            let mut rows = vec![widget::figure(clock), widget::row(icons).gap(6.0), widget::image_view(photo, "A photo", (pw as f32, ph as f32))];
            rows.extend((0..5).map(|k| line(k, rng)));
            widget::column(rows).padding(16.0).gap(8.0)
        };
        let mut r = Rng(seed() + 77);
        let mut ui = Ui::new(view(0, &mut r), Theme::new(i % 2 == 0, "#fbc711"));
        ui.see_through = see_through;
        run_ui(&mut rig, &mut ui, per_scene, |ui, n, rng, _| ui.set_view(view(n, rng)));
        let [masks, pictures] = shared.gpu().resets;
        println!("{}: atlases emptied {masks} times for letters and shapes, {pictures} times for pictures; frames {:?} (whole, in parts, age nought)", rig.name, rig.count);
        if side < 2048 && SERVICE.with_borrow(|s| s.is_none()) {
            assert!(masks + pictures > 0, "{}: the atlases never overflowed, so the scene proves nothing", rig.name);
        }
    }
    atlases(shared, u32::MAX);
}

fn windows_come_and_go(shared: &Rc<Shared>, windows: std::ops::Range<usize>) {
    for w in windows {
        let (size, scale) = SIZES[w % SIZES.len()];
        let mut rig = Rig::new(shared, &format!("window {w}"), size, scale, LAYOUTS[w % 3], seed() + 400 + w as u64);
        let (_, theme) = looks().swap_remove(w % 4);
        let mut ui = Ui::new(crate::gallery::gallery(0), theme);
        run_ui(&mut rig, &mut ui, 6, |ui, _, rng, now| use_it(ui, rng, now));
        drop(rig);
        assert_eq!(shared.gpu().error(), 0, "window {w}: a GL error when it closed");
    }
}

fn resizes(shared: &Rc<Shared>, per_size: usize) {
    let mut rig = Rig::new(shared, "resizing", (400, 300), 1.0, Layout::Linear, seed() + 500);
    let mut ui = Ui::new(crate::gallery::gallery(0), Theme::new(false, "#fbc711"));
    let steps = [((400, 300), 1.0), ((520, 380), 1.25), ((400, 300), 1.0), ((640, 460), 1.5), ((300, 220), 2.0), ((640, 460), 1.5), ((401, 301), 1.0), ((400, 300), 2.0)];
    for (round, (size, scale)) in steps.into_iter().enumerate() {
        rig.layout = LAYOUTS[round % 3];
        rig.resize(size, scale);
        run_ui(&mut rig, &mut ui, per_size, |ui, _, rng, now| use_it(ui, rng, now));
    }
}

const CELL: (i32, i32) = (150, 100);
const GRID: (usize, usize) = (6, 6);
const MARGIN: i32 = 120;

struct Spot {
    version: u32,
    pictures: Vec<(u64, Rc<Pixmap>)>,
}

struct Raw {
    spots: Vec<Spot>,
    sharp: Sharp,
    hinting: Hinting,
    shadows: Shadows,
    seed: u64,
    opaque: bool,
}

fn colour(rng: &mut Rng) -> crate::colour::Colour {
    let a = if rng.chance(600) { 1.0 } else { rng.range(0.1, 0.95) };
    crate::colour::Colour { r: rng.range(0.0, 1.0), g: rng.range(0.0, 1.0), b: rng.range(0.0, 1.0), a }
}

impl Raw {
    fn cell(i: usize) -> (i32, i32) {
        ((i % GRID.0) as i32 * CELL.0, (i / GRID.0) as i32 * CELL.1)
    }

    fn change(&mut self, i: usize, rng: &mut Rng) {
        let s = &mut self.spots[i];
        s.version += 1;
        s.pictures = (0..rng.below(3)).filter_map(|_| {
            let (w, h) = (4 + rng.below(100) as u32, 4 + rng.below(70) as u32);
            let mut pix = Pixmap::new(w, h)?;
            let k = rng.below(250) as u8;
            for (j, px) in pix.data_mut().chunks_exact_mut(4).enumerate() {
                let a = 90 + (j % 160) as u8;
                px.copy_from_slice(&[(k as u16 * a as u16 / 255) as u8, (j % 97) as u8 / 2, (a as u16 * (j as u16 % 7) / 8) as u8, a]);
            }
            Some((crate::paint::new_id(), Rc::new(pix)))
        }).collect();
    }

    fn record(&self, p: &mut Painter<'static>, size: (u32, u32)) {
        let whole = IntRect::from_xywh(0, 0, size.0, size.1).unwrap();
        p.mark(1_000_000, Some(whole));
        if self.opaque {
            p.fill(crate::widget::Rect { x: 0.0, y: 0.0, w: size.0 as f32, h: size.1 as f32 }, crate::colour::Colour { r: 0.93, g: 0.91, b: 0.85, a: 1.0 });
        }
        for (i, s) in self.spots.iter().enumerate() {
            let (cx, cy) = Raw::cell(i);
            let bounds = IntRect::from_xywh(cx - MARGIN, cy - MARGIN, (CELL.0 + 2 * MARGIN) as u32, (CELL.1 + 2 * MARGIN) as u32).and_then(|b| b.intersect(&whole));
            p.mark(i as u64, bounds);
            self.draw_spot(p, i, s, (cx, cy));
        }
    }

    fn spot_letters(&self, p: &mut Painter<'static>, rng: &mut Rng, (cx, cy): (i32, i32)) {
        let size = (rng.below(66) as f32 + 8.0) * if rng.chance(300) { 1.0 } else { 0.5 };
        let ink = colour(rng);
        let (mut x, y) = (cx + 4, cy + 10 + size as i32);
        let mut run = Vec::new();
        while x + size as i32 / 2 < cx + CELL.0 && run.len() < 14 {
            let key = MaskKey { font: fontdb::ID::dummy(), glyph: 3 + rng.below(250) as u16, size: size.to_bits(), weight: 400, lean: rng.chance(100) };
            if let Some(mask) = self.sharp.mask(NOTO, 0, "Noto Sans", &self.hinting, key) {
                run.push(crate::paint::Glyph { key, mask, x, y });
            }
            x += (size * 0.7) as i32 + 1;
        }
        p.glyphs(run, ink, self.sharp.table(ink));
    }

    fn draw_spot(&self, p: &mut Painter<'static>, i: usize, s: &Spot, (cx, cy): (i32, i32)) {
        use crate::widget::Rect;
        let mut rng = Rng((self.seed ^ (i as u64 * 0x9E37_79B9) ^ (u64::from(s.version) << 32)) | 1);
        let (fx, fy) = (cx as f32, cy as f32);
        let r = |rng: &mut Rng| {
            let (w, h) = (rng.range(6.0, 90.0), rng.range(6.0, 60.0));
            Rect { x: fx + rng.range(2.0, 100.0 - w / 2.0 + 20.0), y: fy + rng.range(2.0, 70.0), w, h }
        };
        for _ in 0..1 + rng.below(6) {
            match rng.below(11) {
                0 => p.fill(r(&mut rng), colour(&mut rng)),
                1 | 2 => {
                    let b = r(&mut rng);
                    let radii = [rng.range(0.0, 25.0), rng.range(0.0, 25.0), rng.range(0.0, 25.0), rng.range(0.0, 25.0)];
                    p.round(b, if rng.chance(500) { [radii[0]; 4] } else { radii }, colour(&mut rng));
                }
                3 => {
                    let ink = colour(&mut rng);
                    let width = if ink.a >= 1.0 { rng.range(0.4, 6.0) } else { rng.range(1.2, 6.0) };
                    p.line((fx + rng.range(0.0, 150.0), fy + rng.range(0.0, 100.0)), (fx + rng.range(0.0, 150.0), fy + rng.range(0.0, 100.0)), width, ink);
                }
                4 => spot_path(p, &mut rng, (fx, fy)),
                5 | 6 => self.spot_letters(p, &mut rng, (cx, cy)),
                7 => {
                    if let Some((id, pix)) = s.pictures.get(rng.below(s.pictures.len().max(1))) {
                        p.image(*id, pix.clone(), cx + rng.below(40) as i32, cy + rng.below(20) as i32);
                    }
                }
                8 => {
                    let b = r(&mut rng);
                    let falls = [crate::theme::Fall { y: rng.range(0.0, 5.0), blur: rng.range(1.0, 20.0), spread: 0.0, alpha: rng.range(0.1, 0.6) }];
                    if rng.chance(700) {
                        p.shadow(b, rng.range(0.0, 14.0), &falls, crate::colour::Colour { r: 0.0, g: 0.0, b: 0.0, a: 1.0 });
                    } else {
                        p.shadow_inside(b, rng.range(0.0, 14.0), falls[0], crate::colour::Colour { r: 0.0, g: 0.0, b: 0.0, a: 1.0 });
                    }
                }
                9 => spot_layer(p, &mut rng, (cx, cy)),
                _ => {
                    if let Some(at) = IntRect::from_xywh(cx + 10, cy + 10, 60, 40).filter(|_| i == 7) {
                        let k = s.version as f32;
                        p.pixels(at, move |c| {
                            if let Some(b) = tiny_skia::Rect::from_xywh(at.x() as f32 + (k % 30.0), at.y() as f32 + 5.0, 20.0, 20.0) {
                                c.fill_rect(b, &crate::colour::paint(crate::colour::Colour { r: 0.2, g: 0.7, b: 0.4, a: 0.6 }), tiny_skia::Transform::identity(), None);
                            }
                        });
                    }
                }
            }
        }
    }
}

fn spot_path(p: &mut Painter<'static>, rng: &mut Rng, (fx, fy): (f32, f32)) {
    let mut b = tiny_skia::PathBuilder::new();
    b.move_to(fx + rng.range(0.0, 150.0), fy + rng.range(0.0, 100.0));
    b.quad_to(fx + rng.range(0.0, 150.0), fy + rng.range(0.0, 100.0), fx + rng.range(0.0, 150.0), fy + rng.range(0.0, 100.0));
    b.line_to(fx + rng.range(0.0, 150.0), fy + rng.range(0.0, 100.0));
    b.close();
    if let Some(path) = b.finish() {
        let ink = colour(rng);
        if rng.chance(500) {
            p.fill_path(path, tiny_skia::FillRule::EvenOdd, ink);
        } else {
            let width = if ink.a >= 1.0 { rng.range(0.4, 5.0) } else { rng.range(1.2, 5.0) };
            p.stroke_path(path, tiny_skia::Stroke { width, ..Default::default() }, ink);
        }
    }
}

fn spot_layer(p: &mut Painter<'static>, rng: &mut Rng, (cx, cy): (i32, i32)) {
    use crate::widget::Rect;
    let (w, h) = (rng.range(20.0, 100.0) as u32, rng.range(20.0, 70.0) as u32);
    if let Some(at) = IntRect::from_xywh(cx + rng.below(40) as i32, cy + rng.below(25) as i32, w, h) {
        if rng.chance(500) {
            p.push_layer(at, rng.range(0.1, 1.0));
        } else {
            p.push_clip(at, rng.range(0.0, 20.0));
        }
        p.fill(Rect { x: -5.0, y: -5.0, w: w as f32 * 0.7, h: h as f32 + 9.0 }, colour(rng));
        p.round(Rect { x: w as f32 * 0.3, y: 3.0, w: w as f32 * 0.9, h: h as f32 * 0.6 }, [rng.range(0.0, 15.0); 4], colour(rng));
        p.pop();
    }
}

fn name(op: &Op) -> String {
    match op {
        Op::Fill { r, colour } => format!("fill {r:?} {colour:?}"),
        Op::Round { r, radii, colour } => format!("round {r:?} {radii:?} {colour:?}"),
        Op::Line { from, to, width, colour } => format!("line {from:?} {to:?} {width} {colour:?}"),
        Op::Path { how, colour, path } => format!("path {} {colour:?} {:?}", if matches!(how, crate::paint::How::Fill(_)) { "fill" } else { "stroke" }, path.bounds()),
        Op::Glyphs { run, colour, .. } => format!("glyphs {} {colour:?}", run.len()),
        Op::Image { id, x, y, pixmap } => format!("image {id} {x},{y} {}x{}", pixmap.width(), pixmap.height()),
        Op::Shadow { r, radius, .. } => format!("shadow {r:?} {radius}"),
        Op::Inside { r, radius, .. } => format!("inside {r:?} {radius}"),
        Op::Push { at, opacity, radius } => format!("push {at:?} {opacity} {radius}"),
        Op::Pop => "pop".into(),
        Op::Pixels { .. } => "pixels".into(),
        Op::Laid { x, y, .. } => format!("laid {x},{y}"),
    }
}

fn cpu_at(ops: Vec<Op>, size: (u32, u32), at: (u32, u32)) -> [u8; 4] {
    let mut want = Pixmap::new(size.0, size.1).unwrap();
    crate::paint::cpu::play(ops, &mut want.as_mut(), &Shadows::default());
    let i = ((at.1 * size.0 + at.0) * 4) as usize;
    want.data()[i..i + 4].try_into().unwrap()
}

fn gpu_at(shared: &Shared, ops: Vec<Op>, size: (u32, u32), at: (u32, u32), part: Option<[i32; 4]>) -> [u8; 4] {
    let mut gpu = shared.gpu_mut();
    let t = gpu.target(size.0, size.1).unwrap();
    match part {
        Some(part) => gpu.play_part(ops, &t, &Shadows::default(), part),
        None => gpu.play(ops, &t, &Shadows::default()),
    }
    let got = gpu.read(&t).unwrap();
    gpu.free(t);
    let i = ((at.1 * size.0 + at.0) * 4) as usize;
    got.data()[i..i + 4].try_into().unwrap()
}

fn bisect(at: (u32, u32), count: usize, mut after: impl FnMut(usize) -> (String, [u8; 4], [u8; 4])) {
    for k in 1..=count {
        let (last, a, g) = after(k);
        if a.iter().zip(&g).any(|(x, y)| x.abs_diff(*y) > 2) {
            println!("  op {k} of {count} parts them at {at:?}: processor {a:?}, GPU {g:?}\n  {last}");
            return;
        }
    }
    println!("  no op parts them at {at:?}, so what the atlases held matters");
}

fn raw_ops(shared: &Rc<Shared>, per_scene: usize) {
    for (i, (side, opaque)) in [(192u32, true), (256, false), (512, true), (2048, false)].into_iter().enumerate() {
        atlases(shared, side);
        let size = (GRID.0 as u32 * CELL.0 as u32, GRID.1 as u32 * CELL.1 as u32);
        let mut rig = Rig::new(shared, &format!("raw ops, atlases {side}{}", if opaque { "" } else { ", see-through" }), size, 1.0, LAYOUTS[i % 3], seed() + 600 + i as u64);
        let raw = Rc::new(RefCell::new(Raw { spots: (0..GRID.0 * GRID.1).map(|_| Spot { version: 0, pictures: Vec::new() }).collect(), sharp: Sharp::default(), hinting: Hinting::default(), shadows: Shadows::default(), seed: seed() + i as u64, opaque }));
        let mut rng = Rng(seed() + 9 + i as u64);
        let count = GRID.0 * GRID.1;
        for k in 0..count {
            raw.borrow_mut().change(k, &mut rng);
        }
        for n in 0..per_scene {
            let changing = match rng.below(20) {
                0 => 0,
                1..=11 => 1,
                12..=16 => 3,
                17 | 18 => 10,
                _ => count,
            };
            let extra = if n > 0 && rng.chance(100) { rng.below(4) } else { 0 };
            for _ in 0..changing + extra {
                raw.borrow_mut().change(rng.below(count), &mut rng);
            }
            let wrong = rig.step(|frame, drawing| {
                let raw = raw.borrow();
                let mut want = Pixmap::new(size.0, size.1).unwrap();
                let mut p = Painter::default();
                raw.record(&mut p, size);
                crate::paint::cpu::play(p.ops, &mut want.as_mut(), &raw.shadows);
                let mut p = Painter::default();
                raw.record(&mut p, size);
                let drawn = drawing.draw(frame, p, &raw.shadows);
                (want, drawn)
            });
            if let Some((what, at, part)) = wrong {
                println!("{what}");
                let raw = raw.borrow();
                let ops = |k: usize| { let mut p = Painter::default(); raw.record(&mut p, size); p.ops.truncate(k); p.ops };
                bisect(at, ops(usize::MAX).len(), |k| (name(&ops(k)[k - 1]), cpu_at(ops(k), size, at), gpu_at(shared, ops(k), size, at, part)));
                fail(what);
            }
        }
        let [masks, pictures] = shared.gpu().resets;
        println!("{}: atlases emptied {masks} times for letters and shapes, {pictures} times for pictures; frames {:?} (whole, in parts, age nought)", rig.name, rig.count);
    }
    atlases(shared, u32::MAX);
}

fn soak(per_scene: usize) {
    let shared = here();
    let started = Instant::now();
    println!("seed {}, {per_scene} frames a scene; buffers by the driver are laid out {:#x}", seed(), buffer_with(&shared, 64, 64, &[]).1.modifier);
    raw_ops(&shared, per_scene * 3);
    words_and_pictures(&shared, per_scene, false);
    words_and_pictures(&shared, per_scene / 2, true);
    gallery(&shared, per_scene / 2, false);
    gallery(&shared, per_scene / 4, true);
    scrolling(&shared, per_scene / 2);
    lists(&shared, per_scene);
    resizes(&shared, per_scene / 6);
    windows_come_and_go(&shared, 0..24);
    println!("soak took {:?}; in frames that passed, {} pixels were more than two levels apart (antialiased edges) and {} more than eight, the widest {} apart", started.elapsed(), ROUNDED.get(), STRAY.get(), WIDEST.get());
    assert_eq!(shared.gpu().error(), 0);
    FAILS.with_borrow(|f| assert!(f.is_empty(), "{} frames failed", f.len()));
}

#[test]
#[ignore = "needs a GPU and Mesa; TRINDLE_SOAK_SEED and TRINDLE_SOAK_FRAMES change it, 1500 frames a scene is a long run"]
fn a_soak_of_the_gpu_drawer_matches_the_processor_frame_after_frame() {
    soak(frames(80));
}

#[test]
#[ignore = "needs a GPU and Mesa; TRINDLE_SOAK_SEED and TRINDLE_SOAK_FRAMES change it"]
fn a_soak_through_the_drawing_service_matches_the_processor_frame_after_frame() {
    for side in [0u32, 1024] {
        let mut service = Running::start(&format!("soak{side}"), side, 0);
        let remote = Remote::at(&service.socket).expect("the service answers");
        SERVICE.set(Some(remote.clone()));
        println!("through the service, atlases {side}");
        soak(frames(40));
        SERVICE.set(None);
        assert!(!remote.gone(), "the service let a good program go");
        assert!(service.alive(), "the service ended");
        println!("the service held {:.0} MB at most", service.peak_mb());
    }
}

#[test]
#[ignore = "needs a GPU and Mesa"]
fn a_lost_context_fails_every_window_on_the_gpu_without_drawing() {
    let shared = here();
    let (_bo, d) = buffer_with(&shared, 64, 64, &[gbm::MOD_LINEAR]);
    let frame = Frame { dmabuf: Some(&d), modifier: Some(gbm::MOD_LINEAR), id: 1, size: (64, 64), scale: 1.0, age: 0, behind: &[] };
    let mut first = Drawing::new(shared.clone());
    assert!(matches!(first.draw(&frame, Painter::default(), &Shadows::default()), Drawn::Whole));
    shared.lost.set(true);
    for mut drawing in [first, Drawing::new(shared.clone())] {
        assert!(matches!(drawing.draw(&frame, Painter::default(), &Shadows::default()), Drawn::Failed));
    }
}
