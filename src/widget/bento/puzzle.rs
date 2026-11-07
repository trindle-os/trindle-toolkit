//! The puzzle bento: tiles of whole twelfths locked into one rectangle with no gap straight across or down, packed the same way each time.

use super::{below, highest, join, lower_beside, lowest, straight, Luck, Rect, Strip, LOOSE};

pub(crate) const TWELFTHS: usize = 12;
const TRIES: usize = 3000;
const STRAIGHT: f32 = 400.0;
const BAR: f32 = 2.0;
const PAST_LOOSE: f32 = 4.0;
const ORDER: f32 = 6.0;
const FIRST: f32 = 600.0;
const GROWN: f32 = 40.0;

#[derive(Clone, Copy, Debug)]
pub(crate) struct Piece {
    pub least: usize,
    pub most: usize,
    pub gives: bool,
}

#[derive(Clone, Debug)]
pub(crate) struct Laid {
    pub rects: Vec<Rect>,
    pub seen: Vec<usize>,
    pub units: Vec<usize>,
    pub cost: f32,
}

fn edge(c: usize, width: f32, gap: f32) -> f32 {
    if c == TWELFTHS {
        return width + gap;
    }
    (c as f32 * (width + gap) / TWELFTHS as f32).round()
}

pub(crate) fn across(c: usize, u: usize, width: f32, gap: f32) -> (f32, f32) {
    let x = edge(c, width, gap);
    (x, edge(c + u, width, gap) - gap - x)
}

pub(crate) fn twelfths_for(points: f32, width: f32, gap: f32) -> usize {
    (1..=TWELFTHS).find(|&u| across(0, u, width, gap).1 >= points - 0.5).unwrap_or(TWELFTHS)
}

#[derive(Clone, Copy, Debug)]
struct Ledge {
    c: usize,
    u: usize,
    y: f32,
}

impl Strip for Ledge {
    fn y(&self) -> f32 {
        self.y
    }

    fn absorb(&mut self, other: &Self) {
        self.u += other.u;
    }
}

struct Try {
    at: Vec<Option<(usize, usize, f32, f32)>>,
    natural: Vec<f32>,
    ledges: Vec<Ledge>,
    seen: Vec<usize>,
    units: Vec<usize>,
}

impl Try {
    fn new(n: usize) -> Try {
        Try { at: vec![None; n], natural: vec![0.0; n], ledges: vec![Ledge { c: 0, u: TWELFTHS, y: 0.0 }], seen: Vec::new(), units: Vec::new() }
    }

    fn lift(&mut self, i: usize, to: f32, gap: f32) -> Option<()> {
        let l = self.ledges[i];
        let mut covered = 0;
        let mut above = Vec::new();
        for (t, a) in self.at.iter().enumerate() {
            if let Some((c, u, y, h)) = *a {
                if c >= l.c && c + u <= l.c + l.u && (y + h + gap - l.y).abs() < 0.5 {
                    covered += u;
                    above.push(t);
                }
            }
        }
        if covered < l.u || l.y < 0.5 {
            return None;
        }
        for t in above {
            if let Some(a) = self.at[t].as_mut() {
                a.3 += to - l.y;
            }
        }
        self.ledges[i].y = to;
        join(&mut self.ledges);
        Some(())
    }

    fn raise(&mut self, gap: f32) -> Option<()> {
        let i = lowest(&self.ledges);
        let to = lower_beside(&self.ledges, i)?;
        self.lift(i, to, gap)
    }

    fn place(&mut self, i: usize, t: usize, u: usize, h: f32, gap: f32) {
        let l = self.ledges[i];
        self.at[t] = Some((l.c, u, l.y, h));
        self.natural[t] = h;
        self.seen.push(t);
        self.units.push(u);
        let mut put = vec![Ledge { c: l.c, u, y: l.y + h + gap }];
        if l.u > u {
            put.push(Ledge { c: l.c + u, u: l.u - u, y: l.y });
        }
        self.ledges.splice(i..=i, put);
        join(&mut self.ledges);
    }

    fn finish(mut self, pieces: &[Piece], width: f32, gap: f32) -> Option<Laid> {
        let top = highest(&self.ledges);
        while let Some(i) = below(&self.ledges, top) {
            self.lift(i, top, gap)?;
        }
        let mut rects = Vec::with_capacity(pieces.len());
        let mut spare = 0.0;
        let mut grown = 0;
        let mut bars = 0.0;
        for (t, p) in pieces.iter().enumerate() {
            let (c, u, y, h) = self.at[t]?;
            let (x, w) = across(c, u, width, gap);
            rects.push(Rect { x, y, w, h });
            bars += (w - 3.0 * h).max(0.0) + (h - 3.0 * w).max(0.0);
            grown += u.saturating_sub(p.most);
            if !p.gives {
                let below = h - self.natural[t];
                spare += w * below + PAST_LOOSE * w * (below - LOOSE).max(0.0);
            }
        }
        let moved: usize = self.seen.iter().enumerate().map(|(k, &t)| k.abs_diff(t)).sum();
        let cost = top - gap + spare / width.max(1.0) + STRAIGHT * straight(&rects, gap) as f32
            + ORDER * moved as f32 + GROWN * grown as f32 + BAR * bars
            + if rects.first().is_some_and(|r| r.x > 0.5 || r.y > 0.5) { FIRST } else { 0.0 };
        Some(Laid { rects, seen: self.seen, units: self.units, cost })
    }
}

fn fits(p: &Piece, room: usize, others: Option<usize>) -> Vec<usize> {
    (p.least..=room)
        .filter(|&u| {
            let rest = room - u;
            rest == 0 || (others.is_some_and(|o| rest >= o) && u <= p.most)
        })
        .collect()
}

pub(crate) fn pack(pieces: &[Piece], width: f32, gap: f32, height: &dyn Fn(usize, f32) -> f32) -> Option<Laid> {
    let n = pieces.len();
    if n == 0 {
        return None;
    }
    let mut luck = Luck::new();
    let mut best: Option<Laid> = None;
    for way in 0..TRIES {
        let order = stirred(n, way, &mut luck);
        let Some(tr) = fill(pieces, order, way, &mut luck, width, gap, height) else { continue };
        if let Some(laid) = tr.finish(pieces, width, gap) {
            if best.as_ref().is_none_or(|b| laid.cost < b.cost - 1e-3) {
                best = Some(laid);
            }
        }
    }
    best
}

fn stirred(n: usize, way: usize, luck: &mut Luck) -> Vec<usize> {
    let mut order: Vec<usize> = (0..n).collect();
    if way > 0 && luck.roll(3) > 0 {
        let most = if luck.roll(2) == 0 { 2 } else { n };
        let swaps = 1 + luck.roll(most);
        for _ in 0..swaps {
            let (a, b) = (luck.roll(n), luck.roll(n));
            order.swap(a, b);
        }
    }
    order
}

fn fill(pieces: &[Piece], order: Vec<usize>, way: usize, luck: &mut Luck, width: f32, gap: f32, height: &dyn Fn(usize, f32) -> f32) -> Option<Try> {
    let mut tr = Try::new(pieces.len());
    let mut left = order;
    while !left.is_empty() {
        let i = lowest(&tr.ledges);
        let room = tr.ledges[i].u;
        let mut pick = None;
        for (k, &t) in left.iter().enumerate() {
            let others = left.iter().filter(|&&o| o != t).map(|&o| pieces[o].least).min();
            let ws = fits(&pieces[t], room, others);
            if ws.is_empty() {
                continue;
            }
            let liked: Vec<usize> = ws.iter().copied().filter(|&u| u <= pieces[t].most).collect();
            let u = if way == 0 {
                liked.last().copied().unwrap_or(ws[0])
            } else if !liked.is_empty() && luck.roll(6) > 0 {
                liked[luck.roll(liked.len())]
            } else {
                ws[luck.roll(ws.len())]
            };
            pick = Some((k, t, u));
            break;
        }
        match pick {
            Some((k, t, u)) => {
                let c = tr.ledges[i].c;
                let h = height(t, across(c, u, width, gap).1);
                tr.place(i, t, u, h, gap);
                left.remove(k);
            }
            None => tr.raise(gap)?,
        }
    }
    Some(tr)
}

pub(crate) fn replay(pieces: &[Piece], seen: &[usize], units: &[usize], width: f32, gap: f32, height: &dyn Fn(usize, f32) -> f32) -> Option<Laid> {
    let mut once = vec![false; pieces.len()];
    if seen.len() != pieces.len() || units.len() != pieces.len() || seen.iter().any(|&t| t >= once.len() || std::mem::replace(&mut once[t], true)) {
        return None;
    }
    let mut tr = Try::new(pieces.len());
    for (&t, &u) in seen.iter().zip(units) {
        loop {
            let i = lowest(&tr.ledges);
            if tr.ledges[i].u >= u {
                let c = tr.ledges[i].c;
                tr.place(i, t, u, height(t, across(c, u, width, gap).1), gap);
                break;
            }
            tr.raise(gap)?;
        }
    }
    tr.finish(pieces, width, gap)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn piece(least: usize, most: usize) -> Piece {
        Piece { least, most, gives: false }
    }

    fn holes(rects: &[Rect], width: f32, gap: f32) -> bool {
        let tall = rects.iter().map(|r| r.y + r.h).fold(0.0, f32::max);
        let mut y = 0.5;
        while y < tall {
            let mut on: Vec<&Rect> = rects.iter().filter(|r| r.y <= y && r.y + r.h > y).collect();
            on.sort_by(|a, b| a.x.total_cmp(&b.x));
            let row: f32 = on.iter().map(|r| r.w).sum::<f32>() + gap * on.len().saturating_sub(1) as f32;
            if on.is_empty() || (row - width).abs() > 0.5 {
                let between = rects.iter().any(|r| r.y + r.h <= y && r.y + r.h + gap > y);
                if !between {
                    return true;
                }
            }
            y += 1.0;
        }
        false
    }

    fn overlap(rects: &[Rect]) -> bool {
        rects.iter().enumerate().any(|(i, a)| rects[i + 1..].iter().any(|b| {
            a.x < b.x + b.w - 0.5 && b.x < a.x + a.w - 0.5 && a.y < b.y + b.h - 0.5 && b.y < a.y + a.h - 0.5
        }))
    }

    #[test]
    fn twelfths_meet_the_edges_exactly() {
        for width in [900.0, 1001.0, 1333.0] {
            let (x, w) = across(4, 8, width, 12.0);
            assert_eq!(x + w, width);
            let (a, wa) = across(0, 4, width, 12.0);
            assert_eq!(a + wa + 12.0, x);
        }
    }

    #[test]
    fn tiles_of_different_heights_lock_together_into_one_rectangle() {
        let grows = |least, most| Piece { least, most, gives: true };
        let pieces = vec![grows(5, 7), grows(2, 3), grows(2, 3), piece(4, 6), piece(4, 6), piece(4, 6), piece(4, 6)];
        let tall = [150.0, 90.0, 90.0, 260.0, 180.0, 220.0, 140.0];
        let h = |t: usize, w: f32| tall[t] * 300.0 / w.max(100.0);
        let laid = pack(&pieces, 1000.0, 12.0, &h).expect("a puzzle closes");
        assert!(!overlap(&laid.rects));
        assert!(!holes(&laid.rects, 1000.0, 12.0), "{:?}", laid.rects);
        let right = laid.rects.iter().map(|r| r.x + r.w).fold(0.0, f32::max);
        assert_eq!(right, 1000.0);
        let bottom: Vec<f32> = laid.rects.iter().map(|r| r.y + r.h).collect();
        let tallest = bottom.iter().copied().fold(0.0, f32::max);
        for c in 0..TWELFTHS {
            let (x, _) = across(c, 1, 1000.0, 12.0);
            let low = laid.rects.iter().filter(|r| r.x <= x + 0.5 && r.x + r.w > x + 0.5).map(|r| r.y + r.h).fold(0.0, f32::max);
            assert_eq!(low, tallest, "twelfth {c}");
        }
        assert!(straight(&laid.rects, 12.0) <= 1, "{:?}", laid.rects);
    }

    #[test]
    fn the_order_seen_is_top_first_then_left_to_right() {
        let pieces = vec![piece(4, 6), piece(4, 6), piece(4, 6), piece(4, 6), piece(4, 6)];
        let tall = [200.0, 120.0, 160.0, 90.0, 140.0];
        let h = |t: usize, _: f32| tall[t];
        let laid = pack(&pieces, 1000.0, 12.0, &h).expect("closes");
        for p in laid.seen.windows(2) {
            let (a, b) = (laid.rects[p[0]], laid.rects[p[1]]);
            assert!(a.y < b.y + 0.5, "{a:?} then {b:?}");
        }
        assert_eq!(laid.seen[0], 0);
        assert_eq!((laid.rects[0].x, laid.rects[0].y), (0.0, 0.0));
    }

    #[test]
    fn tiles_that_cannot_sit_side_by_side_stack() {
        let pieces = vec![piece(8, 12), piece(9, 12), piece(7, 12)];
        let h = |_: usize, _: f32| 100.0;
        let laid = pack(&pieces, 400.0, 12.0, &h).expect("stacks");
        assert!(laid.rects.iter().all(|r| r.x == 0.0 && r.w == 400.0));
        assert_eq!(laid.seen, vec![0, 1, 2]);
    }

    #[test]
    fn replaying_a_packing_places_it_the_same() {
        let pieces = vec![piece(2, 3), piece(4, 6), piece(4, 6), piece(3, 5), piece(4, 6)];
        let tall = [80.0, 210.0, 150.0, 120.0, 170.0];
        let h = |t: usize, w: f32| (tall[t] * 320.0 / w).round();
        let laid = pack(&pieces, 1100.0, 12.0, &h).expect("closes");
        let again = replay(&pieces, &laid.seen, &laid.units, 1100.0, 12.0, &h).expect("replays");
        assert_eq!(format!("{:?}", laid.rects), format!("{:?}", again.rects));
    }
}
