//! The wire's tests: frames that go and come back, and every way a frame can be wrong.

use tiny_skia::{PathBuilder, PixmapMut, Transform};

use super::super::cpu;
use super::*;
use crate::shadow::Shadows;
use crate::sharp::{Hinting, Sharp};

const INK: Colour = Colour { r: 0.2, g: 0.5, b: 0.9, a: 0.7 };
const SIDE: u32 = 96;

fn r(x: f32, y: f32, w: f32, h: f32) -> Rect {
    Rect { x, y, w, h }
}

fn at(x: i32, y: i32, w: u32, h: u32) -> IntRect {
    IntRect::from_xywh(x, y, w, h).unwrap()
}

fn triangle() -> tiny_skia::Path {
    let mut b = PathBuilder::new();
    b.move_to(10.3, 80.0);
    b.line_to(70.6, 12.2);
    b.quad_to(90.0, 30.0, 88.0, 70.5);
    b.cubic_to(70.0, 90.0, 40.0, 95.0, 20.0, 85.0);
    b.close();
    b.move_to(30.0, 30.0);
    b.line_to(40.0, 30.0);
    b.line_to(40.0, 40.0);
    b.close();
    b.finish().unwrap()
}

fn picture(w: u32, h: u32, alpha: u8) -> Pixmap {
    let mut p = Pixmap::new(w, h).unwrap();
    p.fill(tiny_skia::Color::from_rgba8(20, 120, 60, alpha));
    p
}

fn letters() -> (Vec<Glyph>, Rc<[u8; 256]>) {
    let sharp = Sharp::default();
    let data = include_bytes!("../../../share/fonts/NotoSans-Regular.ttf");
    let hinting = Hinting::default();
    let run = (36..44u16).enumerate().filter_map(|(n, glyph)| {
        let key = MaskKey { font: fontdb::ID::dummy(), glyph, size: 18.0f32.to_bits(), weight: 400, lean: false };
        Some(Glyph { key, mask: sharp.mask(data, 0, "Noto Sans", &hinting, key)?, x: 4 + 11 * n as i32, y: 50 })
    }).collect();
    (run, sharp.table(INK))
}

fn scene(shared: &(Vec<Glyph>, Rc<[u8; 256]>), image: &Rc<Pixmap>) -> Vec<Op<'static>> {
    let run = shared.0.iter().map(|g| Glyph { key: g.key, mask: g.mask.clone(), x: g.x, y: g.y }).collect();
    let falls = vec![Fall { y: 3.0, blur: 8.0, spread: 0.0, alpha: 0.4 }, Fall { y: 1.0, blur: 2.0, spread: 1.0, alpha: 0.2 }];
    vec![
        Op::Fill { r: r(3.3, 4.6, 70.2, 30.5), colour: INK },
        Op::Round { r: r(6.4, 8.2, 80.0, 50.7), radii: [9.5, 0.0, 20.0, 3.0], colour: INK },
        Op::Line { from: (5.5, 7.0), to: (90.2, 60.3), width: 2.5, colour: INK },
        Op::Path { path: triangle(), how: How::Fill(FillRule::EvenOdd), colour: INK },
        Op::Path { path: triangle(), how: How::Fill(FillRule::Winding), colour: INK },
        Op::Path {
            path: triangle(),
            how: How::Stroke(Stroke { width: 3.2, miter_limit: 2.5, line_cap: LineCap::Round, line_join: LineJoin::Bevel, dash: None }),
            colour: INK,
        },
        Op::Glyphs { run, colour: INK, table: shared.1.clone() },
        Op::Image { id: 7, pixmap: image.clone(), x: 40, y: 50 },
        Op::Shadow { r: r(20.0, 20.0, 50.0, 40.0), radius: 8.0, falls, colour: Colour { a: 1.0, ..INK } },
        Op::Inside { r: r(20.0, 20.0, 50.0, 40.0), radius: 8.0, fall: Fall { y: 2.0, blur: 4.0, spread: 0.0, alpha: 0.3 }, colour: INK },
        Op::Push { at: at(8, 9, 70, 60), opacity: 0.6, radius: 0.0 },
        Op::Push { at: at(5, 5, 40, 30), opacity: 1.0, radius: 12.0 },
        Op::Fill { r: r(2.5, 3.5, 30.0, 20.0), colour: INK },
        Op::Pop,
        Op::Line { from: (0.0, 0.0), to: (60.0, 50.0), width: 1.5, colour: INK },
        Op::Pop,
        Op::Laid { x: -3, y: 60, pixmap: picture(20, 10, 160) },
        Op::Push { at: at(-10, 70, 30, 20), opacity: 0.5, radius: 0.0 },
        Op::Fill { r: r(0.0, 0.0, 50.0, 50.0), colour: INK },
    ]
}

fn same_op(a: &Op, b: &Op) -> bool {
    match (a, b) {
        (Op::Fill { r: a, colour: c }, Op::Fill { r: b, colour: d }) => a == b && c == d,
        (Op::Round { r: a, radii: e, colour: c }, Op::Round { r: b, radii: f, colour: d }) => a == b && e == f && c == d,
        (Op::Line { from: a, to: e, width: g, colour: c }, Op::Line { from: b, to: f, width: h, colour: d }) => a == b && e == f && g == h && c == d,
        (Op::Path { path: a, how: x, colour: c }, Op::Path { path: b, how: y, colour: d }) => {
            a.verbs() == b.verbs() && a.points() == b.points() && c == d && match (x, y) {
                (How::Fill(x), How::Fill(y)) => x == y,
                (How::Stroke(x), How::Stroke(y)) => (x.width, x.miter_limit, x.line_cap, x.line_join) == (y.width, y.miter_limit, y.line_cap, y.line_join) && x.dash.is_none() && y.dash.is_none(),
                _ => false,
            }
        }
        (Op::Glyphs { run: a, colour: c, table: s }, Op::Glyphs { run: b, colour: d, table: t }) => {
            a.len() == b.len() && c == d && s == t
                && a.iter().zip(b).all(|(a, b)| (a.x, a.y) == (b.x, b.y) && a.mask.parts() == b.mask.parts())
        }
        (Op::Image { id: i, pixmap: a, x: p, y: q }, Op::Image { id: j, pixmap: b, x: s, y: t }) => {
            let _ = (i, j);
            a.data() == b.data() && (a.width(), a.height()) == (b.width(), b.height()) && (p, q) == (s, t)
        }
        (Op::Shadow { r: a, radius: e, falls: g, colour: c }, Op::Shadow { r: b, radius: f, falls: h, colour: d }) => a == b && e == f && g == h && c == d,
        (Op::Inside { r: a, radius: e, fall: g, colour: c }, Op::Inside { r: b, radius: f, fall: h, colour: d }) => a == b && e == f && g == h && c == d,
        (Op::Push { at: a, opacity: e, radius: g }, Op::Push { at: b, opacity: f, radius: h }) => a == b && e == f && g == h,
        (Op::Pop, Op::Pop) => true,
        (Op::Laid { x: p, y: q, pixmap: a }, Op::Laid { x: s, y: t, pixmap: b }) => (p, q) == (s, t) && a.data() == b.data() && a.width() == b.width(),
        _ => false,
    }
}

fn ground() -> Pixmap {
    let mut p = Pixmap::new(SIDE, SIDE).unwrap();
    p.fill(tiny_skia::Color::from_rgba8(240, 230, 200, 255));
    let half = tiny_skia::Rect::from_xywh(SIDE as f32 / 2.0, 0.0, SIDE as f32 / 2.0, SIDE as f32).unwrap();
    let mut clear = tiny_skia::Paint::default();
    clear.set_color(tiny_skia::Color::from_rgba8(30, 40, 50, 90));
    clear.blend_mode = tiny_skia::BlendMode::Source;
    p.fill_rect(half, &clear, Transform::identity(), None);
    p
}

fn drawn(ops: Vec<Op>) -> Pixmap {
    let mut canvas = ground();
    cpu::play(ops, &mut canvas.as_mut(), &Shadows::default());
    canvas
}

fn fixtures() -> ((Vec<Glyph>, Rc<[u8; 256]>), Rc<Pixmap>) {
    let letters = letters();
    assert_eq!(letters.0.len(), 8);
    (letters, Rc::new(picture(30, 20, 200)))
}

#[test]
fn every_op_comes_back_as_it_went() {
    let (shared, image) = fixtures();
    let ops = scene(&shared, &image);
    let frame = Encoder::default().frame(&ops).unwrap();
    let back = Decoder::default().frame(&frame.control, &frame.pixels).unwrap();
    assert_eq!(back.len(), ops.len());
    for (n, (a, b)) in ops.iter().zip(&back).enumerate() {
        assert!(same_op(a, b), "op {n} differs");
    }
}

#[test]
fn the_decoded_list_draws_the_same_pixels() {
    let (shared, image) = fixtures();
    let direct = drawn(scene(&shared, &image));
    assert!(direct.data() != ground().data());
    let (mut enc, mut dec) = (Encoder::default(), Decoder::default());
    for round in 0..3 {
        let frame = enc.frame(&scene(&shared, &image)).unwrap();
        if round > 0 {
            assert!(frame.pixels.len() < 1000, "uploads sent again");
        }
        let back = drawn(dec.frame(&frame.control, &frame.pixels).unwrap());
        assert!(back.data() == direct.data(), "round {round} draws differently");
    }
}

#[test]
fn the_wire_is_smaller_than_the_text_it_draws() {
    let (shared, image) = fixtures();
    let mut enc = Encoder::default();
    enc.frame(&scene(&shared, &image)).unwrap();
    let again = enc.frame(&scene(&shared, &image)).unwrap();
    assert!(again.control.len() < 1500, "{} bytes", again.control.len());
}

#[test]
fn a_pixels_op_cannot_be_sent() {
    let ops = vec![Op::Pixels { at: at(0, 0, 4, 4), draw: Box::new(|_: &mut PixmapMut| {}) }];
    assert_eq!(Encoder::default().frame(&ops).err(), Some(Error::Pixels));
}

#[test]
fn what_cannot_be_read_back_is_not_sent() {
    let dash = tiny_skia::StrokeDash::new(vec![2.0, 2.0], 0.0).unwrap();
    let ops = vec![Op::Path { path: triangle(), how: How::Stroke(Stroke { dash: Some(dash), ..Stroke::default() }), colour: INK }];
    assert_eq!(Encoder::default().frame(&ops).err(), Some(Error::Dash));
    for bad in [f32::NAN, f32::INFINITY, 1e9] {
        let ops = vec![Op::Fill { r: r(bad, 0.0, 1.0, 1.0), colour: INK }];
        assert_eq!(Encoder::default().frame(&ops).err(), Some(Error::Bad("number")));
    }
}

#[test]
fn the_encoder_starts_again_when_its_uploads_are_full() {
    let table = Rc::new([0u8; 256]);
    let many = |n: usize, from: usize| -> Vec<Op<'static>> {
        let masks: Vec<Rc<Mask>> = (from..from + n).map(|_| Rc::new(Mask::from_parts(0, 0, 1, 1, Box::new([9])))).collect();
        masks.chunks(MAX_RUN).map(|c| Op::Glyphs {
            run: c.iter().map(|m| Glyph { key: MaskKey { font: fontdb::ID::dummy(), glyph: 0, size: 0, weight: 0, lean: false }, mask: m.clone(), x: 1, y: 2 }).collect(),
            colour: INK,
            table: table.clone(),
        }).collect()
    };
    let (mut enc, mut dec) = (Encoder::default(), Decoder::default());
    for round in 0..3 {
        let ops = many(10_000, round * 10_000);
        let frame = enc.frame(&ops).unwrap();
        assert_eq!(frame.control[6], u8::from(round > 0), "round {round}");
        assert_eq!(dec.frame(&frame.control, &frame.pixels).unwrap().len(), ops.len());
    }
    assert_eq!(enc.frame(&many(MAX_MASKS + 1, 0)).err(), Some(Error::TooBig("frame")));
    let ops = many(10, 0);
    let frame = enc.frame(&ops).unwrap();
    assert_eq!(dec.frame(&frame.control, &frame.pixels).unwrap().len(), 1);
}

#[test]
fn a_letter_freed_and_another_made_is_not_taken_for_it() {
    let table = Rc::new([0u8; 256]);
    let glyph = |m: &Rc<Mask>| Op::Glyphs {
        run: vec![Glyph { key: MaskKey { font: fontdb::ID::dummy(), glyph: 0, size: 0, weight: 0, lean: false }, mask: m.clone(), x: 0, y: 0 }],
        colour: INK,
        table: table.clone(),
    };
    let (mut enc, mut dec) = (Encoder::default(), Decoder::default());
    for n in 1..40u8 {
        let m = Rc::new(Mask::from_parts(0, 0, 1, 1, Box::new([n])));
        let frame = enc.frame(&[glyph(&m)]).unwrap();
        let back = dec.frame(&frame.control, &frame.pixels).unwrap();
        let Op::Glyphs { run, .. } = &back[0] else { panic!() };
        assert_eq!(run[0].mask.parts().4, [n]);
    }
}

fn header(flags: u16, version: u16) -> W {
    let mut w = W(MAGIC.to_vec());
    w.0.extend_from_slice(&version.to_le_bytes());
    w.0.extend_from_slice(&flags.to_le_bytes());
    w
}

fn ops_frame(n: u64, ops: impl FnOnce(&mut W)) -> Vec<u8> {
    let mut w = header(0, VERSION);
    w.uv(0);
    w.uv(n);
    ops(&mut w);
    w.0
}

fn fails(control: &[u8], pixels: &[u8], want: Error) {
    assert_eq!(Decoder::default().frame(control, pixels).err(), Some(want));
}

fn rect_bytes(w: &mut W, v: &[f32]) {
    v.iter().copied().for_each(|v| w.0.extend_from_slice(&v.to_le_bytes()));
}

#[test]
fn a_header_that_is_wrong_is_turned_away() {
    let mut ok = header(0, VERSION);
    ok.uv(0);
    ok.uv(0);
    assert!(Decoder::default().frame(&ok.0, &[]).unwrap().is_empty());
    let mut bad = ok.0.clone();
    bad[0] = b'X';
    fails(&bad, &[], Error::Bad("not a frame"));
    let mut wrong = header(0, VERSION + 1);
    wrong.uv(0);
    wrong.uv(0);
    fails(&wrong.0, &[], Error::Version);
    let mut flags = header(2, VERSION);
    flags.uv(0);
    flags.uv(0);
    fails(&flags.0, &[], Error::Bad("flags"));
    let mut trailing = ok.0.clone();
    trailing.push(0);
    fails(&trailing, &[], Error::Bad("bytes left over"));
    fails(&ok.0, &[1], Error::Bad("bytes left over"));
}

#[test]
fn numbers_that_are_not_finite_or_too_large_are_turned_away() {
    for v in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY, 2e6, -2e6] {
        let c = ops_frame(1, |w| {
            w.0.push(FILL);
            rect_bytes(w, &[v, 0.0, 1.0, 1.0]);
            rect_bytes(w, &[0.0; 4]);
        });
        fails(&c, &[], Error::Bad("number"));
    }
    let c = ops_frame(1, |w| {
        w.0.push(SHADOW);
        rect_bytes(w, &[0.0, 0.0, 20000.0, 5.0]);
    });
    fails(&c, &[], Error::Bad("number"));
    let c = ops_frame(1, |w| {
        w.0.push(LAID);
        w.uv(2 * ((1u64 << 24) + 1));
        w.uv(0);
    });
    fails(&c, &[], Error::Bad("place"));
}

#[test]
fn counts_over_the_limit_or_over_the_bytes_are_turned_away() {
    fails(&ops_frame(MAX_OPS as u64 + 1, |_| {}), &[], Error::TooBig("number"));
    fails(&ops_frame(1 << 40, |_| {}), &[], Error::TooBig("number"));
    fails(&ops_frame(5, |w| w.0.push(POP)), &[], Error::Short);
    let glyphs = |n: u64| {
        ops_frame(1, |w| {
            w.0.push(GLYPHS);
            rect_bytes(w, &[0.0; 4]);
            w.uv(0);
            w.uv(n);
        })
    };
    let mut c = header(0, VERSION);
    c.uv(1);
    c.0.push(UP_TABLE);
    c.uv(0);
    c.0.extend_from_slice(&[0; 256]);
    c.uv(1);
    c.0.push(GLYPHS);
    rect_bytes(&mut c, &[0.0; 4]);
    c.uv(0);
    c.uv(MAX_RUN as u64 + 1);
    fails(&c.0, &[], Error::TooBig("number"));
    c.0.truncate(c.0.len() - 3);
    c.uv(MAX_RUN as u64);
    fails(&c.0, &[], Error::Short);
    fails(&glyphs(3), &[], Error::Bad("table not sent"));
    let path = |n: u64| {
        ops_frame(1, |w| {
            w.0.push(PATH);
            w.0.push(HOW_WINDING);
            rect_bytes(w, &[0.0; 4]);
            w.uv(n);
        })
    };
    fails(&path(MAX_PATH_SEGMENTS as u64 + 1), &[], Error::TooBig("number"));
    fails(&path(MAX_PATH_SEGMENTS as u64), &[], Error::Short);
    let falls = ops_frame(1, |w| {
        w.0.push(SHADOW);
        rect_bytes(w, &[0.0; 4]);
        rect_bytes(w, &[0.0; 1]);
        rect_bytes(w, &[0.0; 4]);
        w.uv(MAX_FALLS as u64 + 1);
    });
    fails(&falls, &[], Error::TooBig("number"));
}

#[test]
fn paths_that_make_no_shape_or_use_a_made_up_segment_are_turned_away() {
    let one = |segment: u8, floats: usize| {
        ops_frame(1, |w| {
            w.0.push(PATH);
            w.0.push(HOW_WINDING);
            rect_bytes(w, &[0.0; 4]);
            w.uv(1);
            w.0.push(segment);
            (0..floats).for_each(|_| rect_bytes(w, &[1.0; 1]));
        })
    };
    fails(&one(0, 2), &[], Error::Bad("path"));
    fails(&one(9, 0), &[], Error::Bad("path segment"));
}

#[test]
fn layers_nested_too_deep_or_too_big_are_turned_away() {
    let push = |w: &mut W, side: u64| {
        w.0.push(PUSH);
        w.uv(0);
        w.uv(0);
        w.uv(side);
        w.uv(side);
        rect_bytes(w, &[1.0, 0.0]);
    };
    let deep = |n: u64| ops_frame(n, |w| (0..n).for_each(|_| push(w, 4)));
    assert_eq!(Decoder::default().frame(&deep(MAX_DEPTH as u64), &[]).unwrap().len(), MAX_DEPTH);
    fails(&deep(MAX_DEPTH as u64 + 1), &[], Error::TooBig("layers"));
    fails(&ops_frame(1, |w| push(w, MAX_SIDE + 1)), &[], Error::TooBig("picture size"));
    fails(&ops_frame(1, |w| push(w, MAX_SIDE)), &[], Error::TooBig("picture size"));
    fails(&ops_frame(3, |w| (0..3).for_each(|_| push(w, 5792))), &[], Error::TooBig("layers"));
    let popped = ops_frame(MAX_DEPTH as u64 * 3, |w| {
        for _ in 0..MAX_DEPTH * 3 {
            push(w, 4);
            w.0.push(POP);
        }
    });
    assert!(Decoder::default().frame(&popped, &[]).is_err());
    let stray = ops_frame(2, |w| w.0.extend_from_slice(&[POP, POP]));
    assert_eq!(Decoder::default().frame(&stray, &[]).unwrap().len(), 2);
}

#[test]
fn uploads_that_are_wrong_are_turned_away() {
    let one = |upload: &dyn Fn(&mut W)| {
        let mut w = header(0, VERSION);
        w.uv(1);
        upload(&mut w);
        w.uv(0);
        w.0
    };
    let mask = |w: &mut W, id: u64, side: u64| {
        w.0.push(UP_MASK);
        w.uv(id);
        w.uv(0);
        w.uv(0);
        w.uv(side);
        w.uv(side);
    };
    let image = |w: &mut W, wide: u64, high: u64| {
        w.0.push(UP_IMAGE);
        w.uv(1);
        w.uv(wide);
        w.uv(high);
    };
    fails(&one(&|w| mask(w, 0, MAX_MASK_SIDE + 1)), &[], Error::TooBig("number"));
    fails(&one(&|w| mask(w, 1 << 33, 1)), &[], Error::TooBig("number"));
    fails(&one(&|w| mask(w, 0, MAX_MASK_SIDE)), &[0; 100], Error::TooBig("letters kept"));
    fails(&one(&|w| mask(w, 0, 2048)), &[0; 100], Error::Short);
    fails(&one(&|w| image(w, MAX_SIDE + 1, 1)), &[], Error::TooBig("picture size"));
    fails(&one(&|w| image(w, 0, 4)), &[], Error::TooBig("picture size"));
    fails(&one(&|w| image(w, MAX_SIDE, MAX_SIDE)), &[], Error::TooBig("picture size"));
    fails(&one(&|w| image(w, 2, 2)), &[0; 15], Error::Short);
    assert_eq!(pixmap(&mut Rd(&[200, 0, 0, 100]), 1, 1).unwrap().data(), [100, 0, 0, 100]);
    fails(&one(&|w| w.0.extend([77, 0, 0, 0])), &[], Error::Bad("upload"));
    let twice = {
        let mut w = header(0, VERSION);
        w.uv(2);
        mask(&mut w, 3, 1);
        mask(&mut w, 3, 1);
        w.uv(0);
        w.0
    };
    fails(&twice, &[1, 1], Error::Bad("letter id used twice"));
}

#[test]
fn a_bad_frame_leaves_a_decoder_that_refuses_the_rest() {
    let mut dec = Decoder::default();
    assert!(dec.frame(&[1, 2, 3], &[]).is_err());
    let mut ok = header(0, VERSION);
    ok.uv(0);
    ok.uv(0);
    assert_eq!(dec.frame(&ok.0, &[]).err(), Some(Error::Failed));
}

#[test]
fn a_strict_start_of_a_frame_is_never_a_frame() {
    let (shared, image) = fixtures();
    let frame = Encoder::default().frame(&scene(&shared, &image)).unwrap();
    assert!(Decoder::default().frame(&frame.control, &frame.pixels).is_ok());
    for n in 0..frame.control.len() {
        assert!(Decoder::default().frame(&frame.control[..n], &frame.pixels).is_err(), "control cut at {n}");
    }
    for n in 0..frame.pixels.len() {
        assert!(Decoder::default().frame(&frame.control, &frame.pixels[..n]).is_err(), "pixels cut at {n}");
    }
}

struct Rng(u64);

impl Rng {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }

    fn below(&mut self, n: usize) -> usize {
        (self.next() % n as u64) as usize
    }

    fn bytes(&mut self, n: usize) -> Vec<u8> {
        (0..n).map(|_| self.next() as u8).collect()
    }
}

#[test]
fn the_largest_number_is_not_taken_for_a_place() {
    let mut w = W(vec![0xff; 9]);
    w.0.push(1);
    assert_eq!(Rd(&w.0).iv(), Err(Error::Bad("place")));
}

#[test]
fn two_peers_never_share_a_name_for_their_letters_or_pictures() {
    let (shared, image) = fixtures();
    let frame = Encoder::default().frame(&scene(&shared, &image)).unwrap();
    let (a, b) = (Decoder::default().frame(&frame.control, &frame.pixels).unwrap(), Decoder::default().frame(&frame.control, &frame.pixels).unwrap());
    let names = |ops: &[Op]| {
        let mut found = HashSet::new();
        for op in ops {
            match op {
                Op::Glyphs { run, .. } => found.extend(run.iter().map(|g| (u64::from(g.key.glyph) << 32) | u64::from(g.key.size))),
                Op::Image { id, .. } => {
                    found.insert(*id);
                }
                _ => {}
            }
        }
        found
    };
    let (a, b) = (names(&a), names(&b));
    assert!(a.len() > 1);
    assert!(a.is_disjoint(&b));
}

#[test]
fn a_colour_out_of_range_is_turned_away_and_never_sent() {
    let frame = ops_frame(1, |w| {
        w.0.push(FILL);
        rect_bytes(w, &[0.0, 0.0, 1.0, 1.0, 2.0, 0.0, 0.0, 1.0]);
    });
    fails(&frame, &[], Error::Bad("colour"));
    let over = [Op::Fill { r: r(0.0, 0.0, 1.0, 1.0), colour: Colour { r: 1.5, g: 0.0, b: 0.0, a: 1.0 } }];
    let frame = Encoder::default().frame(&over).unwrap();
    assert!(Decoder::default().frame(&frame.control, &frame.pixels).is_ok());
}

#[test]
fn shadows_that_would_make_the_drawer_build_too_much_are_turned_away() {
    let frame = ops_frame(1, |w| {
        w.0.push(INSIDE);
        rect_bytes(w, &[0.0, 0.0, 16384.0, 16384.0, 0.0, 0.0, 16384.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0]);
    });
    fails(&frame, &[], Error::TooBig("shadows"));
}

#[test]
fn random_bytes_never_panic_or_grow_without_bound() {
    let mut rng = Rng(0x9e37_79b9_7f4a_7c15);
    let mut head = header(0, VERSION).0;
    head.push(0);
    let mut got = 0;
    for n in 0..20_000 {
        let mut control = if n % 4 == 0 { Vec::new() } else { head.clone() };
        let n = rng.below(300);
        control.extend(rng.bytes(n));
        let n = rng.below(200);
        let pixels = rng.bytes(n);
        let past_the_header = !matches!(Decoder::default().frame(&control, &pixels), Err(Error::Bad("not a frame" | "flags") | Error::Version));
        got += u32::from(past_the_header);
    }
    assert!(got > 0, "the loop never got past the header");
}

#[test]
fn once_handed_over_no_copy_of_a_letter_or_picture_stays_and_what_comes_next_still_names_them() {
    let (shared, image) = fixtures();
    let (mut enc, mut dec) = (Encoder::default(), Decoder::default());
    let frame = enc.frame(&scene(&shared, &image)).unwrap();
    let ops = dec.frame(&frame.control, &frame.pixels).unwrap();
    let news = dec.news().unwrap();
    assert!(!news.reset);
    assert_eq!((news.masks.len(), news.images.len()), (8, 1));
    assert!(news.masks.iter().all(|(_, full, hollow)| !full.parts().4.is_empty() && hollow.parts().4.is_empty() && hollow.parts().2 == full.parts().2));
    assert_eq!(news.images[0].1.width(), 30);
    assert!(dec.masks.values().all(|(_, m)| m.parts().4.is_empty()), "no letter's cover is kept");
    assert!(dec.images.values().all(|(_, p)| p.width() == 1), "no picture is kept");
    drop((ops, news));
    let frame = enc.frame(&scene(&shared, &image)).unwrap();
    let again = dec.frame(&frame.control, &frame.pixels).unwrap();
    assert!(again.iter().any(|op| matches!(op, Op::Glyphs { run, .. } if run[0].mask.parts().2 > 0)));
    let news = dec.news().unwrap();
    assert!(news.masks.is_empty() && news.images.is_empty(), "nothing is new");
    enc.start_again();
    let frame = enc.frame(&scene(&shared, &image)).unwrap();
    dec.frame(&frame.control, &frame.pixels).unwrap();
    let news = dec.news().unwrap();
    assert!(news.reset, "the store is to be emptied");
    assert_eq!((news.masks.len(), news.images.len()), (8, 1));
}

#[test]
fn held_to_a_window_a_layer_far_bigger_than_it_is_turned_away() {
    let layer = |w: u32, h: u32| vec![Op::Push { at: at(0, 0, w, h), opacity: 0.5, radius: 0.0 }, Op::Pop];
    let frame = Encoder::default().frame(&layer(4000, 3000)).unwrap();
    assert!(Decoder::default().frame(&frame.control, &frame.pixels).is_ok());
    let mut held = Decoder::default();
    held.hold_to(1000, 700);
    assert_eq!(held.frame(&frame.control, &frame.pixels).err(), Some(Error::TooBig("layers")));
    let frame = Encoder::default().frame(&layer(1900, 1400)).unwrap();
    let mut held = Decoder::default();
    held.hold_to(1000, 700);
    assert!(held.frame(&frame.control, &frame.pixels).is_ok());
}

#[test]
fn a_frame_changed_a_little_never_panics() {
    let (shared, image) = fixtures();
    let frame = Encoder::default().frame(&scene(&shared, &image)).unwrap();
    let mut rng = Rng(0x2545_f491_4f6c_dd1d);
    let (mut good, mut bad) = (0, 0);
    for _ in 0..20_000 {
        let (mut control, mut pixels) = (frame.control.clone(), frame.pixels.clone());
        for _ in 0..1 + rng.below(4) {
            let target = if rng.below(8) == 0 { &mut pixels } else { &mut control };
            let at = rng.below(target.len());
            match rng.below(5) {
                0 => target[at] = rng.next() as u8,
                1 => target[at] ^= 1 << rng.below(8),
                2 => target.truncate(at),
                3 => target.insert(at, rng.next() as u8),
                _ => {
                    let end = (at + rng.below(16)).min(target.len());
                    let chunk = target[at..end].to_vec();
                    target.splice(at..at, chunk);
                }
            }
            if target.is_empty() {
                target.push(0);
            }
        }
        match Decoder::default().frame(&control, &pixels) {
            Ok(ops) => {
                good += 1;
                drawn(ops);
            }
            Err(_) => bad += 1,
        }
    }
    assert!(good > 0 && bad > 0, "{good} good, {bad} bad");
}
