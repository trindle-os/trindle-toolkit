//! The GPU's tests: what it draws against what the processor draws, and how its atlases fill and grow.

use std::time::Instant;

use tiny_skia::PixmapMut;

use super::super::{cpu, Glyph, Painter};
use super::*;
use crate::sharp::{Hinting, Sharp};
use crate::sys::os::thread_time;
use crate::theme::{Fall, Theme};

const INK: Colour = Colour { r: 0.2, g: 0.5, b: 0.9, a: 0.7 };
const FALLS: [Fall; 2] = [Fall { y: 3.0, blur: 8.0, spread: 0.0, alpha: 0.4 }, Fall { y: 1.0, blur: 2.0, spread: 0.0, alpha: 0.2 }];

fn ground(w: u32, h: u32) -> Pixmap {
    let mut p = Pixmap::new(w, h).unwrap();
    p.fill(tiny_skia::Color::from_rgba8(240, 230, 200, 255));
    let mut clear = tiny_skia::Paint::default();
    clear.set_color(tiny_skia::Color::from_rgba8(30, 40, 50, 90));
    clear.blend_mode = tiny_skia::BlendMode::Source;
    p.fill_rect(tiny_skia::Rect::from_xywh(w as f32 / 2.0, 0.0, w as f32 / 2.0, h as f32).unwrap(), &clear, Transform::identity(), None);
    p
}

fn both(gpu: &mut Gpu, under: &Pixmap, record: impl Fn(&mut Painter)) -> (Pixmap, Pixmap) {
    let (shadows, mut drawn) = (Shadows::default(), under.clone());
    let mut p = Painter::default();
    record(&mut p);
    cpu::play(p.ops, &mut drawn.as_mut(), &shadows);
    let t = gpu.target(under.width(), under.height()).unwrap();
    gpu.put(&t, under);
    let mut p = Painter::default();
    record(&mut p);
    gpu.play(p.ops, &t, &shadows);
    let back = gpu.read(&t).unwrap();
    assert_eq!(gpu.error(), 0, "GL error");
    (drawn, back)
}

fn widest(a: &Pixmap, b: &Pixmap) -> (u8, usize) {
    a.data().iter().zip(b.data()).enumerate().map(|(i, (x, y))| (x.abs_diff(*y), i / 4)).max().unwrap_or((0, 0))
}

fn agree(gpu: &mut Gpu, name: &str, under: &Pixmap, record: impl Fn(&mut Painter)) {
    let (c, g) = both(gpu, under, record);
    assert!(c.data() != under.data(), "{name}: the processor drew nothing");
    let (most, at) = widest(&c, &g);
    let w = c.width() as usize;
    assert!(most <= 2, "{name}: {most} levels apart at ({}, {}): processor {:?}, GPU {:?}", at % w, at / w, &c.data()[at * 4..at * 4 + 4], &g.data()[at * 4..at * 4 + 4]);
}

fn gpu() -> Gpu {
    let g = Gpu::new().expect("no GLES 3.0 context");
    println!("renderer {}", g.renderer());
    g
}

fn r(x: f32, y: f32, w: f32, h: f32) -> Rect {
    Rect { x, y, w, h }
}

fn at(x: i32, y: i32, w: u32, h: u32) -> IntRect {
    IntRect::from_xywh(x, y, w, h).unwrap()
}

fn triangle() -> Path {
    let mut b = PathBuilder::new();
    b.move_to(10.3, 80.0);
    b.line_to(70.6, 12.2);
    b.line_to(88.0, 70.5);
    b.close();
    b.finish().unwrap()
}

#[test]
#[ignore = "needs a GPU and Mesa"]
fn the_same_letter_from_two_stores_lies_in_one_place_and_only_the_same_letter_does() {
    let mut g = gpu();
    g.serve(256);
    let (mut a, mut b) = (g.store(64, 64), g.store(64, 64));
    let key = |size: u32| MaskKey { font: fontdb::ID::dummy(), glyph: 1, size, weight: 1, lean: false };
    let mask = |v: u8, left: i32| Rc::new(crate::sharp::Mask::from_parts(left, 0, 3, 3, vec![v; 9].into_boxed_slice()));
    let place = |g: &mut Gpu, s: &mut Store, k: u32, m: Rc<crate::sharp::Mask>| {
        g.swap_store(s);
        assert!(g.letter(key(k), &m, Rc::new(m.hollow())));
        let spot = g.glyphs[&key(k)].1;
        g.swap_store(s);
        (spot.u, spot.v, spot.tex)
    };
    let first = place(&mut g, &mut a, 1, mask(9, 0));
    assert_eq!(place(&mut g, &mut b, 2, mask(9, 5)), first, "the same bytes, wherever each program puts them");
    assert_eq!(g.shared_letters(), 1);
    assert_ne!(place(&mut g, &mut b, 3, mask(8, 0)), first, "other bytes");
    assert_eq!(g.shared_letters(), 2);
    let big = |v: u8| Rc::new(crate::sharp::Mask::from_parts(0, 0, 60, 60, vec![v; 3600].into_boxed_slice()));
    g.swap_store(&mut a);
    g.shared_bytes = SHARE_MOST;
    let own = g.masks.tex;
    assert!(g.letter(key(4), &big(1), Rc::new(big(1).hollow())));
    assert_eq!(g.glyphs[&key(4)].1.tex, own);
    assert!(g.letter(key(5), &mask(9, 0), Rc::new(mask(9, 0).hollow())));
    assert_eq!(g.glyphs[&key(5)].1.tex, first.2);
    assert!(!g.letter(key(6), &big(2), Rc::new(big(2).hollow())), "no room left in its own: it must start again");
    g.empty(Texel::R8);
    assert!(g.glyphs.contains_key(&key(5)));
    assert!(!g.glyphs.contains_key(&key(4)));
    g.empty_store();
    assert_eq!(g.shared_bytes, SHARE_MOST);
    g.swap_store(&mut a);
    g.free_store(a);
    g.free_store(b);
}

#[test]
#[ignore = "needs a GPU and Mesa"]
fn a_wide_thin_picture_and_a_tall_thin_one_make_no_vast_island() {
    let mut g = gpu();
    let t = g.target(2048, 2048).unwrap();
    let (wide, tall) = (Pixmap::new(2048, 1).unwrap(), Pixmap::new(1, 2048).unwrap());
    g.play(vec![Op::Laid { x: 0, y: 0, pixmap: wide }, Op::Laid { x: 0, y: 0, pixmap: tall }], &t, &Shadows::default());
    assert_eq!(g.error(), 0);
    let island = g.island.unwrap();
    assert!(island.w * island.h <= 2048, "{} x {}", island.w, island.h);
    let t = g.target(3841, 2161).unwrap();
    g.play(vec![Op::Laid { x: 0, y: 0, pixmap: Pixmap::new(3841, 2161).unwrap() }], &t, &Shadows::default());
    assert!(g.island.is_none(), "kept past its frame");
    assert_eq!(g.error(), 0);
}

#[test]
#[ignore = "needs a GPU and Mesa"]
fn a_box_a_million_across_makes_a_mask_only_of_what_is_seen() {
    let mut g = gpu();
    let t = g.target(64, 48).unwrap();
    let mut p = Painter::default();
    p.round(r(-500_000.0, -300.0, 1_000_000.0, 1_000_000.0), [30.0; 4], INK);
    p.round(r(-2000.0, 5.0, 1_000_000.0, 8.0), [4.0; 4], INK);
    p.round(r(-2_000_000.0, 5.0, 5.0, 8.0), [4.0; 4], INK);
    g.play(p.ops, &t, &Shadows::default());
    assert_eq!(g.error(), 0);
    assert!(g.masks.top <= 48 * 2, "{} rows of the atlas taken", g.masks.top);
    let got = g.read(&t).unwrap();
    assert!(got.data()[(20 * 64 + 20) * 4 + 3] > 0, "it is drawn");
    g.free(t);
}

#[test]
fn places_given_before_an_atlas_doubles_stay_free_of_those_given_after() {
    let mut a = Atlas { tex: 0, texel: Texel::R8, shelves: Vec::new(), top: 0, side: START, max: SIDE };
    let mut given = Vec::new();
    while let Some(at) = a.take(100, 60) {
        given.push((at, (100, 60)));
    }
    let before = given.len();
    assert!(before >= 8);
    a.side *= 2;
    while let Some(at) = a.take(100, 60) {
        given.push((at, (100, 60)));
    }
    assert!(given.len() > before * 3);
    for (i, ((x, y), (w, h))) in given.iter().enumerate() {
        assert!(x + w <= a.side && y + h <= a.side);
        for ((x2, y2), (w2, h2)) in &given[i + 1..] {
            assert!(x + w <= *x2 || x2 + w2 <= *x || y + h <= *y2 || y2 + h2 <= *y, "{:?} and {:?} overlap", (x, y), (x2, y2));
        }
    }
}

#[test]
#[ignore = "needs a GPU and Mesa"]
fn a_round_box_cut_by_the_edge_of_a_layer_is_drawn_as_the_processor_draws_it() {
    let mut g = gpu();
    let under = ground(260, 220);
    let solid = Colour { r: 0.7, g: 0.3, b: 0.2, a: 1.0 };
    let cases = [
        ((53, 7), r(23.274708, 3.5624695, 40.612137, 36.946068), [24.51154; 4]),
        ((3, 52), r(1.3202548, 21.829432, 87.18031, 54.911564), [23.345684; 4]),
        ((73, 73), r(5.62551, 37.320015, 45.910915, 38.55813), [14.411026, 12.901555, 24.568062, 7.9646482]),
        ((19, 45), r(16.070217, 20.333174, 75.16395, 48.860092), [0.6840095, 1.726371, 18.797598, 7.4499087]),
        ((10, 10), r(-12.3, -7.6, 50.4, 30.2), [11.0; 4]),
    ];
    for (n, ((x, y), shape, radii)) in cases.into_iter().enumerate() {
        agree(&mut g, &format!("case {n}"), &under, |p| {
            p.push_clip(at(x, y, 61, 69), 0.0);
            p.round(shape, radii, solid);
            p.pop();
        });
    }
}

#[test]
#[ignore = "needs a GPU and Mesa"]
fn a_letter_whose_mask_changed_under_the_same_key_is_drawn_from_the_new_mask() {
    let mut g = gpu();
    let under = ground(60, 60);
    let (sharp, hinting) = (Sharp::default(), Hinting::default());
    let data = include_bytes!("../../../share/fonts/NotoSans-Regular.ttf");
    let key = |size: f32| MaskKey { font: fontdb::ID::dummy(), glyph: 36, size: size.to_bits(), weight: 400, lean: false };
    let table = sharp.table(INK);
    for size in [18.0, 30.0, 18.0, 12.0] {
        let mask = sharp.mask(data, 0, "Noto Sans", &hinting, key(size)).unwrap();
        agree(&mut g, &format!("size {size} under the key of 18"), &under, |p| {
            p.glyphs(vec![Glyph { key: key(18.0), mask: mask.clone(), x: 10, y: 40 }], INK, table.clone());
        });
    }
}

#[test]
#[ignore = "needs a GPU and Mesa"]
fn every_op_draws_within_two_levels_of_the_processor() {
    let mut g = gpu();
    let under = ground(160, 120);
    let solid = Colour { a: 1.0, ..INK };
    agree(&mut g, "fill", &under, |p| {
        p.fill(r(3.3, 4.6, 70.2, 30.5), INK);
        p.fill(r(60.25, 50.75, 90.5, 0.4), solid);
        p.fill(r(10.0, 80.0, 140.0, 30.0), solid);
    });
    agree(&mut g, "round", &under, |p| {
        p.round(r(6.4, 8.2, 80.0, 50.7), [9.5; 4], INK);
        p.round(r(90.5, 10.25, 60.3, 100.7), [0.0, 30.0, 4.0, 12.5], solid);
        p.round(r(10.0, 70.0, 70.0, 40.0), [20.0; 4], solid);
        p.round(r(20.0, 64.0, 130.0, 50.0), [25.0; 4], INK);
    });
    agree(&mut g, "line", &under, |p| {
        p.line((5.5, 7.0), (150.2, 110.3), 2.5, INK);
        p.line((5.0, 100.5), (150.0, 100.5), 1.0, solid);
    });
    agree(&mut g, "paths", &under, |p| {
        p.fill_path(triangle(), FillRule::EvenOdd, INK);
        p.stroke_path(triangle(), Stroke { width: 3.2, ..Stroke::default() }, solid);
    });
    agree_on_letters(&mut g, &under, solid);
    agree_on_image(&mut g, &under, solid);
    if !crate::theme::drawn_by_processor() {
        let shadow = Colour { a: 1.0, ..INK };
        agree(&mut g, "shadows", &under, |p| {
            p.shadow(r(20.0, 20.0, 50.0, 40.0), 8.0, &FALLS, shadow);
            p.shadow(r(70.4, 30.6, 80.0, 70.0), 12.0, &FALLS[..1], shadow);
            p.shadow_inside(r(20.0, 70.0, 120.0, 40.0), 8.0, FALLS[0], shadow);
        });
    }
    agree(&mut g, "rounded clip", &under, |p| {
        p.push_clip(at(10, 12, 120, 90), 12.0);
        p.fill(r(-5.0, -5.0, 200.0, 70.0), INK);
        p.round(r(30.0, 30.0, 100.0, 100.0), [10.0; 4], solid);
        p.pop();
    });
    agree(&mut g, "layers", &under, |p| {
        p.push_layer(at(8, 9, 140, 100), 0.6);
        p.push_clip(at(5, 5, 40, 30), 0.0);
        p.fill(r(2.5, 3.5, 60.0, 40.0), INK);
        p.pop();
        p.line((0.0, 0.0), (120.0, 90.0), 1.5, INK);
        p.push_layer(at(60, 40, 50, 50), 0.5);
        p.fill(r(0.0, 0.0, 50.0, 50.0), solid);
    });
    agree(&mut g, "pixels", &under, |p| {
        p.push_clip(at(20, 10, 100, 100), 0.0);
        p.pixels(at(5, 5, 90, 90), |c: &mut PixmapMut| {
            c.fill_path(&triangle(), &paint(INK), FillRule::Winding, Transform::identity(), None);
        });
        p.pop();
    });
}

fn agree_on_letters(g: &mut Gpu, under: &Pixmap, solid: Colour) {
    let sharp = Sharp::default();
    let data = include_bytes!("../../../share/fonts/NotoSans-Regular.ttf");
    let hinting = Hinting::default();
    let run = || -> Vec<Glyph> {
        (36..50u16).enumerate().filter_map(|(n, glyph)| {
            let key = MaskKey { font: fontdb::ID::dummy(), glyph, size: 18.0f32.to_bits(), weight: 400, lean: false };
            Some(Glyph { key, mask: sharp.mask(data, 0, "Noto Sans", &hinting, key)?, x: 4 + 11 * n as i32, y: 50 })
        }).collect()
    };
    for ink in [INK, solid, Colour { r: 0.95, g: 0.95, b: 0.9, a: 1.0 }] {
        let table = sharp.table(ink);
        agree(g, "letters", under, |p| p.glyphs(run(), ink, table.clone()));
    }
}

fn agree_on_image(g: &mut Gpu, under: &Pixmap, solid: Colour) {
    let mut pix = Pixmap::new(30, 20).unwrap();
    pix.fill(tiny_skia::Color::from_rgba8(20, 120, 60, 160));
    pix.fill_path(&triangle(), &paint(solid), FillRule::Winding, Transform::from_scale(0.3, 0.2), None);
    let pix = Rc::new(pix);
    agree(g, "image", under, |p| {
        p.image(7, pix.clone(), 40, 50);
        p.image(7, pix.clone(), 140, -5);
    });
}

#[test]
#[ignore = "needs a GPU and Mesa"]
fn the_golden_pages_and_a_page_of_cards_draw_within_two_levels_of_the_processor() {
    let mut g = gpu();
    let under = Pixmap::new(300, 150).unwrap();
    for (name, theme) in crate::theme::tests::every_look() {
        agree(&mut g, &name, &under, |p| {
            let mut ui = crate::ui::Ui::new(crate::gallery::golden::parts(), theme.clone());
            ui.lay_at(300, 150);
            p.pixels(at(0, 0, 300, 150), move |c| ui.paint(c, 1.0));
        });
    }
    let under = Pixmap::new(1280, 800).unwrap();
    let shared = Shared::new();
    agree(&mut g, "cards", &under, |p| cards(p, &shared, 1280, 800, 2.0));
}

struct Shared {
    glyphs: Vec<(MaskKey, Rc<Mask>)>,
    table: Rc<[u8; 256]>,
    icon: Rc<Pixmap>,
    theme: Theme,
}

impl Shared {
    fn new() -> Shared {
        let sharp = Sharp::default();
        let data = include_bytes!("../../../share/fonts/NotoSans-Regular.ttf");
        let hinting = Hinting::default();
        let glyphs = (36..90u16).filter_map(|glyph| {
            let key = MaskKey { font: fontdb::ID::dummy(), glyph, size: 28.0f32.to_bits(), weight: 400, lean: false };
            Some((key, sharp.mask(data, 0, "Noto Sans", &hinting, key)?))
        }).collect();
        let theme = Theme::chosen();
        let mut icon = Pixmap::new(32, 32).unwrap();
        icon.fill_path(&crate::ui::rounded(r(4.0, 4.0, 24.0, 24.0), 6.0).unwrap(), &paint(theme.accent), FillRule::Winding, Transform::identity(), None);
        Shared { glyphs, table: sharp.table(theme.fg), icon: Rc::new(icon), theme }
    }

    fn words(&self, p: &mut Painter, x: f32, y: f32, n: usize, scale: f32) {
        let run = (0..n).map(|i| {
            let (key, mask) = &self.glyphs[(i * 7 + x as usize) % self.glyphs.len()];
            Glyph { key: *key, mask: mask.clone(), x: x as i32 + (i as f32 * 7.5 * scale) as i32, y: y as i32 }
        }).collect();
        p.glyphs(run, self.theme.fg, self.table.clone());
    }
}

fn cards(p: &mut Painter, s: &Shared, w: u32, h: u32, scale: f32) {
    let t = &s.theme;
    p.fill(r(0.0, 0.0, w as f32, h as f32), t.bg);
    let cols = ((w as f32 / scale / 320.0) as usize).max(1);
    let rows = ((h as f32 / scale / 230.0).ceil() as usize).max(1);
    let (gap, pad) = (16.0 * scale, 24.0 * scale);
    let cw = (w as f32 - 2.0 * pad - gap * (cols - 1) as f32) / cols as f32;
    let ch = 214.0 * scale;
    let falls: Vec<Fall> = FALLS.iter().map(|f| Fall { y: f.y * scale, blur: f.blur * scale, ..*f }).collect();
    s.words(p, pad, pad + 20.0 * scale, 14, scale);
    for row in 0..rows {
        for col in 0..cols {
            let (x, y) = (pad + col as f32 * (cw + gap), pad + 40.0 * scale + row as f32 * (ch + gap));
            let corner = 12.0 * scale;
            p.shadow(r(x, y, cw, ch), corner, &falls, t.shadow);
            p.round(r(x, y, cw, ch), [corner; 4], t.raised);
            let (ix, mut iy) = (x + 16.0 * scale, y + 16.0 * scale);
            s.words(p, ix, iy + 14.0 * scale, 16, scale);
            s.words(p, ix, iy + 38.0 * scale, ((cw / (7.5 * scale)) as usize).saturating_sub(4), scale);
            iy += 52.0 * scale;
            for b in 0..3 {
                let bx = ix + b as f32 * 96.0 * scale;
                p.round(r(bx, iy, 88.0 * scale, 32.0 * scale), [8.0 * scale; 4], t.control);
                p.image(1, s.icon.clone(), (bx + 6.0 * scale) as i32, (iy + 2.0 * scale) as i32);
                s.words(p, bx + 40.0 * scale, iy + 22.0 * scale, 5, scale);
            }
            iy += 44.0 * scale;
            p.round(r(ix, iy, 44.0 * scale, 24.0 * scale), [12.0 * scale; 4], t.accent);
            p.round(r(ix + 22.0 * scale, iy + 2.0 * scale, 20.0 * scale, 20.0 * scale), [10.0 * scale; 4], t.knob);
            iy += 34.0 * scale;
            let long = cw - 32.0 * scale;
            p.round(r(ix, iy + 9.0 * scale, long, 6.0 * scale), [3.0 * scale; 4], t.control);
            p.round(r(ix, iy + 9.0 * scale, 0.4 * long, 6.0 * scale), [3.0 * scale; 4], t.accent);
            let mut knob = PathBuilder::new();
            knob.push_circle(ix + 0.4 * long, iy + 12.0 * scale, 10.0 * scale);
            if let Some(k) = knob.finish() {
                p.fill_path(k, FillRule::Winding, t.accent);
            }
            iy += 30.0 * scale;
            p.round(r(ix, iy, long, 32.0 * scale), [8.0 * scale; 4], t.field);
            p.shadow_inside(r(ix, iy, long, 32.0 * scale), 8.0 * scale, Fall { y: scale, blur: 3.0 * scale, spread: 0.0, alpha: 0.3 }, t.shadow);
            s.words(p, ix + 10.0 * scale, iy + 22.0 * scale, 10, scale);
        }
    }
}

#[test]
#[ignore = "needs a GPU and Mesa"]
fn atlases_start_small_and_double_keeping_what_they_hold() {
    let (mut g, s) = (gpu(), Shared::new());
    assert_eq!((g.masks.side, g.pictures.side), (START, START));
    let under = ground(400, 300);
    let page = |p: &mut Painter| {
        p.image(1, s.icon.clone(), 20, 20);
        s.words(p, 60.0, 50.0, 20, 1.0);
        let mut ring = PathBuilder::new();
        ring.push_circle(300.0, 220.0, 20.0);
        p.fill_path(ring.finish().unwrap(), FillRule::Winding, INK);
    };
    agree(&mut g, "first", &under, page);
    for batch in 0..4u64 {
        let mut big = Pixmap::new(300, 300).unwrap();
        big.fill(tiny_skia::Color::from_rgba8(40, 90 + batch as u8 * 20, 160, 255));
        let big = Rc::new(big);
        agree(&mut g, &format!("batch {batch}"), &under, |p| {
            page(p);
            for n in batch * 25..batch * 25 + 25 {
                let mut ring = PathBuilder::new();
                ring.push_circle(200.0, 150.0, 70.0 - n as f32 * 0.4);
                p.fill_path(ring.finish().unwrap(), FillRule::Winding, INK);
            }
            p.image(100 + batch, big.clone(), 50, 0);
            page(p);
        });
    }
    assert!(g.masks.side >= 1024 && g.pictures.side >= 1024, "{} and {}", g.masks.side, g.pictures.side);
    assert!(g.growths[0] >= 1 && g.growths[1] >= 1, "{:?}", g.growths);
    assert_eq!(g.resets, [0, 0], "emptied with room to grow: {} and {} across, grown {:?}", g.masks.side, g.pictures.side, g.growths);
    agree(&mut g, "again", &under, page);
}

#[test]
#[ignore = "needs a GPU and Mesa"]
fn a_full_atlas_at_its_most_is_emptied_not_grown() {
    let (mut g, s) = (gpu(), Shared::new());
    g.small_atlases(512);
    let under = ground(400, 300);
    for n in 0..4u64 {
        let mut big = Pixmap::new(300, 300).unwrap();
        big.fill(tiny_skia::Color::from_rgba8(40, 90 + n as u8, 160, 255));
        let big = Rc::new(big);
        agree(&mut g, &format!("frame {n}"), &under, |p| {
            p.image(100 + n, big.clone(), 50, 0);
            s.words(p, 60.0, 50.0, 20, 1.0);
        });
    }
    assert_eq!((g.pictures.side, g.growths), (512, [0, 0]));
    assert!(g.resets[1] >= 1);
}

#[test]
#[ignore = "needs a GPU and Mesa"]
fn kept_pictures_and_letters_outlive_the_atlases_being_emptied() {
    let (mut g, s) = (gpu(), Shared::new());
    let under = ground(400, 300);
    let page = |p: &mut Painter| {
        p.image(1, s.icon.clone(), 20, 20);
        s.words(p, 60.0, 50.0, 20, 1.0);
    };
    agree(&mut g, "first", &under, page);
    for n in 0..12u64 {
        let mut big = Pixmap::new(600, 600).unwrap();
        big.fill(tiny_skia::Color::from_rgba8(40, 90 + n as u8, 160, 255));
        let big = Rc::new(big);
        let mut ring = PathBuilder::new();
        ring.push_circle(500.0 + n as f32 * 0.3, 500.0, 480.0);
        let ring = ring.finish().unwrap();
        agree(&mut g, &format!("frame {n}"), &under, |p| {
            page(p);
            p.image(100 + n, big.clone(), 150, 120);
            p.fill_path(ring.clone(), FillRule::Winding, INK);
            page(p);
        });
    }
    agree(&mut g, "again", &under, page);
}

fn median(mut v: Vec<f64>) -> f64 {
    v.sort_by(|a, b| a.total_cmp(b));
    v[v.len() / 2]
}

#[test]
#[ignore = "a timing run, needs a GPU"]
fn measure_gpu_and_processor() {
    let (mut g, s, n) = (gpu(), Shared::new(), 40);
    println!("| Window | Processor plays the list | GPU, processor time | GPU, until drawn |");
    println!("|---|---|---|---|");
    for (w, h) in [(1280u32, 800u32), (1920, 1080), (2256, 1504), (3840, 2160)] {
        let shadows = Shadows::default();
        let mut pix = Pixmap::new(w, h).unwrap();
        let record = || {
            let mut p = Painter::default();
            cards(&mut p, &s, w, h, 2.0);
            p.ops
        };
        let processor = median((0..n).map(|_| {
            let ops = record();
            let t = thread_time();
            cpu::play(ops, &mut pix.as_mut(), &shadows);
            (thread_time() - t).as_secs_f64() * 1e3
        }).collect());
        let target = g.target(w, h).unwrap();
        g.play(record(), &target, &shadows);
        g.finish();
        let (mut on_cpu, mut whole) = (Vec::new(), Vec::new());
        for _ in 0..n {
            let ops = record();
            let (t, wall) = (thread_time(), Instant::now());
            g.play(ops, &target, &shadows);
            on_cpu.push((thread_time() - t).as_secs_f64() * 1e3);
            g.finish();
            whole.push(wall.elapsed().as_secs_f64() * 1e3);
        }
        let back = g.read(&target).unwrap();
        let (most, at) = widest(&pix, &back);
        let bad = pix.data().chunks_exact(4).zip(back.data().chunks_exact(4)).filter(|(a, b)| a.iter().zip(b.iter()).any(|(x, y)| x.abs_diff(*y) > 2)).count();
        assert!(most <= 2, "{w} x {h}: {most} levels apart at ({}, {}), {bad} pixels over 2: {:?} {:?}", at % w as usize, at / w as usize, &pix.data()[at * 4..at * 4 + 4], &back.data()[at * 4..at * 4 + 4]);
        println!("| {w} x {h} | {processor:.1} ms | {:.2} ms | {:.2} ms |", median(on_cpu), median(whole));
    }
}
