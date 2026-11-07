//! A real service in a process of its own, spoken to over its socket by programs well and ill behaved.

use std::os::fd::AsFd;
use std::rc::Rc;
use std::time::{Duration, Instant};

use tiny_skia::{FillRule, IntRect, IntSize, PathBuilder, Pixmap, Stroke};

use super::rig::Running;
use crate::colour::Colour;
use crate::paint::link::{write_ask, Ask};
use crate::paint::remote::{Answer, Remote};
use crate::paint::wire::Encoder;
use crate::paint::{Glyph, How, Op};
use crate::sharp::{Mask, MaskKey};
use crate::sys::gbm::{self, Dmabuf};
use crate::theme::Fall;
use crate::widget::Rect;

const RED: Colour = Colour { r: 1.0, g: 0.0, b: 0.0, a: 1.0 };
const INK: Colour = Colour { r: 0.1, g: 0.6, b: 0.9, a: 0.7 };
const HELD_UP_MOST: Duration = Duration::from_secs(2);

fn rect(x: f32, y: f32, w: f32, h: f32) -> Rect {
    Rect { x, y, w, h }
}

struct Program {
    remote: Rc<Remote>,
    encoder: Encoder,
    id: u64,
    dmabuf: Dmabuf,
    size: (u32, u32),
}

impl Program {
    fn new(service: &Running, size: (u32, u32)) -> Program {
        let remote = Remote::at(&service.socket).expect("the service answers");
        let id = 1;
        let dmabuf = remote.buffer(id, size, &[gbm::MOD_LINEAR]).expect("a buffer");
        Program { remote, encoder: Encoder::default(), id, dmabuf, size }
    }

    fn draw(&mut self, ops: &[Op]) -> Answer {
        let frame = self.encoder.frame(ops).expect("a frame that can be sent");
        let whole = [0, 0, self.size.0 as i32, self.size.1 as i32];
        let seq = self.remote.send_raw(self.id, whole, &frame.control, &frame.pixels);
        self.remote.block_for(seq)
    }

    fn pixel(&self, x: usize, y: usize) -> [u8; 4] {
        let bytes = gbm::read_linear(&self.dmabuf).expect("the buffer read");
        let p = &self.dmabuf.planes[0];
        let at = p.offset as usize + y * p.stride as usize + x * 4;
        bytes[at..at + 4].try_into().unwrap()
    }
}

struct Victim(Program);

impl Victim {
    fn new(service: &Running) -> Victim {
        Victim(Program::new(service, (200, 100)))
    }

    fn check(&mut self) -> Duration {
        let began = Instant::now();
        let ops = [Op::Fill { r: rect(0.0, 0.0, 200.0, 100.0), colour: RED }, Op::Round { r: rect(20.0, 20.0, 40.0, 40.0), radii: [8.0; 4], colour: Colour { r: 0.0, g: 1.0, b: 0.0, a: 1.0 } }];
        assert_eq!(self.0.draw(&ops), Answer::Done);
        assert_eq!(self.0.pixel(100, 50), [0, 0, 255, 255], "another program's frame showed through");
        assert_eq!(self.0.pixel(40, 40), [0, 255, 0, 255], "another program's frame showed through");
        began.elapsed()
    }
}

#[test]
#[ignore = "needs a GPU and Mesa"]
fn a_frame_sent_small_or_in_a_file_is_drawn_and_a_new_buffer_holds_nothing_of_anyone_elses() {
    let service = Running::start("basic", 0, 0);
    let mut a = Program::new(&service, (64, 64));
    assert_eq!(a.pixel(10, 10), [0; 4], "a new buffer is clear");
    assert_eq!(a.draw(&[Op::Fill { r: rect(0.0, 0.0, 64.0, 64.0), colour: RED }]), Answer::Done);
    assert_eq!(a.pixel(10, 10), [0, 0, 255, 255]);
    let mut picture = Pixmap::new(300, 300).unwrap();
    picture.fill(tiny_skia::Color::from_rgba8(0, 0, 255, 255));
    let ops = [Op::Image { id: 9, pixmap: Rc::new(picture), x: 0, y: 0 }];
    assert_eq!(a.draw(&ops), Answer::Done, "more than a message holds");
    assert_eq!(a.pixel(10, 10), [255, 0, 0, 255]);
    assert_eq!(a.draw(&ops), Answer::Done, "named this time, not sent");
    assert_eq!(a.pixel(20, 20), [255, 0, 0, 255]);
    a.remote.free(a.id);
    drop(a);
    for _ in 0..4 {
        let b = Program::new(&service, (64, 64));
        assert_eq!(b.pixel(10, 10), [0; 4], "a new buffer shows nothing of another program's");
    }
}

#[test]
#[ignore = "needs a GPU and Mesa"]
fn no_program_can_name_another_programs_buffer_picture_or_letter() {
    let mut service = Running::start("names", 0, 0);
    let mut a = Victim::new(&service);
    let mut picture = Pixmap::new(8, 8).unwrap();
    picture.fill(tiny_skia::Color::from_rgba8(0, 0, 255, 255));
    let picture = Rc::new(picture);
    let image = [Op::Image { id: 42, pixmap: picture.clone(), x: 0, y: 0 }];
    assert_eq!(a.0.draw(&image), Answer::Done);

    let other = Remote::at(&service.socket).unwrap();
    let frame = Encoder::default().frame(&[Op::Fill { r: rect(0.0, 0.0, 9.0, 9.0), colour: RED }]).unwrap();
    assert_eq!(other.block_for(other.send_raw(a.0.id, [0, 0, 9, 9], &frame.control, &frame.pixels)), Answer::Gone);
    let other = Remote::at(&service.socket).unwrap();
    other.free(a.0.id);
    std::thread::sleep(Duration::from_millis(100));
    assert!(other.ended(), "freeing a buffer of another's");

    let mut thief = Program::new(&service, (8, 8));
    let mut knows = Encoder::default();
    knows.frame(&image).unwrap();
    let named = knows.frame(&image).unwrap();
    assert!(named.pixels.is_empty());
    assert_eq!(thief.remote.block_for(thief.remote.send_raw(thief.id, [0, 0, 8, 8], &named.control, &[])), Answer::Gone);
    let _ = thief.draw(&[]);

    let table = Rc::new([255; 256]);
    let mask = Rc::new(Mask::from_parts(0, 0, 2, 2, vec![255; 4].into_boxed_slice()));
    let key = MaskKey { font: fontdb::ID::dummy(), glyph: 1, size: 1, weight: 1, lean: false };
    let letters = [Op::Glyphs { run: vec![Glyph { key, mask, x: 1, y: 1 }], colour: RED, table }];
    let mut knows = Encoder::default();
    knows.frame(&letters).unwrap();
    let named = knows.frame(&letters).unwrap();
    let thief = Program::new(&service, (8, 8));
    assert_eq!(thief.remote.block_for(thief.remote.send_raw(thief.id, [0, 0, 8, 8], &named.control, &[])), Answer::Gone);

    a.check();
    assert!(service.alive());
}

#[test]
#[ignore = "needs a GPU and Mesa"]
fn the_same_letter_from_two_programs_draws_right_for_each() {
    let service = Running::start("letters", 0, 0);
    let table: Rc<[u8; 256]> = Rc::new(std::array::from_fn(|i| i as u8));
    let run = |cover: u8| {
        let mask = Rc::new(Mask::from_parts(0, 0, 4, 4, vec![cover; 16].into_boxed_slice()));
        let key = MaskKey { font: fontdb::ID::dummy(), glyph: 1, size: 1, weight: 1, lean: false };
        vec![Glyph { key, mask, x: 2, y: 2 }]
    };
    let (mut a, mut b) = (Program::new(&service, (16, 16)), Program::new(&service, (16, 16)));
    let draw = |p: &mut Program, cover: u8, colour: Colour| {
        assert_eq!(p.draw(&[Op::Glyphs { run: run(cover), colour, table: table.clone() }]), Answer::Done);
    };
    draw(&mut a, 255, RED);
    draw(&mut b, 255, Colour { r: 0.0, g: 0.0, b: 1.0, a: 1.0 });
    draw(&mut a, 128, RED);
    assert_eq!(a.pixel(3, 3), [0, 0, 128, 128], "its own second letter");
    assert_eq!(b.pixel(3, 3), [255, 0, 0, 255], "its own colour");
}

#[test]
#[ignore = "needs a GPU and Mesa"]
fn hostile_frames_are_drawn_within_a_limit_of_memory_and_hold_no_one_up() {
    let mut service = Running::start("hostile", 0, 900);
    let mut victim = Victim::new(&service);
    victim.check();
    let size = (1920u32, 1080u32);
    let mut worst = Duration::ZERO;
    for (name, ops) in hostile_frames() {
        let mut bad = Program::new(&service, size);
        let frame = bad.encoder.frame(&ops);
        let Ok(frame) = frame else {
            println!("{name}: refused by the encoder");
            continue;
        };
        let began = Instant::now();
        let seq = bad.remote.send_raw(bad.id, [0, 0, size.0 as i32, size.1 as i32], &frame.control, &frame.pixels);
        let mut most = Duration::ZERO;
        let answer = loop {
            std::thread::sleep(Duration::from_millis(2));
            most = most.max(victim.check());
            if let Some(a) = bad.remote.answered_yet(seq) {
                break a;
            }
            if bad.remote.ended() {
                break Answer::Gone;
            }
            if began.elapsed() > Duration::from_secs(30) {
                panic!("{name}: not answered in 30 seconds");
            }
        };
        println!("{name}: {answer:?} after {:?}, the other program held up at most {most:?}, the service at {:.0} MB", began.elapsed(), service.peak_mb());
        worst = worst.max(most);
        assert!(service.alive(), "{name}: the service ended");
        assert!(most < HELD_UP_MOST, "{name}: the other program was held up {most:?}");
    }
    let budget = |from: u64| -> Vec<Op<'static>> { (from..from + 2).map(|i| Op::Image { id: i, pixmap: Rc::new(Pixmap::new(3000, 1300).unwrap()), x: 0, y: 0 }).collect() };
    let bad = Program::new(&service, size);
    let whole = [0, 0, size.0 as i32, size.1 as i32];
    for from in [1, 100] {
        let first = Encoder::default().frame(&budget(from)).unwrap();
        assert_eq!(bad.remote.block_for(bad.remote.send_raw(bad.id, whole, &first.control, &first.pixels)), Answer::Done);
    }
    let second = Encoder::default().frame(&budget(200)).unwrap();
    assert_eq!(bad.remote.block_for(bad.remote.send_raw(bad.id, whole, &second.control, &second.pixels)), Answer::Gone, "over its budget of pictures");
    victim.check();
    println!("held up at most {worst:?}, peak {:.0} MB", service.peak_mb());
    assert!(service.peak_mb() < 900.0, "{:.0} MB", service.peak_mb());
}

type Hostile = (&'static str, Vec<Op<'static>>);

fn hostile_frames() -> Vec<Hostile> {
    let mut all = Vec::new();
    hostile_boxes_and_layers(&mut all);
    hostile_paths(&mut all);
    hostile_shadows(&mut all);
    hostile_pictures(&mut all);
    hostile_letters(&mut all);
    all.push(("as many fills as can be", (0..110_000).map(|i| Op::Fill { r: rect((i % 1900) as f32, (i % 1000) as f32, 2000.0, 2000.0), colour: INK }).collect()));
    all
}

fn hostile_boxes_and_layers(all: &mut Vec<Hostile>) {
    let vast = 1_000_000.0;
    all.push(("round boxes a million across, cut by every edge", (0..2000).map(|i| {
        let v = i as f32;
        Op::Round { r: rect(-500_000.0 + v, -400_000.0 - v, vast, vast), radii: [20.0 + v / 100.0; 4], colour: INK }
    }).collect()));
    all.push(("round boxes wide and thin", (0..2000).map(|i| Op::Round { r: rect(-1000.0 - i as f32, i as f32 / 4.0, vast, 3.0), radii: [1.5; 4], colour: INK }).collect()));
    all.push(("round layers cut by the edge", (0..200).flat_map(|i| {
        let at = IntRect::from_xywh(-1000 - i, -700, 2500 + i as u32, 2500).unwrap();
        [Op::Push { at, opacity: 0.5, radius: 400.0 }, Op::Fill { r: rect(0.0, 0.0, 4000.0, 3000.0), colour: INK }, Op::Pop]
    }).collect()));
    all.push(("layers of a new size each, as many as can be", (0..40000).flat_map(|i| {
        let at = IntRect::from_xywh(0, 0, 1000 + (i % 3000) as u32, 1000 + (i / 3000) as u32).unwrap();
        [Op::Push { at, opacity: 0.5, radius: 0.0 }, Op::Pop]
    }).collect()));
    all.push(("layers nested to the limit", {
        let mut ops: Vec<Op> = (0..30).map(|i| Op::Push { at: IntRect::from_xywh(i, i, 900, 500).unwrap(), opacity: 0.9, radius: 3.0 }).collect();
        ops.push(Op::Fill { r: rect(0.0, 0.0, 9000.0, 9000.0), colour: INK });
        ops
    }));
}

fn hostile_paths(all: &mut Vec<Hostile>) {
    let long_path = |seed: f32, far: f32| {
        let mut b = PathBuilder::new();
        b.move_to(seed, seed);
        for i in 0..16000 {
            let (x, y) = ((i as f32 * 37.0 + seed) % far - far / 2.0, (i as f32 * 91.0) % far - far / 2.0);
            b.line_to(x, y);
        }
        b.finish().unwrap()
    };
    let strokes = |width: f32, far: f32, n: usize| (0..n).map(move |k| k as f32).map(move |k| (width, far, k));
    all.push(("long paths, far and wide, stroked thick", strokes(50000.0, 2.0e6, 20).map(|(width, far, k)| Op::Path {
        path: long_path(k, far), how: How::Stroke(Stroke { width, line_join: tiny_skia::LineJoin::Round, line_cap: tiny_skia::LineCap::Round, ..Stroke::default() }), colour: INK,
    }).collect()));
    all.push(("long paths filled", (0..20).map(|k| Op::Path { path: long_path(k as f32, 3000.0), how: How::Fill(FillRule::EvenOdd), colour: INK }).collect()));
    all.push(("as many paths as the window holds, each new", (0..4000).map(|k| {
        let mut b = PathBuilder::new();
        b.move_to(-10.0, -10.0);
        b.line_to(3000.0 + k as f32, -10.0);
        b.line_to(3000.0, 2000.0 + k as f32);
        b.close();
        Op::Path { path: b.finish().unwrap(), how: How::Fill(FillRule::Winding), colour: INK }
    }).collect()));
}

fn hostile_shadows(all: &mut Vec<Hostile>) {
    let fall = |k: f32| Fall { y: k % 7.0, blur: 300.0 + k, spread: 2.0, alpha: 0.5 };
    all.push(("shadows at the most a mask may be, each new", (0..400).map(|k| {
        let k = k as f32;
        Op::Shadow { r: rect(k, k, 1000.0 + k, 700.0), radius: 200.0, falls: vec![fall(k), fall(k + 1.0), fall(k + 2.0)], colour: Colour { r: 0.0, g: 0.0, b: 0.0, a: 1.0 } }
    }).collect()));
    all.push(("shadows inside, each new", (0..400).map(|k| {
        let k = k as f32;
        Op::Inside { r: rect(k, k, 800.0 + k, 800.0), radius: 300.0, fall: Fall { blur: 100.0 + k / 10.0, ..fall(k) }, colour: INK }
    }).collect()));
}

fn hostile_pictures(all: &mut Vec<Hostile>) {
    let pixels = |w: u32, h: u32| Pixmap::new(w, h).unwrap();
    all.push(("pictures laid wide then tall", vec![
        Op::Laid { x: -5000, y: 10, pixmap: pixels(16384, 256) },
        Op::Laid { x: 10, y: -5000, pixmap: pixels(256, 16384) },
        Op::Laid { x: 1 << 20, y: 1 << 20, pixmap: pixels(512, 512) },
    ]));
    all.push(("pictures too big for an atlas, up to the budget", (0..2u64).map(|i| Op::Image { id: i + 1, pixmap: Rc::new(pixels(3000, 1300)), x: i as i32, y: 0 }).collect()));
}

fn hostile_letters(all: &mut Vec<Hostile>) {
    let table = Rc::new([255; 256]);
    let letters = |n: u32, side: usize| -> Vec<Glyph> {
        (0..n).map(|i| {
            let cover = vec![i as u8; side * side].into_boxed_slice();
            let key = MaskKey { font: fontdb::ID::dummy(), glyph: 1, size: i, weight: 1, lean: false };
            Glyph { key, mask: Rc::new(Mask::from_parts(i as i32, 0, side, side, cover)), x: (i % 1900) as i32, y: (i / 100) as i32 }
        }).collect()
    };
    all.push(("letters to fill every atlas", vec![Op::Glyphs { run: letters(1000, 64), colour: INK, table: table.clone() }]));
    let many = letters(16384, 3);
    all.push(("runs of letters, as many as can be", (0..16).map(|_| Op::Glyphs {
        run: many.iter().map(|g| Glyph { key: g.key, mask: g.mask.clone(), x: g.x, y: g.y }).collect(), colour: INK, table: table.clone(),
    }).collect()));
    all.push(("as many tables as are kept", (0..1000).map(|i| {
        let t: [u8; 256] = std::array::from_fn(|k| (k + i) as u8);
        Op::Glyphs { run: vec![Glyph { key: many[0].key, mask: many[0].mask.clone(), x: 0, y: 0 }], colour: INK, table: Rc::new(t) }
    }).collect()));
}

fn messages_that_are_none(service: &Running, victim: &mut Victim, next: &mut impl FnMut() -> u64) {
    for n in 0..3000 {
        let Some(p) = Remote::at(&service.socket) else {
            std::thread::sleep(Duration::from_millis(5));
            continue;
        };
        let len = next() as usize % 120;
        let mut bytes: Vec<u8> = (0..len).map(|_| next() as u8).collect();
        if n % 2 == 0 && !bytes.is_empty() {
            bytes[0] = 1 + next() as u8 % 4;
        }
        p.send_bytes(&bytes);
        p.send_bytes(&write_ask(&Ask::Sleep));
        if n % 500 == 0 {
            victim.check();
        }
    }
}

fn connect(service: &Running) -> Rc<Remote> {
    let began = Instant::now();
    loop {
        if let Some(p) = Remote::at(&service.socket) {
            return p;
        }
        assert!(began.elapsed() < Duration::from_secs(10), "the service takes no more programs");
        std::thread::sleep(Duration::from_millis(5));
    }
}

fn fuzz_scene() -> Vec<Op<'static>> {
    let picture = {
        let mut p = Pixmap::new(40, 30).unwrap();
        p.fill(tiny_skia::Color::from_rgba8(10, 20, 30, 200));
        Rc::new(p)
    };
    let mask = Rc::new(Mask::from_parts(1, 2, 5, 7, vec![200; 35].into_boxed_slice()));
    let key = MaskKey { font: fontdb::ID::dummy(), glyph: 1, size: 1, weight: 1, lean: false };
    let table = Rc::new([200; 256]);
    let path = {
        let mut b = PathBuilder::new();
        b.move_to(5.0, 5.0);
        b.cubic_to(90.0, 10.0, 30.0, 70.0, 10.0, 60.0);
        b.close();
        b.finish().unwrap()
    };
    let falls = vec![Fall { y: 2.0, blur: 8.0, spread: 1.0, alpha: 0.4 }];
    vec![
        Op::Fill { r: rect(3.0, 4.0, 90.0, 40.0), colour: INK },
        Op::Round { r: rect(-30.0, 6.0, 90.0, 50.0), radii: [9.0, 0.0, 20.0, 3.0], colour: INK },
        Op::Line { from: (5.0, 7.0), to: (90.0, 60.0), width: 2.5, colour: INK },
        Op::Path { path: path.clone(), how: How::Stroke(Stroke { width: 3.0, ..Stroke::default() }), colour: INK },
        Op::Path { path, how: How::Fill(FillRule::EvenOdd), colour: INK },
        Op::Glyphs { run: vec![Glyph { key, mask, x: 4, y: 40 }], colour: INK, table },
        Op::Image { id: 7, pixmap: picture, x: 40, y: 50 },
        Op::Shadow { r: rect(20.0, 20.0, 50.0, 40.0), radius: 8.0, falls, colour: Colour { a: 1.0, ..INK } },
        Op::Inside { r: rect(20.0, 20.0, 50.0, 40.0), radius: 8.0, fall: Fall { y: 2.0, blur: 4.0, spread: 0.0, alpha: 0.3 }, colour: INK },
        Op::Push { at: IntRect::from_xywh(8, 9, 70, 60).unwrap(), opacity: 0.6, radius: 7.0 },
        Op::Fill { r: rect(2.0, 3.0, 30.0, 20.0), colour: INK },
        Op::Pop,
        Op::Laid { x: -3, y: 60, pixmap: Pixmap::from_vec(vec![90; 40 * 10 * 4], IntSize::from_wh(40, 10).unwrap()).unwrap() },
    ]
}

fn spoil(control: &mut Vec<u8>, pixels: &mut Vec<u8>, next: &mut impl FnMut() -> u64) {
    for _ in 0..1 + next() as usize % 4 {
        let target = if next() % 8 == 0 { &mut *pixels } else { &mut *control };
        let at = next() as usize % target.len();
        match next() % 5 {
            0 => target[at] = next() as u8,
            1 => target[at] ^= 1 << (next() % 8),
            2 => target.truncate(at.max(1)),
            3 => target.insert(at, next() as u8),
            _ => {
                let end = (at + 4).min(target.len());
                target[at..end].iter_mut().for_each(|b| *b = 0xff);
            }
        }
    }
}

#[test]
#[ignore = "needs a GPU and Mesa"]
fn the_socket_handler_survives_anything_a_program_sends() {
    let mut service = Running::start("fuzz", 0, 900);
    let mut victim = Victim::new(&service);
    let mut x = 0x2545_f491_4f6c_dd1du64;
    let mut next = move || {
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        x
    };
    messages_that_are_none(&service, &mut victim, &mut next);
    victim.check();
    for (w, h) in [(0, 1), (1, 0), (u32::MAX, 1), (1, u32::MAX), (16385, 1), (16384, 16384)] {
        let p = connect(&service);
        assert!(p.buffer(7, (w, h), &[]).is_none(), "{w} x {h}");
    }
    let p = connect(&service);
    assert!(p.buffer(7, (8, 8), &[0x0123_4567_89ab_cdef]).is_none(), "a layout no one has");
    assert!(p.buffer(8, (8, 8), &[gbm::MOD_LINEAR]).is_some());
    assert!(p.buffer(8, (8, 8), &[gbm::MOD_LINEAR]).is_none(), "the same number twice");
    let unsealed = crate::common::shm::Memory::new(c"not-sealed", 64).unwrap();
    let p = Program::new(&service, (8, 8));
    let not_a_memory_file = std::fs::File::open("/proc/self/stat").unwrap();
    assert_eq!(p.remote.block_for(p.remote.send_with_file(p.id, 64, unsealed.fd())), Answer::Gone);
    let p = Program::new(&service, (8, 8));
    assert_eq!(p.remote.block_for(p.remote.send_with_file(p.id, 64, not_a_memory_file.as_fd())), Answer::Gone);
    let p = Program::new(&service, (8, 8));
    let short = crate::sys::sock::sealed(c"short", &[&[0; 8]]).unwrap();
    assert_eq!(p.remote.block_for(p.remote.send_with_file(p.id, 64, short.as_fd())), Answer::Gone);
    victim.check();
    let scene = fuzz_scene();
    let frame = Encoder::default().frame(&scene).unwrap();
    let iterations: usize = std::env::var("TRINDLE_FUZZ_FRAMES").ok().and_then(|v| v.parse().ok()).unwrap_or(6000);
    let (mut p, mut drawn, mut turned_away) = (Program::new(&service, (120, 90)), 0, 0);
    for n in 0..iterations {
        let (mut control, mut pixels) = (frame.control.clone(), frame.pixels.clone());
        spoil(&mut control, &mut pixels, &mut next);
        if control.len() > 7 {
            control[6] |= 1;
        }
        match p.remote.block_for(p.remote.send_raw(p.id, [0, 0, 120, 90], &control, &pixels)) {
            Answer::Gone => {
                turned_away += 1;
                p = Program::new(&service, (120, 90));
            }
            _ => drawn += 1,
        }
        if n % 200 == 0 {
            victim.check();
            assert!(service.alive(), "after {n}: the service ended");
        }
    }
    println!("{drawn} frames drawn, {turned_away} turned away; the service held {:.0} MB at most", service.peak_mb());
    assert!(drawn > 0 && turned_away > 0);
    victim.check();
    assert!(service.alive());
    assert!(service.peak_mb() < 900.0, "{:.0} MB", service.peak_mb());
}

#[test]
#[ignore = "needs a GPU and Mesa"]
fn a_service_killed_is_heard_at_once_and_no_one_waits() {
    let mut service = Running::start("killed", 0, 0);
    let p = Program::new(&service, (64, 64));
    let frame = Encoder::default().frame(&[Op::Fill { r: rect(0.0, 0.0, 64.0, 64.0), colour: RED }]).unwrap();
    assert_eq!(p.remote.block_for(p.remote.send_raw(p.id, [0, 0, 64, 64], &frame.control, &frame.pixels)), Answer::Done);
    service.child.kill().unwrap();
    service.child.wait().unwrap();
    let began = Instant::now();
    let seq = p.remote.send_raw(p.id, [0, 0, 64, 64], &frame.control, &frame.pixels);
    assert_eq!(p.remote.block_for(seq), Answer::Gone);
    assert!(began.elapsed() < Duration::from_millis(100), "{:?}", began.elapsed());
    let began = Instant::now();
    assert!(p.remote.buffer(9, (8, 8), &[]).is_none());
    assert!(began.elapsed() < Duration::from_millis(100), "{:?}", began.elapsed());
    assert!(Remote::at(&service.socket).is_none(), "nothing answers at the old socket");
    assert_eq!(p.pixel(10, 10), [0, 0, 255, 255]);
}

#[test]
#[ignore = "needs a GPU and Mesa"]
fn a_second_service_leaves_the_first_serving() {
    let service = Running::start("twice", 0, 0);
    let dir = service.socket.parent().unwrap();
    assert_eq!(super::serve(&dir.join("s"), &dir.join("l"), super::Sizes::default()), std::process::ExitCode::SUCCESS);
    let mut v = Victim::new(&service);
    v.check();
}

fn gpu_memory(pid: u32) -> u64 {
    let mut clients = std::collections::HashMap::new();
    for fd in std::fs::read_dir(format!("/proc/{pid}/fdinfo")).unwrap().flatten() {
        let Ok(text) = std::fs::read_to_string(fd.path()) else { continue };
        let Some(id) = text.lines().find_map(|l| l.strip_prefix("drm-client-id:")).map(str::to_string) else { continue };
        let kb = |l: &str| l.split_whitespace().nth(1).and_then(|v| v.parse::<u64>().ok());
        clients.insert(id, text.lines().filter(|l| l.starts_with("drm-total-") && !l.starts_with("drm-total-cpu")).filter_map(kb).sum::<u64>());
    }
    clients.values().sum::<u64>() / 1024
}

#[test]
#[ignore = "needs a GPU and Mesa"]
fn a_program_that_asks_for_a_buffer_and_goes_at_once_leaves_no_memory_behind() {
    use crate::sys::sock;
    use std::os::fd::AsRawFd;
    let service = Running::start("leave", 0, 0);
    let pid = service.child.id();
    let limits = std::fs::read_to_string(format!("/proc/{pid}/limits")).unwrap();
    let open: Vec<&str> = limits.lines().find(|l| l.starts_with("Max open files")).unwrap().split_whitespace().skip(3).take(2).collect();
    assert_eq!(open[0], open[1], "soft and hard limits of open files");
    let mut v = Victim::new(&service);
    v.check();
    let before = gpu_memory(pid);
    let ask = write_ask(&Ask::Buffer { id: 1, w: 2048, h: 2048, layouts: vec![gbm::MOD_LINEAR] });
    for _ in 0..64 {
        let fd = sock::connect(&service.socket).unwrap();
        sock::send(fd.as_raw_fd(), &ask, &[]).unwrap();
        drop(fd);
        v.check();
    }
    v.check();
    let after = gpu_memory(pid);
    assert!(after < before + 128, "{before} MB before, {after} MB after");
}
