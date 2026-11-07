//! What changed since the last frame: each spot's recorded list signed and compared with the last frame's.

use std::collections::HashMap;
use std::hash::{DefaultHasher, Hash, Hasher};

use tiny_skia::{IntRect, Path, Pixmap, PixmapMut};

use super::list::{How, Op};
use super::{Glyph, Painter};
use crate::colour::Colour;
use crate::widget::Rect;

pub(crate) struct Mark {
    pub(crate) at: usize,
    pub(crate) key: u64,
    pub(crate) bounds: Option<IntRect>,
}

pub(crate) type Seen = HashMap<u64, (u64, IntRect)>;

const MOST: usize = 16;

pub(crate) fn changes<'a>(p: Painter<'a>, size: (u32, u32), seen: &mut Seen, scratch: &mut Option<Pixmap>) -> (Vec<Op<'a>>, Option<Vec<IntRect>>) {
    let mut marks = p.marks;
    let ops = settle(p.ops, size, scratch, &mut marks);
    let changed = damage(seen, &ops, &marks, size);
    (ops, changed)
}

pub(crate) fn settle<'a>(ops: Vec<Op<'a>>, size: (u32, u32), scratch: &mut Option<Pixmap>, marks: &mut [Mark]) -> Vec<Op<'a>> {
    let mut rooms = vec![size];
    let mut out = Vec::with_capacity(ops.len());
    let mut dropped = Vec::new();
    for (n, op) in ops.into_iter().enumerate() {
        match op {
            Op::Push { at, .. } => rooms.push((at.width(), at.height())),
            Op::Pop if rooms.len() > 1 => {
                rooms.pop();
            }
            Op::Pixels { at, draw } => {
                match rooms.last().and_then(|&room| laid(at, draw, room, scratch)) {
                    Some(laid) => out.push(laid),
                    None => dropped.push(n),
                }
                continue;
            }
            _ => {}
        }
        out.push(op);
    }
    for m in marks.iter_mut() {
        m.at -= dropped.iter().filter(|&&n| n < m.at).count();
    }
    out
}

fn laid<'a>(at: IntRect, draw: Box<dyn FnOnce(&mut PixmapMut) + 'a>, room: (u32, u32), scratch: &mut Option<Pixmap>) -> Option<Op<'a>> {
    let bytes = room.0 as usize * room.1 as usize * 4;
    if scratch.as_ref().is_none_or(|p| p.data().len() < bytes) {
        *scratch = Pixmap::new(room.0, room.1);
    }
    let bytes = scratch.as_mut()?.data_mut().get_mut(..bytes)?;
    let w = room.0 as usize;
    let x0 = at.x().clamp(0, room.0 as i32) as usize;
    let x1 = (at.x() + at.width() as i32).clamp(0, room.0 as i32) as usize;
    let y0 = at.y().clamp(0, room.1 as i32) as usize;
    let y1 = (at.y() + at.height() as i32).clamp(0, room.1 as i32) as usize;
    for row in bytes.chunks_exact_mut(w * 4).take(y1).skip(y0) {
        row[x0 * 4..x1 * 4].fill(0);
    }
    draw(&mut PixmapMut::from_bytes(&mut *bytes, room.0, room.1)?);
    let (mut left, mut top, mut right, mut bottom) = (x1, y1, x0, y0);
    for (y, row) in bytes.chunks_exact(w * 4).enumerate().take(y1).skip(y0) {
        let row = &row[x0 * 4..x1 * 4];
        let (Some(a), Some(b)) = (row.chunks_exact(4).position(|p| p[3] != 0), row.chunks_exact(4).rposition(|p| p[3] != 0)) else { continue };
        (left, right, top, bottom) = (left.min(x0 + a), right.max(x0 + b + 1), top.min(y), y + 1);
    }
    let mut cut = Pixmap::new(right.checked_sub(left)? as u32, bottom.checked_sub(top)? as u32)?;
    let cw = cut.width() as usize * 4;
    for (y, row) in cut.data_mut().chunks_exact_mut(cw).enumerate() {
        let from = ((top + y) * w + left) * 4;
        row.copy_from_slice(&bytes[from..from + cw]);
    }
    Some(Op::Laid { x: left as i32, y: top as i32, pixmap: cut })
}

pub(crate) fn damage(seen: &mut Seen, ops: &[Op], marks: &[Mark], size: (u32, u32)) -> Option<Vec<IntRect>> {
    let whole = IntRect::from_xywh(0, 0, size.0, size.1)?;
    let mut now = Seen::new();
    for (n, m) in marks.iter().enumerate() {
        let part = &ops[m.at.min(ops.len())..marks.get(n + 1).map_or(ops.len(), |m| m.at.min(ops.len()))];
        let kept = if part.iter().all(|op| matches!(op, Op::Laid { .. })) { extent(part) } else { Some(whole) };
        let Some(b) = m.bounds.or(kept).and_then(|b| b.intersect(&whole)) else { continue };
        let mut h = DefaultHasher::new();
        part.iter().for_each(|op| sign(op, &mut h));
        now.insert(m.key, (h.finish(), b));
    }
    let was = std::mem::replace(seen, now);
    if was.is_empty() || marks.is_empty() {
        return None;
    }
    let mut parts = Vec::new();
    for (k, &(s, b)) in seen.iter() {
        match was.get(k) {
            Some(&w) if w == (s, b) => {}
            Some(&(_, w)) => parts.extend([w, b]),
            None => parts.push(b),
        }
    }
    parts.extend(was.iter().filter(|(k, _)| !seen.contains_key(k)).map(|(_, &(_, b))| b));
    parts.sort_by_key(|r| (r.y(), r.x(), r.width(), r.height()));
    parts.dedup();
    let inside = |a: &IntRect, b: &IntRect| a != b && a.left() >= b.left() && a.top() >= b.top() && a.right() <= b.right() && a.bottom() <= b.bottom();
    let parts: Vec<IntRect> = parts.iter().filter(|a| !parts.iter().any(|b| inside(a, b))).copied().collect();
    let area: u64 = parts.iter().map(|r| u64::from(r.width()) * u64::from(r.height())).sum();
    (parts.len() <= MOST && area * 2 < u64::from(size.0) * u64::from(size.1)).then_some(parts)
}

fn extent(part: &[Op]) -> Option<IntRect> {
    part.iter().filter_map(|op| match op {
        Op::Laid { x, y, pixmap } => IntRect::from_xywh(*x, *y, pixmap.width(), pixmap.height()),
        _ => None,
    }).reduce(|a, b| IntRect::from_ltrb(a.left().min(b.left()), a.top().min(b.top()), a.right().max(b.right()), a.bottom().max(b.bottom())).unwrap_or(a))
}

fn floats(h: &mut DefaultHasher, v: &[f32]) {
    v.iter().for_each(|f| h.write_u32(f.to_bits()));
}

fn rect(h: &mut DefaultHasher, r: &Rect) {
    floats(h, &[r.x, r.y, r.w, r.h]);
}

fn colour(h: &mut DefaultHasher, c: &Colour) {
    floats(h, &[c.r, c.g, c.b, c.a]);
}

fn sign(op: &Op, h: &mut DefaultHasher) {
    match op {
        Op::Fill { r, colour: c } => {
            h.write_u8(0);
            rect(h, r);
            colour(h, c);
        }
        Op::Round { r, radii, colour: c } => {
            h.write_u8(1);
            rect(h, r);
            floats(h, radii);
            colour(h, c);
        }
        Op::Line { from, to, width, colour: c } => {
            h.write_u8(2);
            floats(h, &[from.0, from.1, to.0, to.1, *width]);
            colour(h, c);
        }
        Op::Path { path, how, colour: c } => sign_path(path, how, c, h),
        Op::Glyphs { run, colour: c, table } => sign_glyphs(run, c, table, h),
        Op::Image { id, x, y, .. } => {
            h.write_u8(5);
            h.write_u64(*id);
            h.write_i32(*x);
            h.write_i32(*y);
        }
        Op::Shadow { r, radius, falls, colour: c } => {
            h.write_u8(6);
            rect(h, r);
            h.write_u32(radius.to_bits());
            falls.iter().for_each(|f| floats(h, &[f.y, f.blur, f.spread, f.alpha]));
            colour(h, c);
        }
        Op::Inside { r, radius, fall: f, colour: c } => {
            h.write_u8(7);
            rect(h, r);
            floats(h, &[*radius, f.y, f.blur, f.spread, f.alpha]);
            colour(h, c);
        }
        Op::Push { at, opacity, radius } => {
            h.write_u8(8);
            [at.x(), at.y(), at.width() as i32, at.height() as i32].iter().for_each(|v| h.write_i32(*v));
            floats(h, &[*opacity, *radius]);
        }
        Op::Pop => h.write_u8(9),
        Op::Pixels { .. } => h.write_u64(super::new_id()),
        Op::Laid { x, y, pixmap } => {
            h.write_u8(11);
            [*x, *y, pixmap.width() as i32].iter().for_each(|v| h.write_i32(*v));
            h.write_u64(quick(pixmap.data()));
        }
    }
}

fn sign_path(path: &Path, how: &How, c: &Colour, h: &mut DefaultHasher) {
    h.write_u8(3);
    path.points().iter().for_each(|p| floats(h, &[p.x, p.y]));
    path.verbs().iter().for_each(|v| h.write_u8(*v as u8));
    match how {
        How::Fill(rule) => h.write_u8(*rule as u8),
        How::Stroke(s) => h.write(format!("{s:?}").as_bytes()),
    }
    colour(h, c);
}

fn sign_glyphs(run: &[Glyph], c: &Colour, table: &[u8; 256], h: &mut DefaultHasher) {
    h.write_u8(4);
    for g in run {
        g.key.hash(h);
        h.write_usize(std::rc::Rc::as_ptr(&g.mask) as usize);
        h.write_i32(g.x);
        h.write_i32(g.y);
    }
    colour(h, c);
    h.write(&table[..]);
}

fn quick(bytes: &[u8]) -> u64 {
    const K: u64 = 0x9E37_79B9_7F4A_7C15;
    let mix = |x: u64, w: u64| (x ^ w).wrapping_mul(K).rotate_left(31);
    let word = |w: &[u8]| u64::from_ne_bytes(w.try_into().unwrap_or_default());
    let mut lanes = [bytes.len() as u64, 1, 2, 3];
    let mut blocks = bytes.chunks_exact(32);
    for b in blocks.by_ref() {
        for (i, lane) in lanes.iter_mut().enumerate() {
            *lane = mix(*lane, word(&b[i * 8..i * 8 + 8]));
        }
    }
    let mut x = lanes.iter().fold(0, |x, l| mix(x, *l));
    let mut words = blocks.remainder().chunks_exact(8);
    x = words.by_ref().fold(x, |x, w| mix(x, word(w)));
    words.remainder().iter().fold(x, |x, b| mix(x, u64::from(*b)))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn at(x: i32, y: i32, w: u32, h: u32) -> IntRect {
        IntRect::from_xywh(x, y, w, h).unwrap()
    }

    #[test]
    fn what_pixels_drew_is_kept_cut_to_where_it_drew() {
        let mut p = Painter::default();
        p.push_layer(at(10, 10, 50, 40), 0.5);
        p.pixels(at(0, 0, 50, 40), |c| c.fill_rect(tiny_skia::Rect::from_xywh(5.0, 6.0, 3.0, 2.0).unwrap(), &crate::colour::paint(Colour { r: 1.0, g: 0.0, b: 0.0, a: 1.0 }), tiny_skia::Transform::identity(), None));
        p.pixels(at(0, 0, 50, 40), |_| {});
        p.pop();
        let ops = settle(p.ops, (100, 100), &mut None, &mut []);
        assert_eq!(ops.len(), 3, "the empty one is dropped");
        let Op::Laid { x, y, pixmap } = &ops[1] else { panic!("not laid") };
        assert_eq!((*x, *y, pixmap.width(), pixmap.height()), (5, 6, 3, 2));
    }

    #[test]
    fn a_pixels_op_that_drew_nothing_does_not_shift_the_spots_after_it() {
        let frame = |x: f32| {
            let mut p = Painter::default();
            for (key, left) in [(1, 0), (2, 100), (3, 200)] {
                p.marks.push(Mark { at: p.ops.len(), key, bounds: IntRect::from_xywh(left, 0, 50, 50) });
                if key == 1 {
                    p.pixels(at(0, 0, 50, 50), |_| {});
                } else {
                    let x = if key == 2 { x } else { 200.0 };
                    p.fill(Rect { x, y: 0.0, w: 50.0, h: 50.0 }, Colour { r: 0.5, g: 0.5, b: 0.5, a: 1.0 });
                }
            }
            p
        };
        let (mut seen, mut scratch) = (Seen::new(), None);
        assert!(changes(frame(100.0), (400, 300), &mut seen, &mut scratch).1.is_none(), "the first frame is all of it");
        let (ops, parts) = changes(frame(110.0), (400, 300), &mut seen, &mut scratch);
        assert_eq!(ops.len(), 2, "the empty one is dropped");
        assert_eq!(parts, Some(vec![at(100, 0, 50, 50)]), "the second spot changed, not the first");
    }

    #[test]
    fn a_letter_whose_mask_changed_under_the_same_key_is_damage() {
        use crate::sharp::{Hinting, MaskKey, Sharp};
        let (sharp, hinting) = (Sharp::default(), Hinting::default());
        let data = include_bytes!("../../share/fonts/NotoSans-Regular.ttf");
        let key = |size: f32| MaskKey { font: fontdb::ID::dummy(), glyph: 36, size: size.to_bits(), weight: 400, lean: false };
        let ink = Colour { r: 0.1, g: 0.2, b: 0.3, a: 1.0 };
        let frame = |size: f32| {
            let mut p = Painter::default();
            p.marks.push(Mark { at: 0, key: 1, bounds: IntRect::from_xywh(0, 0, 50, 50) });
            let mask = sharp.mask(data, 0, "Noto Sans", &hinting, key(size)).unwrap();
            p.glyphs(vec![crate::paint::Glyph { key: key(18.0), mask, x: 5, y: 30 }], ink, sharp.table(ink));
            p
        };
        let mut seen = Seen::new();
        let mut damage_of = |p: Painter| damage(&mut seen, &p.ops, &p.marks, (400, 300));
        assert_eq!(damage_of(frame(18.0)), None);
        assert_eq!(damage_of(frame(18.0)), Some(vec![]));
        assert_eq!(damage_of(frame(30.0)), Some(vec![at(0, 0, 50, 50)]));
    }

    #[test]
    fn only_the_spots_that_changed_are_damage_both_where_they_were_and_are() {
        let frame = |moved: f32, gone: bool| {
            let mut p = Painter::default();
            for (key, x) in [(1, 0.0), (2, 100.0 + moved), (3, 200.0)] {
                if gone && key == 3 {
                    continue;
                }
                p.marks.push(Mark { at: p.ops.len(), key, bounds: IntRect::from_xywh(x as i32, 0, 50, 50) });
                p.fill(Rect { x, y: 0.0, w: 50.0, h: 50.0 }, Colour { r: 0.5, g: 0.5, b: 0.5, a: 1.0 });
            }
            p
        };
        let mut seen = Seen::new();
        let mut damage_of = |p: Painter| damage(&mut seen, &p.ops, &p.marks, (400, 300));
        assert_eq!(damage_of(frame(0.0, false)), None, "the first frame is all of it");
        assert_eq!(damage_of(frame(0.0, false)), Some(vec![]));
        assert_eq!(damage_of(frame(10.0, false)), Some(vec![at(100, 0, 50, 50), at(110, 0, 50, 50)]));
        assert_eq!(damage_of(frame(10.0, true)), Some(vec![at(200, 0, 50, 50)]));
        assert_eq!(damage_of(Painter::default()), None, "nothing marked");
    }

    #[test]
    fn ops_that_are_not_pictures_in_a_mark_without_bounds_are_damage() {
        let frame = |x: f32| {
            let mut p = Painter::default();
            p.marks.push(Mark { at: 0, key: 1, bounds: IntRect::from_xywh(0, 0, 400, 300) });
            p.fill(Rect { x: 0.0, y: 0.0, w: 50.0, h: 50.0 }, Colour { r: 0.5, g: 0.5, b: 0.5, a: 1.0 });
            p.marks.push(Mark { at: p.ops.len(), key: 2, bounds: None });
            p.fill(Rect { x, y: 0.0, w: 20.0, h: 20.0 }, Colour { r: 0.5, g: 0.5, b: 0.5, a: 1.0 });
            p
        };
        let mut seen = Seen::new();
        let mut damage_of = |p: Painter| damage(&mut seen, &p.ops, &p.marks, (400, 300));
        assert_eq!(damage_of(frame(100.0)), None);
        assert_eq!(damage_of(frame(100.0)), Some(vec![]));
        assert_ne!(damage_of(frame(110.0)), Some(vec![]), "a moved box of the second mark is told");
    }

    #[test]
    fn pixels_in_rooms_of_different_shapes_share_one_scratch_picture() {
        let frame = |scratch: &mut Option<Pixmap>| {
            let mut p = Painter::default();
            for (room, hole) in [((400u32, 300u32), (3i32, 4i32)), ((400, 20), (5, 6)), ((200, 60), (7, 8)), ((400, 300), (9, 10))] {
                p.push_layer(at(0, 0, room.0, room.1), 1.0);
                p.pixels(at(0, 0, room.0, room.1), move |c| c.fill_rect(tiny_skia::Rect::from_xywh(hole.0 as f32, hole.1 as f32, 2.0, 2.0).unwrap(), &crate::colour::paint(Colour { r: 1.0, g: 0.0, b: 0.0, a: 1.0 }), tiny_skia::Transform::identity(), None));
                p.pop();
            }
            let ops = settle(p.ops, (500, 400), scratch, &mut []);
            ops.iter().filter_map(|o| if let Op::Laid { x, y, pixmap } = o { Some((*x, *y, pixmap.width(), pixmap.height())) } else { None }).collect::<Vec<_>>()
        };
        let mut scratch = None;
        let first = frame(&mut scratch);
        assert_eq!(first, [(3, 4, 2, 2), (5, 6, 2, 2), (7, 8, 2, 2), (9, 10, 2, 2)]);
        let kept = scratch.as_ref().map(|p| p.data().as_ptr());
        assert_eq!(frame(&mut scratch), first, "what an earlier, larger room left behind is not seen");
        assert_eq!(scratch.as_ref().map(|p| p.data().as_ptr()), kept, "no new picture");
    }

    #[test]
    fn the_quick_sum_tells_a_change_of_any_one_byte() {
        let base: Vec<u8> = (0..100u32).map(|i| (i * 7) as u8).collect();
        let sum = quick(&base);
        assert_eq!(sum, quick(&base.clone()));
        for i in 0..base.len() {
            let mut other = base.clone();
            other[i] ^= 1;
            assert_ne!(quick(&other), sum, "byte {i}");
        }
        assert_ne!(quick(&base[..99]), sum, "a shorter picture");
    }
}
