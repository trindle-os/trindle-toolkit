//! Tests of bento packing.

use super::*;
use crate::text::Fonts;
use crate::widget::{button, card, column, grid, label, row, Texts};

fn words(lo: f32, hi: f32, area: f32) -> Size {
    let at = [lo, (lo + hi) / 2.0, hi].iter().map(|&w| (w, (area / w).max(area / hi).ceil())).collect();
    Size { lo, hi, at, gives: false }
}

fn fixed(lo: f32, hi: f32, h: f32) -> Size {
    Size { lo, hi, at: vec![(lo, h)], gives: false }
}

fn shape(bands: &[Vec<Column>]) -> Vec<Vec<usize>> {
    bands.iter().map(|b| b.iter().map(|c| c.count).collect()).collect()
}

#[test]
fn every_band_fills_the_width_and_keeps_the_order() {
    let sizes = vec![fixed(200.0, 260.0, 80.0), fixed(200.0, 300.0, 80.0), fixed(200.0, 220.0, 80.0),
                     fixed(200.0, 400.0, 80.0), fixed(200.0, 380.0, 80.0)];
    for width in [1280.0, 900.0, 400.0] {
        let bands = plan(&sizes, width, 12.0, &|t, w| sizes[t].h(w));
        let mut next = 0;
        for b in &bands {
            assert_eq!(b[0].x, 0.0);
            let last = b.last().unwrap();
            assert_eq!(last.x + last.w, width, "{width}: {b:?}");
            for c in b {
                assert_eq!(c.first, next, "tiles stay in the order given");
                next += c.count;
            }
        }
        assert_eq!(next, sizes.len());
    }
}

#[test]
fn tiles_are_grouped_so_lines_come_closest_to_the_width() {
    let sizes = vec![fixed(250.0, 420.0, 80.0), fixed(250.0, 456.0, 80.0),
                     fixed(250.0, 300.0, 80.0), fixed(250.0, 300.0, 80.0), fixed(250.0, 276.0, 80.0)];
    let bands = plan(&sizes, 900.0, 12.0, &|t, w| sizes[t].h(w));
    assert_eq!(shape(&bands), vec![vec![1, 1], vec![1, 1, 1]]);
    let b = &bands[1];
    let grown: Vec<f32> = b.iter().zip([300.0, 300.0, 276.0]).map(|(c, hi)| c.w - hi).collect();
    assert!(grown.iter().all(|g| (g - grown[0]).abs() <= 1.0), "{grown:?}");
}

#[test]
fn words_side_by_side_are_narrowed_to_end_level() {
    let sizes = vec![words(300.0, 900.0, 60_000.0), words(300.0, 900.0, 30_000.0)];
    let bands = plan(&sizes, 900.0, 12.0, &|t, w| sizes[t].h(w));
    assert_eq!(shape(&bands), vec![vec![1, 1]]);
    let (a, b) = (&bands[0][0], &bands[0][1]);
    assert!(a.w > b.w);
    assert!((sizes[0].h(a.w) - sizes[1].h(b.w)).abs() <= LOOSE, "{a:?} {b:?}");
}

#[test]
fn short_tiles_stack_beside_a_tall_one() {
    let sizes = vec![fixed(400.0, 440.0, 300.0), fixed(300.0, 420.0, 144.0), fixed(300.0, 420.0, 144.0)];
    let bands = plan(&sizes, 900.0, 12.0, &|t, w| sizes[t].h(w));
    assert_eq!(shape(&bands), vec![vec![1, 2]]);
}

#[test]
fn a_narrow_window_puts_each_tile_on_its_own_line() {
    let sizes = vec![fixed(300.0, 340.0, 80.0), fixed(300.0, 360.0, 120.0)];
    let bands = plan(&sizes, 400.0, 12.0, &|t, w| sizes[t].h(w));
    assert_eq!(shape(&bands), vec![vec![1], vec![1]]);
    assert!(bands.iter().all(|b| b[0].w == 400.0));
}

#[test]
fn a_picture_takes_the_spare_height_and_width() {
    let mut picture = fixed(400.0, 500.0, 100.0);
    picture.gives = true;
    let sizes = vec![picture, fixed(300.0, 320.0, 400.0)];
    let bands = plan(&sizes, 900.0, 12.0, &|t, w| sizes[t].h(w));
    assert_eq!(shape(&bands), vec![vec![1, 1]]);
    assert_eq!((bands[0][0].w, bands[0][1].w), (568.0, 320.0));
}

fn fact(n: usize) -> Node<()> {
    card(column(vec![label(format!("{n}")), label("Things counted on this computer")]))
}

fn sentence(n: usize) -> Node<()> {
    let words = "Words that say what this card does, long enough to wrap onto a second line or a third. ";
    card(column(vec![label(format!("Card {n}")), label(words.repeat(1 + n % 3)), row(vec![button("a", "A")])]))
}

#[test]
fn mixed_cards_leave_no_loose_tile_at_any_width() {
    let fonts = Fonts::new();
    for width in [1280.0, 900.0, 400.0] {
        let tiles: Vec<Node<()>> = (0..4).map(fact).chain((0..5).map(sentence)).collect();
        let tree = column(vec![grid(280.0, tiles).gap(12.0)]);
        let loose = check::loose_tiles(&fonts, &tree, width, 2000.0);
        assert!(loose.is_empty(), "at {width}: {loose:?}");
    }
}

#[test]
fn measuring_and_placing_agree() {
    let fonts = Fonts::new();
    let texts = Texts::default();
    let scrolled = HashMap::new();
    let lay = Lay::bare(&fonts, &texts, &scrolled);
    let tree = grid(280.0, (0..6).map(sentence).collect()).gap(12.0);
    let h = lay.measure(&tree, 900.0).1;
    let mut spots = Vec::new();
    lay.place(&tree, Rect { x: 0.0, y: 0.0, w: 900.0, h }, &mut Vec::new(), &mut spots);
    let bottom = spots.iter().filter(|s| s.path.len() == 1).map(|s| s.rect.y + s.rect.h).fold(0.0, f32::max);
    assert_eq!(bottom, h);
    let right = spots.iter().filter(|s| s.path.len() == 1).map(|s| s.rect.x + s.rect.w).fold(0.0, f32::max);
    assert_eq!(right, 900.0);
}

fn puzzled(sizes: &[Size], width: f32) -> Option<Vec<Rect>> {
    let h = |t: usize, w: f32| sizes[t].h(w).round();
    let seed: Vec<f32> = sizes.iter().map(|s| s.hi).collect();
    let widths = puzzle(sizes, width, 12.0, &h, &seed, f32::MAX)?;
    replay(sizes, &widths, width, 12.0, &h).map(|(r, _)| r)
}

fn assert_clean(rects: &[Rect], width: f32) {
    let mut seen: Vec<usize> = (0..rects.len()).collect();
    seen.sort_by(|&a, &b| rects[a].y.total_cmp(&rects[b].y).then(rects[a].x.total_cmp(&rects[b].x)));
    assert_eq!(seen, (0..rects.len()).collect::<Vec<_>>(), "{rects:?}");
    for (i, a) in rects.iter().enumerate() {
        assert!(a.x >= 0.0 && a.x + a.w <= width + 0.5, "{a:?}");
        for b in &rects[i + 1..] {
            let apart = a.x + a.w + 11.5 <= b.x || b.x + b.w + 11.5 <= a.x || a.y + a.h + 11.5 <= b.y || b.y + b.h + 11.5 <= a.y;
            assert!(apart, "{a:?} {b:?}");
        }
    }
    let tall = rects.iter().map(|r| r.y + r.h).fold(0.0, f32::max);
    let filled: f32 = rects.iter().map(|r| (r.w + 12.0) * (r.h + 12.0)).sum();
    assert!((filled - (width + 12.0) * (tall + 12.0)).abs() < 1.0, "a hole: {rects:?}");
}

#[test]
fn a_puzzle_locks_tiles_together_with_no_straight_gap() {
    let sizes = vec![fixed(588.0, 588.0, 100.0), fixed(300.0, 300.0, 212.0), fixed(288.0, 288.0, 212.0),
                     fixed(288.0, 288.0, 100.0), fixed(600.0, 600.0, 100.0)];
    let rects = puzzled(&sizes, 900.0).expect("the pinwheel closes");
    assert_clean(&rects, 900.0);
    assert_eq!(straight(&rects, 12.0), 0, "{rects:?}");
    assert_eq!((rects[4].x, rects[4].y), (300.0, 224.0));
    let bands = plan(&sizes, 900.0, 12.0, &|t, w| sizes[t].h(w));
    assert!(bands.len() > 1);
}

#[test]
fn every_puzzle_keeps_the_order_and_closes_a_clean_rectangle() {
    let mut luck: u32 = 11;
    let mut roll = move |n: u32| {
        luck = luck.wrapping_mul(1_103_515_245).wrapping_add(12_345);
        (luck >> 16) % n
    };
    let mut made = 0;
    for _ in 0..60 {
        let sizes: Vec<Size> = (0..3 + roll(8))
            .map(|_| {
                let lo = 240.0 + roll(120) as f32;
                words(lo, lo + roll(500) as f32, 20_000.0 + roll(100_000) as f32)
            })
            .collect();
        for width in [1280.0, 900.0, 600.0] {
            if let Some(rects) = puzzled(&sizes, width) {
                assert_clean(&rects, width);
                made += 1;
            }
        }
    }
    assert!(made > 100, "{made}");
}

#[test]
fn grids_of_every_kind_leave_no_loose_tile() {
    let fonts = Fonts::new();
    let mut luck: u32 = 3;
    let mut roll = move |n: u32| {
        luck = luck.wrapping_mul(1_103_515_245).wrapping_add(12_345);
        (luck >> 16) % n
    };
    let line = "Words that say what this card does, long enough to wrap. ";
    for _ in 0..12 {
        let tiles: Vec<Node<()>> = (0..4 + roll(6) as usize)
            .map(|i| if roll(3) == 0 {
                fact(i)
            } else {
                let rows: Vec<Node<()>> = (0..1 + roll(4)).map(|_| label(line.repeat(1 + roll(3) as usize))).collect();
                card(column(rows.into_iter().chain([row(vec![button("a", "A")])]).collect()))
            })
            .collect();
        let tree = column(vec![grid(280.0, tiles).gap(12.0)]);
        for width in [1280.0, 900.0, 400.0] {
            let loose = check::loose_tiles(&fonts, &tree, width, 2000.0);
            assert!(loose.is_empty(), "at {width}: {loose:?}");
        }
    }
}
