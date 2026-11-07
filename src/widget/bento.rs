//! Bento packing: tiles of different sizes filling one clean rectangle, in bands or as a puzzle, never changing the order given.

use std::cell::RefCell;
use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::rc::Rc;

use super::{Lay, Length, Node, Rect, What};

mod puzzle;

fn hashed(feed: impl FnOnce(&mut DefaultHasher)) -> u64 {
    let mut h = DefaultHasher::new();
    feed(&mut h);
    h.finish()
}

pub(crate) const LOOSE: f32 = 12.0;
#[cfg(any(test, feature = "testing"))]
const GROWN_MOST: f32 = 0.45;
const QUIET_OVER: f32 = 0.25;
const MOST_IN_BAND: usize = 8;
const MOST_STACKED: usize = 4;
const MOST_MEASURED: usize = 16;
const SAMPLES: usize = 5;
const OVER_LOOSE: f32 = 1000.0;

#[derive(Clone, Debug)]
pub(crate) struct Size {
    pub lo: f32,
    pub hi: f32,
    pub at: Vec<(f32, f32)>,
    pub gives: bool,
}

impl Size {
    fn h(&self, w: f32) -> f32 {
        let at = &self.at;
        if w <= at[0].0 {
            return at[0].1;
        }
        for p in at.windows(2) {
            let ((w0, h0), (w1, h1)) = (p[0], p[1]);
            if w <= w1 {
                let t = (1.0 / w0 - 1.0 / w) / (1.0 / w0 - 1.0 / w1);
                return h0 + t * (h1 - h0);
            }
        }
        at[at.len() - 1].1
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Column {
    pub first: usize,
    pub count: usize,
    pub x: f32,
    pub w: f32,
}

#[derive(Debug, Default)]
pub(crate) struct Packed {
    pub rects: Vec<Rect>,
    pub height: f32,
    pub seen: Vec<usize>,
}

impl Packed {
    pub(crate) fn seen(&self) -> Vec<usize> {
        if self.seen.len() == self.rects.len() { self.seen.clone() } else { (0..self.rects.len()).collect() }
    }
}

#[cfg(any(test, feature = "testing"))]
#[derive(Clone, Debug)]
pub(crate) struct Placed {
    pub path: Vec<usize>,
    pub rect: Rect,
}

const MOST_HELD: usize = 3;

#[derive(Clone, Debug)]
pub(crate) struct Held {
    cards: u64,
    edges: Vec<(f32, f32)>,
    over: Vec<Vec<usize>>,
    under: Vec<Vec<usize>>,
    down: Vec<usize>,
    seen: Vec<usize>,
    boxy: bool,
}

impl Held {
    fn of(rects: &[Rect], seen: Vec<usize>, width: f32, gap: f32, cards: u64, boxy: bool) -> Held {
        let whole = (width + gap).max(1.0);
        let n = rects.len();
        let beside = |a: &Rect, b: &Rect| a.x < b.x + b.w - 0.5 && b.x < a.x + a.w - 0.5;
        let mut over = vec![Vec::new(); n];
        let mut under = vec![Vec::new(); n];
        for t in 0..n {
            for s in 0..n {
                if s != t && beside(&rects[s], &rects[t]) && rects[s].y + rects[s].h <= rects[t].y + 0.5 {
                    over[t].push(s);
                    under[s].push(t);
                }
            }
        }
        let mut down: Vec<usize> = (0..n).collect();
        down.sort_by(|&a, &b| rects[a].y.total_cmp(&rects[b].y).then(rects[a].x.total_cmp(&rects[b].x)));
        let edges = rects.iter().map(|r| (r.x / whole, (r.x + r.w + gap) / whole)).collect();
        Held { cards, edges, over, under, down, seen, boxy }
    }

    fn across(&self, width: f32, gap: f32) -> Vec<(f32, f32)> {
        let whole = width + gap;
        self.edges.iter().map(|&(l, r)| {
            let x = (l * whole).round();
            let end = if r > 1.0 - 1e-4 { whole } else { (r * whole).round() };
            (x, (end - gap - x).max(0.0))
        }).collect()
    }
}

#[derive(Default)]
pub(crate) struct Kept {
    held: RefCell<HashMap<String, Vec<Held>>>,
}

impl Kept {
    fn get(&self, name: &str, cards: u64) -> Option<Held> {
        self.held.borrow().get(name)?.iter().find(|h| h.cards == cards).cloned()
    }

    fn put(&self, name: &str, h: Held) {
        let mut held = self.held.borrow_mut();
        let all = held.entry(name.to_string()).or_default();
        all.retain(|o| o.cards != h.cards);
        all.insert(0, h);
        all.truncate(MOST_HELD);
    }

    pub(crate) fn keep_only(&self, names: &[String]) {
        self.held.borrow_mut().retain(|name, _| names.contains(name));
    }

    #[cfg(test)]
    pub fn len(&self) -> usize {
        self.held.borrow().values().map(Vec::len).sum()
    }
}

#[derive(Default)]
pub(crate) struct Memo {
    packed: RefCell<HashMap<(usize, u32), Rc<Packed>>>,
    #[cfg(any(test, feature = "testing"))]
    log: RefCell<Option<Vec<Placed>>>,
}

#[cfg(any(test, feature = "testing"))]
impl Memo {
    pub(crate) fn logging(&self) -> bool {
        self.log.borrow().is_some()
    }

    pub(crate) fn note(&self, path: Vec<usize>, rect: Rect) {
        if let Some(log) = self.log.borrow_mut().as_mut() {
            log.push(Placed { path, rect });
        }
    }
}

pub(crate) fn plan(sizes: &[Size], width: f32, gap: f32, height: &dyn Fn(usize, f32) -> f32) -> Vec<Vec<Column>> {
    let n = sizes.len();
    let mut best: Vec<Option<(f32, usize, Vec<Column>)>> = vec![None; n + 1];
    best[0] = Some((0.0, 0, Vec::new()));
    for j in 1..=n {
        for i in (0..j).rev() {
            let k = j - i;
            let flat = sizes[i..j].iter().map(|s| s.lo).sum::<f32>() + gap * (k - 1) as f32;
            if k > MOST_IN_BAND || (k > MOST_STACKED && flat > width + 0.5) {
                break;
            }
            let Some(before) = best[i].as_ref().map(|b| b.0) else { continue };
            let Some((cost, band)) = best_band(sizes, i, j, width, gap, height) else { continue };
            let total = before + cost + if i > 0 { gap } else { 0.0 };
            if best[j].as_ref().is_none_or(|b| total < b.0 - 1e-3) {
                best[j] = Some((total, i, band));
            }
        }
    }
    let mut bands = Vec::new();
    let mut j = n;
    while j > 0 {
        let (_, i, band) = best[j].take().expect("a single tile always makes a band");
        bands.push(band);
        j = i;
    }
    bands.reverse();
    bands
}

fn best_band(sizes: &[Size], i: usize, j: usize, width: f32, gap: f32, height: &dyn Fn(usize, f32) -> f32) -> Option<(f32, Vec<Column>)> {
    let k = j - i;
    if k == 1 {
        return band(sizes, &[(i, 1)], width, gap, height);
    }
    let all = (1u32 << (k - 1)) - 1;
    let masks: Vec<u32> = if k <= MOST_STACKED { (1..=all).collect() } else { vec![all] };
    masks.into_iter()
        .filter_map(|mask| {
            let mut cols = Vec::new();
            let mut first = i;
            for b in 0..k {
                if b == k - 1 || mask & (1 << b) != 0 {
                    cols.push((first, i + b + 1 - first));
                    first = i + b + 1;
                }
            }
            band(sizes, &cols, width, gap, height)
        })
        .min_by(|a, b| a.0.total_cmp(&b.0))
}

struct Col<'a> {
    tiles: &'a [Size],
    lo: f32,
    hi: f32,
    gap: f32,
}

impl Col<'_> {
    fn new(tiles: &[Size], gap: f32) -> Col<'_> {
        let lo = tiles.iter().map(|t| t.lo).fold(0.0, f32::max);
        Col { tiles, lo, hi: tiles.iter().map(|t| t.hi).fold(lo, f32::max), gap }
    }

    fn h(&self, w: f32) -> f32 {
        self.tiles.iter().map(|t| t.h(w)).sum::<f32>() + self.gap * (self.tiles.len() - 1) as f32
    }

    fn gives(&self) -> bool {
        self.tiles.iter().any(|t| t.gives)
    }
}

fn band(sizes: &[Size], cols: &[(usize, usize)], width: f32, gap: f32, height: &dyn Fn(usize, f32) -> f32) -> Option<(f32, Vec<Column>)> {
    let cs: Vec<Col> = cols.iter().map(|&(f, c)| Col::new(&sizes[f..f + c], gap)).collect();
    let room = width - gap * (cs.len() - 1) as f32;
    let lo: Vec<f32> = cs.iter().map(|c| c.lo).collect();
    let hi: Vec<f32> = cs.iter().map(|c| c.hi).collect();
    if cs.len() > 1 && lo.iter().sum::<f32>() > room + 0.5 {
        return None;
    }
    let gives: Vec<bool> = cs.iter().map(Col::gives).collect();
    let widths = share(&lo, &hi, &gives, room, (24, 14), |c, w| cs[c].h(w));
    let columns = columns(cols, &widths, width, gap);
    let hs: Vec<f32> = columns.iter()
        .map(|col| (col.first..col.first + col.count).map(|t| height(t, col.w)).sum::<f32>() + gap * (col.count - 1) as f32)
        .collect();
    let tall = hs.iter().copied().fold(0.0, f32::max);
    let mut spare = 0.0;
    for ((c, col), h) in cs.iter().zip(&columns).zip(&hs) {
        spare += (col.first..col.first + col.count)
            .filter(|&t| !sizes[t].gives)
            .map(|t| (col.w - sizes[t].hi).max(0.0) * height(t, col.w))
            .sum::<f32>();
        if c.gives() {
            continue;
        }
        let below = tall - h;
        let each = below / c.tiles.len() as f32;
        spare += col.w * below + OVER_LOOSE * col.w * c.tiles.len() as f32 * (each - LOOSE).max(0.0);
    }
    Some((tall + spare / width.max(1.0), columns))
}

fn share(lo: &[f32], hi: &[f32], gives: &[bool], room: f32, steps: (usize, usize), h: impl Fn(usize, f32) -> f32) -> Vec<f32> {
    let n = lo.len();
    if n == 1 {
        return vec![room];
    }
    let mut w = if hi.iter().sum::<f32>() <= room {
        hi.to_vec()
    } else {
        let need = |c: usize, top: f32| -> f32 {
            if h(c, lo[c]) <= top {
                return lo[c];
            }
            let (mut a, mut b) = (lo[c], hi[c]);
            for _ in 0..steps.1 {
                let m = (a + b) / 2.0;
                if h(c, m) <= top { b = m } else { a = m }
            }
            b
        };
        let mut low = (0..n).map(|c| h(c, hi[c])).fold(0.0, f32::max);
        let mut high = (0..n).map(|c| h(c, lo[c])).fold(0.0, f32::max).max(low);
        for _ in 0..steps.0 {
            let mid = (low + high) / 2.0;
            if (0..n).map(|c| need(c, mid)).sum::<f32>() <= room { high = mid } else { low = mid }
        }
        (0..n).map(|c| need(c, high)).collect()
    };
    let takers = gives.iter().filter(|g| **g).count();
    let shares = if takers > 0 { takers } else { n } as f32;
    let left = ((room - w.iter().sum::<f32>()) / shares).max(0.0);
    for (x, g) in w.iter_mut().zip(gives) {
        if takers == 0 || *g {
            *x += left;
        }
    }
    w
}

fn columns(cols: &[(usize, usize)], widths: &[f32], width: f32, gap: f32) -> Vec<Column> {
    let mut x = 0.0;
    let mut out = Vec::with_capacity(cols.len());
    for (c, (&(first, count), w)) in cols.iter().zip(widths).enumerate() {
        let w = if c + 1 == cols.len() { (width - x).max(0.0) } else { w.round().max(0.0) };
        out.push(Column { first, count, x, w });
        x += w + gap;
    }
    out
}

const STRAIGHT: f32 = 60.0;
const WIDE: f32 = 3.0;
const MOST_FOUND: usize = 256;

thread_local! {
    static LAID: RefCell<HashMap<u64, Option<(Vec<usize>, Vec<usize>)>>> = RefCell::new(HashMap::new());
    static FOUND: RefCell<HashMap<u64, Option<Vec<f32>>>> = RefCell::new(HashMap::new());
}

const TRIES: usize = 4000;

fn spare_in(r: &Rect, h: f32, s: &Size) -> f32 {
    if s.gives {
        return 0.0;
    }
    let below = (r.h - h).max(0.0);
    WIDE * (r.w - s.hi).max(0.0) * h + r.w * below + OVER_LOOSE * r.w * (below - LOOSE).max(0.0)
}

fn cost(rects: &[Rect], natural: &[f32], sizes: &[Size], width: f32, gap: f32) -> f32 {
    let tall = rects.iter().map(|r| r.y + r.h).fold(0.0, f32::max);
    let spare: f32 = rects.iter().zip(natural).zip(sizes).map(|((r, &h), s)| spare_in(r, h, s)).sum();
    tall + spare / width.max(1.0) + STRAIGHT * straight(rects, gap) as f32
}

pub(crate) fn straight(rects: &[Rect], gap: f32) -> usize {
    let lines = |starts: Vec<f32>, crosses: &dyn Fn(&Rect, f32) -> bool| -> usize {
        let mut starts: Vec<f32> = starts.into_iter().filter(|v| *v > 0.5).collect();
        starts.sort_by(f32::total_cmp);
        starts.dedup_by(|a, b| (*a - *b).abs() < 0.5);
        starts.iter().filter(|&&v| !rects.iter().any(|r| crosses(r, v - gap / 2.0))).count()
    };
    lines(rects.iter().map(|r| r.y).collect(), &|r, m| r.y < m && r.y + r.h > m)
        + lines(rects.iter().map(|r| r.x).collect(), &|r, m| r.x < m && r.x + r.w > m)
}

#[derive(Clone, Copy, Debug)]
struct Ledge {
    x: f32,
    w: f32,
    y: f32,
}

trait Strip {
    fn y(&self) -> f32;
    fn absorb(&mut self, other: &Self);
}

impl Strip for Ledge {
    fn y(&self) -> f32 {
        self.y
    }

    fn absorb(&mut self, other: &Self) {
        self.w += other.w;
    }
}

fn lowest<L: Strip>(ledges: &[L]) -> usize {
    let mut i = 0;
    for (k, l) in ledges.iter().enumerate() {
        if l.y() < ledges[i].y() - 0.5 {
            i = k;
        }
    }
    i
}

fn join<L: Strip>(ledges: &mut Vec<L>) {
    let mut out: Vec<L> = Vec::with_capacity(ledges.len());
    for l in ledges.drain(..) {
        match out.last_mut() {
            Some(p) if (p.y() - l.y()).abs() < 0.5 => p.absorb(&l),
            _ => out.push(l),
        }
    }
    *ledges = out;
}

fn lower_beside<L: Strip>(ledges: &[L], i: usize) -> Option<f32> {
    [i.checked_sub(1), Some(i + 1)].into_iter().flatten().filter_map(|k| ledges.get(k).map(Strip::y)).reduce(f32::min)
}

fn highest<L: Strip>(ledges: &[L]) -> f32 {
    ledges.iter().map(Strip::y).fold(0.0, f32::max)
}

fn below<L: Strip>(ledges: &[L], top: f32) -> Option<usize> {
    ledges.iter().position(|l| l.y() < top - 0.5)
}

struct Luck(u32);

impl Luck {
    fn new() -> Luck {
        Luck(7)
    }

    fn roll(&mut self, below: usize) -> usize {
        self.0 = self.0.wrapping_mul(1_103_515_245).wrapping_add(12_345);
        (self.0 >> 16) as usize % below.max(1)
    }
}

#[derive(Clone)]
struct Try {
    rects: Vec<Rect>,
    natural: Vec<f32>,
    ledges: Vec<Ledge>,
}

impl Try {
    fn lift(&mut self, i: usize, to: f32, gap: f32) -> Option<()> {
        let l = self.ledges[i];
        let above: Vec<usize> = (0..self.rects.len())
            .filter(|&k| {
                let r = self.rects[k];
                (r.y + r.h + gap - l.y).abs() < 0.5 && r.x > l.x - 0.5 && r.x + r.w + gap < l.x + l.w + 0.5
            })
            .collect();
        let covered: f32 = above.iter().map(|&k| self.rects[k].w + gap).sum();
        if covered < l.w - 0.5 {
            return None;
        }
        for k in above {
            self.rects[k].h += to - l.y;
        }
        self.ledges[i].y = to;
        join(&mut self.ledges);
        Some(())
    }

    fn room(&mut self, lo: f32, gap: f32) -> Option<usize> {
        loop {
            let i = lowest(&self.ledges);
            if self.ledges[i].w - gap > lo - 0.5 {
                return Some(i);
            }
            let to = lower_beside(&self.ledges, i)?;
            self.lift(i, to, gap)?;
        }
    }

    fn place(&mut self, i: usize, w: f32, h: f32, gap: f32) {
        let l = self.ledges[i];
        self.rects.push(Rect { x: l.x, y: l.y, w, h });
        self.natural.push(h);
        let mut put = vec![Ledge { x: l.x, w: w + gap, y: l.y + h + gap }];
        if l.w - w - gap > 0.5 {
            put.push(Ledge { x: l.x + w + gap, w: l.w - w - gap, y: l.y });
        }
        self.ledges.splice(i..=i, put);
        join(&mut self.ledges);
    }

    fn level(&mut self, gap: f32) -> Option<()> {
        let top = highest(&self.ledges);
        while let Some(i) = below(&self.ledges, top) {
            self.lift(i, top, gap)?;
        }
        Some(())
    }
}

fn widths(tr: &Try, i: usize, t: usize, sizes: &[Size], gap: f32, seed: f32, height: &dyn Fn(usize, f32) -> f32) -> Vec<f32> {
    let l = tr.ledges[i];
    let room = l.w - gap;
    let s = &sizes[t];
    let next = sizes.get(t + 1);
    let mut ws = Vec::with_capacity(12);
    let mut levels: Vec<f32> = tr.ledges.iter().map(|o| o.y).filter(|&y| y > l.y + gap + 0.5).collect();
    levels.sort_by(f32::total_cmp);
    levels.dedup_by(|a, b| (*a - *b).abs() < 0.5);
    for y in levels {
        let target = y - l.y - gap;
        if height(t, room) > target {
            continue;
        }
        let (mut a, mut b) = (s.lo.min(room), room);
        if height(t, a) <= target {
            b = a;
        }
        while b - a > 2.0 {
            let m = ((a + b) / 2.0).round();
            if height(t, m) <= target { b = m } else { a = m }
        }
        if target - height(t, b) <= LOOSE {
            ws.push(b);
        }
    }
    ws.extend([seed, room, s.hi]);
    if let Some(n) = next {
        ws.extend([room - gap - n.hi, room - gap - n.lo]);
    }
    ws.push(s.lo);
    ws.extend((1..4).map(|k| s.lo + (room - s.lo) * k as f32 / 4.0));
    let fits = |w: f32| {
        let left = room - w;
        w > s.lo - 0.5 && left > -0.5 && (left < 0.5 || next.is_some_and(|n| left - gap > n.lo - 0.5))
    };
    let mut out: Vec<f32> = Vec::with_capacity(ws.len());
    for w in ws {
        let w = if (w - room).abs() < 0.5 { room } else { w.round() };
        if fits(w) && !out.iter().any(|o| (o - w).abs() < 1.0) {
            out.push(w);
        }
    }
    out
}

fn least(tr: &Try, rest: f32, sizes: &[Size], width: f32, gap: f32) -> f32 {
    let used: f32 = tr.rects.iter().map(|r| (r.w + gap) * (r.h + gap)).sum();
    let top = highest(&tr.ledges);
    let spare: f32 = tr.rects.iter().zip(&tr.natural).zip(sizes).map(|((r, &h), s)| spare_in(r, h, s)).sum();
    top.max((used + rest) / (width + gap)) - gap + spare / width.max(1.0)
}

fn puzzle(sizes: &[Size], width: f32, gap: f32, height: &dyn Fn(usize, f32) -> f32, seed: &[f32], mut bound: f32) -> Option<Vec<f32>> {
    let n = sizes.len();
    if n == 0 {
        return None;
    }
    let mut after = vec![0.0; n + 1];
    for t in (0..n).rev() {
        let s = &sizes[t];
        let least = [s.lo, s.hi].iter().map(|&w| (w + gap) * (s.h(w) + gap)).fold(f32::MAX, f32::min);
        after[t] = after[t + 1] + least;
    }
    let mut luck = Luck::new();
    let mut best = None;
    let mut tries = 0;
    let mut way = 0;
    'ways: while tries < TRIES {
        let mut tr = Try { rects: Vec::with_capacity(n), natural: Vec::with_capacity(n), ledges: vec![Ledge { x: 0.0, w: width + gap, y: 0.0 }] };
        let mut chosen = Vec::with_capacity(n);
        for t in 0..n {
            tries += 1;
            let Some(i) = tr.room(sizes[t].lo, gap) else { break };
            let ws = widths(&tr, i, t, sizes, gap, seed[t], height);
            if ws.is_empty() {
                break;
            }
            let pick = if way == 0 || luck.roll(2) == 0 { 0 } else { luck.roll(ws.len()) };
            tr.place(i, ws[pick], height(t, ws[pick]), gap);
            chosen.push(ws[pick]);
            if least(&tr, after[t + 1], sizes, width, gap) >= bound {
                way += 1;
                continue 'ways;
            }
        }
        way += 1;
        if chosen.len() < n || tr.level(gap).is_none() {
            continue;
        }
        let c = cost(&tr.rects, &tr.natural, sizes, width, gap);
        if c < bound {
            bound = c;
            best = Some(chosen);
        }
    }
    best
}

fn replay(sizes: &[Size], widths: &[f32], width: f32, gap: f32, height: &dyn Fn(usize, f32) -> f32) -> Option<(Vec<Rect>, Vec<f32>)> {
    if widths.len() != sizes.len() {
        return None;
    }
    let mut tr = Try { rects: Vec::with_capacity(sizes.len()), natural: Vec::with_capacity(sizes.len()), ledges: vec![Ledge { x: 0.0, w: width + gap, y: 0.0 }] };
    for (t, &w) in widths.iter().enumerate() {
        let i = tr.room(sizes[t].lo, gap)?;
        if w > tr.ledges[i].w - gap + 0.5 {
            return None;
        }
        tr.place(i, w, height(t, w), gap);
    }
    tr.level(gap)?;
    Some((tr.rects, tr.natural))
}

fn gives<M>(n: &Node<M>) -> bool {
    match &n.what {
        What::Drawing { .. } => n.height == Length::Fill,
        What::Card(inner, _) => gives(inner),
        What::Layers(items) => items.iter().any(gives),
        What::Column { items, .. } => items.iter().any(|i| i.height == Length::Fill && gives(i)),
        _ => false,
    }
}

impl Lay<'_> {
    pub(super) fn packed<M>(&self, items: &[Node<M>], min: f32, gap: f32, width: f32, puzzle: bool) -> Rc<Packed> {
        let key = (items.as_ptr() as usize, width.to_bits());
        if let Some(p) = self.bento.packed.borrow().get(&key) {
            return p.clone();
        }
        let outer = self.natural.replace(false);
        let p = if width < 1.0 {
            Packed { rects: vec![Rect::default(); items.len()], height: 0.0, seen: Vec::new() }
        } else if puzzle {
            self.pack_held(items, min, gap, width)
        } else {
            self.pack(items, min, gap, width)
        };
        let p = Rc::new(p);
        self.natural.set(outer);
        self.bento.packed.borrow_mut().insert(key, p.clone());
        p
    }

    pub(super) fn packed_natural<M>(&self, items: &[Node<M>], min: f32, gap: f32, max_w: f32) -> f32 {
        let least = self.px(min).max(1.0).min(max_w.max(0.0));
        let sum: f32 = items.iter().map(|t| self.natural_width(t, max_w).max(least)).sum();
        (sum + self.px(gap) * items.len().saturating_sub(1) as f32).min(max_w.max(0.0))
    }

    fn natural_width<M>(&self, t: &Node<M>, max_w: f32) -> f32 {
        let outer = self.natural.replace(true);
        let w = match t.width {
            Length::Fixed(p) => self.px(p),
            _ => self.measure_what(t, max_w).0,
        };
        self.natural.set(outer);
        w
    }

    fn size<M>(&self, t: &Node<M>, least: f32, width: f32) -> Size {
        let lo = self.natural_width(t, least).max(least).min(width);
        let hi = self.natural_width(t, width).max(lo).min(width);
        let mut at: Vec<(f32, f32)> = Vec::with_capacity(SAMPLES);
        for s in 0..SAMPLES {
            let f = s as f32 / (SAMPLES - 1) as f32;
            let w = (1.0 / (1.0 / lo.max(1.0) + f * (1.0 / hi.max(1.0) - 1.0 / lo.max(1.0)))).round();
            if at.last().is_none_or(|l| w > l.0 + 0.5) {
                at.push((w, self.measure(t, w).1));
            }
        }
        Size { lo, hi, at, gives: gives(t) }
    }

    fn column_height<M>(&self, items: &[Node<M>], col: &Column, w: f32, gap: f32) -> f32 {
        items[col.first..col.first + col.count].iter().map(|t| self.measure(t, w).1).sum::<f32>() + gap * (col.count - 1) as f32
    }

    fn pack<M>(&self, items: &[Node<M>], min: f32, gap: f32, width: f32) -> Packed {
        let gap = self.px(gap);
        let least = self.px(min).max(1.0);
        let sizes: Vec<Size> = items.iter().map(|t| self.size(t, least, width)).collect();
        let measured: RefCell<HashMap<(usize, u32), f32>> = RefCell::default();
        let real = |t: usize, w: f32| -> f32 {
            let w = w.round();
            if let Some(h) = measured.borrow().get(&(t, w as u32)) {
                return *h;
            }
            let h = self.measure(&items[t], w).1;
            measured.borrow_mut().insert((t, w as u32), h);
            h
        };
        let model = |t: usize, w: f32| sizes[t].h(w);
        let height: &dyn Fn(usize, f32) -> f32 = if items.len() <= MOST_MEASURED { &real } else { &model };
        let (bands, natural) = self.lay_bands(items, &sizes, width, gap, height);
        if items.len() < 3 || items.len() > MOST_MEASURED {
            return bands;
        }
        self.better_puzzle(bands, &natural, &sizes, width, gap, &model, &real)
    }

    fn lay_bands<M>(&self, items: &[Node<M>], sizes: &[Size], width: f32, gap: f32, height: &dyn Fn(usize, f32) -> f32) -> (Packed, Vec<f32>) {
        let mut rects = vec![Rect::default(); items.len()];
        let mut y = 0.0;
        let mut natural = vec![0.0; items.len()];
        for mut band in plan(sizes, width, gap, height) {
            self.settle_band(items, sizes, &mut band, width, gap);
            let hs: Vec<Vec<f32>> = band.iter()
                .map(|c| items[c.first..c.first + c.count].iter().map(|t| self.measure(t, c.w).1).collect())
                .collect();
            let col_h = |h: &Vec<f32>| h.iter().sum::<f32>() + gap * (h.len() - 1) as f32;
            let tall = hs.iter().map(col_h).fold(0.0, f32::max);
            for (c, h) in band.iter().zip(&hs) {
                let spare = tall - col_h(h);
                let giver = (c.first..c.first + c.count).find(|&t| sizes[t].gives);
                let each = if giver.is_some() { 0.0 } else { (spare / c.count as f32).floor() };
                let mut top = y;
                for (k, t) in (c.first..c.first + c.count).enumerate() {
                    let last = k + 1 == c.count;
                    let extra = match giver {
                        Some(g) if g == t => spare,
                        Some(_) => 0.0,
                        None if last => spare - each * (c.count - 1) as f32,
                        None => each,
                    };
                    rects[t] = Rect { x: c.x, y: top, w: c.w, h: h[k] + extra };
                    natural[t] = h[k];
                    top += h[k] + extra + gap;
                }
            }
            y += tall + gap;
        }
        (Packed { rects, height: (y - gap).max(0.0), seen: Vec::new() }, natural)
    }

    fn better_puzzle(&self, bands: Packed, natural: &[f32], sizes: &[Size], width: f32, gap: f32, model: &dyn Fn(usize, f32) -> f32, real: &dyn Fn(usize, f32) -> f32) -> Packed {
        let seed: Vec<f32> = bands.rects.iter().map(|r| r.w).collect();
        let plain = cost(&bands.rects, natural, sizes, width, gap);
        let key = hashed(|h| {
            for s in sizes {
                (s.lo.to_bits(), s.hi.to_bits(), s.gives).hash(h);
                s.at.iter().for_each(|(w, t)| (w.to_bits(), t.to_bits()).hash(h));
            }
            (width.to_bits(), gap.to_bits()).hash(h);
        });
        let found = FOUND.with(|f| f.borrow().get(&key).cloned());
        let widths = found.unwrap_or_else(|| {
            let w = puzzle(sizes, width, gap, model, &seed, plain - 0.5);
            FOUND.with(|f| {
                let mut f = f.borrow_mut();
                if f.len() >= MOST_FOUND {
                    f.clear();
                }
                f.insert(key, w.clone());
            });
            w
        });
        match widths.and_then(|w| replay(sizes, &w, width, gap, real)) {
            Some((rects, nat)) if cost(&rects, &nat, sizes, width, gap) < plain - 0.5 => {
                let height = rects.iter().map(|r| r.y + r.h).fold(0.0, f32::max);
                Packed { rects, height, seen: Vec::new() }
            }
            _ => bands,
        }
    }

    fn fewest<M>(&self, t: &Node<M>, min: f32, gap_px: f32, width: f32) -> usize {
        let (least, words) = t.span.map_or((1, min), |s| (s.least as usize, s.words));
        least.max(puzzle::twelfths_for(self.px(words), width, gap_px)).clamp(1, puzzle::TWELFTHS)
    }

    fn cards<M>(&self, items: &[Node<M>], min: f32, gap_px: f32, width: f32) -> u64 {
        hashed(|h| {
            (items.len(), gap_px.to_bits(), min.to_bits()).hash(h);
            for t in items {
                t.key.hash(h);
                t.span.map(|s| (s.least, s.most, s.words.to_bits(), s.grows)).hash(h);
                self.fewest(t, min, gap_px, width).hash(h);
            }
        })
    }

    fn pack_held<M>(&self, items: &[Node<M>], min: f32, gap: f32, width: f32) -> Packed {
        let gap_px = self.px(gap);
        let cards = self.cards(items, min, gap_px, width);
        let named = self.puzzles.get(&(items.as_ptr() as usize)).zip(self.held);
        if let Some(h) = named.and_then(|(name, kept)| kept.get(name, cards)) {
            return self.place_held(&h, items, width, gap_px);
        }
        let (rects, seen, boxy) = self.pack_puzzle(items, min, gap, width);
        let h = Held::of(&rects, seen, width, gap_px, cards, boxy);
        let packed = self.place_held(&h, items, width, gap_px);
        if let Some((name, kept)) = named {
            kept.put(name, h);
        }
        packed
    }

    fn place_held<M>(&self, h: &Held, items: &[Node<M>], width: f32, gap: f32) -> Packed {
        let across = h.across(width, gap);
        let own: Vec<f32> = items.iter().zip(&across).map(|(t, &(_, w))| {
            let tall = self.measure(t, w).1;
            if h.boxy && !t.span.is_some_and(|s| s.flat) { tall.max((w / 3.0).round()) } else { tall }
        }).collect();
        let mut top = vec![0.0f32; items.len()];
        for &t in &h.down {
            top[t] = h.over[t].iter().map(|&s| top[s] + own[s] + gap).fold(0.0, f32::max);
        }
        let height = top.iter().zip(&own).map(|(y, o)| y + o).fold(0.0, f32::max);
        let rects = (0..items.len()).map(|t| {
            let end = h.under[t].iter().map(|&u| top[u] - gap).fold(height, f32::min);
            let mut tall = (end - top[t]).max(own[t]);
            let stretches = gives(&items[t]) || items[t].span.is_some_and(|s| s.grows);
            if h.boxy && !stretches && tall - own[t] > LOOSE.max(QUIET_OVER * tall) {
                tall = own[t];
            }
            Rect { x: across[t].0, y: top[t], w: across[t].1, h: tall }
        }).collect();
        Packed { rects, height, seen: h.seen.clone() }
    }

    fn pack_puzzle<M>(&self, items: &[Node<M>], min: f32, gap: f32, width: f32) -> (Vec<Rect>, Vec<usize>, bool) {
        let gap_px = self.px(gap);
        let pieces: Vec<puzzle::Piece> = items.iter().map(|t| {
            let most = match t.span {
                Some(s) => s.most as usize,
                None => puzzle::twelfths_for(self.natural_width(t, width), width, gap_px),
            };
            let least = self.fewest(t, min, gap_px, width);
            puzzle::Piece { least, most: most.clamp(least, puzzle::TWELFTHS), gives: gives(t) || t.span.is_some_and(|s| s.grows) }
        }).collect();
        let measured: RefCell<HashMap<(usize, u32), f32>> = RefCell::default();
        let height = |t: usize, w: f32| -> f32 {
            let key = (t, w.round() as u32);
            if let Some(h) = measured.borrow().get(&key) {
                return *h;
            }
            let natural = self.measure(&items[t], w.round()).1;
            let h = if items[t].span.is_some_and(|s| s.flat) { natural } else { natural.max((w / 3.0).round()) };
            measured.borrow_mut().insert(key, h);
            h
        };
        let key = hashed(|h| {
            for (t, p) in pieces.iter().enumerate() {
                (p.least, p.most, p.gives).hash(h);
                for u in [p.least, p.most] {
                    height(t, puzzle::across(0, u, width, gap_px).1).to_bits().hash(h);
                }
            }
            (width.to_bits(), gap_px.to_bits()).hash(h);
        });
        let found = LAID.with(|f| f.borrow().get(&key).cloned());
        let laid = match found {
            Some(Some((seen, units))) => puzzle::replay(&pieces, &seen, &units, width, gap_px, &height)
                .or_else(|| puzzle::pack(&pieces, width, gap_px, &height)),
            Some(None) => None,
            None => {
                let laid = puzzle::pack(&pieces, width, gap_px, &height);
                LAID.with(|f| {
                    let mut f = f.borrow_mut();
                    if f.len() >= MOST_FOUND {
                        f.clear();
                    }
                    f.insert(key, laid.as_ref().map(|l| (l.seen.clone(), l.units.clone())));
                });
                laid
            }
        };
        match laid {
            Some(l) => (l.rects, l.seen, true),
            None => {
                let p = self.pack(items, min, gap, width);
                (p.rects, p.seen, false)
            }
        }
    }

    fn settle_band<M>(&self, items: &[Node<M>], sizes: &[Size], band: &mut Vec<Column>, width: f32, gap: f32) {
        if band.len() < 2 {
            return;
        }
        let cs: Vec<Col> = band.iter().map(|c| Col::new(&sizes[c.first..c.first + c.count], gap)).collect();
        let room = width - gap * (band.len() - 1) as f32;
        let (lo, hi): (Vec<f32>, Vec<f32>) = cs.iter().map(|c| (c.lo, c.hi)).unzip();
        if hi.iter().sum::<f32>() <= room {
            return;
        }
        let worst = |band: &[Column]| -> f32 {
            let hs: Vec<f32> = band.iter().map(|c| self.column_height(items, c, c.w, gap)).collect();
            let tall = hs.iter().copied().fold(0.0, f32::max);
            band.iter().zip(&hs).zip(&cs)
                .filter(|(_, c)| !c.gives())
                .map(|((col, h), _)| (tall - h) / col.count as f32)
                .fold(0.0, f32::max)
        };
        let now = worst(band);
        if now <= LOOSE {
            return;
        }
        let cols: Vec<(usize, usize)> = band.iter().map(|c| (c.first, c.count)).collect();
        let gives: Vec<bool> = cs.iter().map(Col::gives).collect();
        let widths = share(&lo, &hi, &gives, room, (10, 8), |c, w| self.column_height(items, &band[c], w.round(), gap));
        let better = columns(&cols, &widths, width, gap);
        if worst(&better) < now {
            *band = better;
        }
    }
}

#[cfg(any(test, feature = "testing"))]
pub mod check;

#[cfg(any(test, feature = "testing"))]
pub mod shape;

#[cfg(test)]
mod tests;
