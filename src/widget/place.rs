//! Putting each node where it goes on the canvas and noting where that was.

use std::collections::HashMap;

use super::measure::line_height;
use super::{Anchor, Lay, Length, Node, Rect, SLIDE_ROOM, Spot, What, WINDOW_MARGIN, id_of};

#[derive(Default)]
pub(crate) struct Stays {
    kept: std::cell::RefCell<HashMap<String, ((f32, f32), (f32, f32))>>,
    seen: std::cell::RefCell<Vec<String>>,
}

impl Stays {
    fn at(&self, id: &str, corner: (f32, f32), room: (f32, f32)) -> (f32, f32) {
        self.seen.borrow_mut().push(id.to_string());
        let mut kept = self.kept.borrow_mut();
        match kept.get(id) {
            Some((was, in_room)) if *in_room == room => *was,
            _ => {
                kept.insert(id.to_string(), (corner, room));
                corner
            }
        }
    }

    pub(crate) fn settle(&self) {
        let seen = std::mem::take(&mut *self.seen.borrow_mut());
        self.kept.borrow_mut().retain(|id, _| seen.contains(id));
    }
}

fn own_spot<M>(n: &Node<M>, rect: Rect, clip: Option<Rect>, id: &str, room: Option<(f32, f32)>, path: &[usize], spots: &mut Vec<Spot>) -> Option<Rect> {
    let mut inner_clip = clip;
    match &n.what {
        What::Column { .. } | What::Row { .. } if n.says.as_deref().is_some_and(|s| s.role.is_some()) => {
            spots.push(Spot { id: id.to_string(), path: path.to_vec(), rect, clip, full: 0.0 });
        }
        What::Space | What::Column { .. } | What::Row { .. } | What::Grid { .. } | What::Layers(_) | What::Placed { .. } | What::Aspect { .. }
            if n.says.as_ref().is_none_or(|s| s.landmark.is_none()) => {}
        What::Slide { .. } => {
            spots.push(Spot { id: id.to_string(), path: path.to_vec(), rect, clip, full: 0.0 });
            inner_clip = Some(clip.map_or(rect, |c| c.intersect(&rect)));
        }
        What::ScrollX(_) => {
            let full = room.map_or(0.0, |r| r.0);
            spots.push(Spot { id: id.to_string(), path: path.to_vec(), rect, clip, full });
            inner_clip = Some(clip.map_or(rect, |c| c.intersect(&rect)));
        }
        What::Scroll(_) => {
            let full = room.map_or(0.0, |r| r.1);
            spots.push(Spot { id: id.to_string(), path: path.to_vec(), rect, clip, full });
            inner_clip = Some(clip.map_or(rect, |c| c.intersect(&rect)));
        }
        _ => spots.push(Spot { id: id.to_string(), path: path.to_vec(), rect, clip, full: 0.0 }),
    }
    inner_clip
}

impl Lay<'_> {
    pub(crate) fn place<M>(&self, n: &Node<M>, rect: Rect, path: &mut Vec<usize>, spots: &mut Vec<Spot>) {
        self.place_in(n, rect, None, None, path, spots)
    }

    fn place_in<M>(&self, n: &Node<M>, rect: Rect, clip: Option<Rect>, anchor: Anchor, path: &mut Vec<usize>, spots: &mut Vec<Spot>) {
        let id = id_of(n.key.as_ref(), anchor, path);
        let anchor = n.key.as_deref().map(|k| (k, path.len())).or(anchor);
        #[cfg(any(test, feature = "testing"))]
        let here = matches!(n.what, What::Grid { even: false, .. }).then(|| path.clone()).filter(|_| self.bento.logging());
        let room = match &n.what {
            What::Scroll(inner) => Some(self.scroll_room(inner, rect)),
            What::ScrollX(inner) => Some(self.scroll_x_room(inner, rect)),
            _ => None,
        };
        let inner_clip = own_spot(n, rect, clip, &id, room, path, spots);
        let mut child = |i: usize, item: &Node<M>, r: Rect, spots: &mut Vec<Spot>| {
            path.push(i);
            self.place_in(item, r, inner_clip, anchor, path, spots);
            path.pop();
        };
        match &n.what {
            What::Placed { .. } => self.place_placed(n, rect, &id, &mut child, spots),
            What::Split { .. } => self.place_split(n, rect, &id, &mut child, spots),
            What::Column { .. } => self.place_column(n, rect, &mut child, spots),
            What::Row { .. } => self.place_row(n, rect, &mut child, spots),
            What::Grid { .. } => self.place_grid(n, rect, #[cfg(any(test, feature = "testing"))] &here, &mut child, spots),
            _ => self.place_inside(n, rect, &id, room, &mut child, spots),
        }
    }

    fn place_inside<M>(&self, n: &Node<M>, rect: Rect, id: &str, room: Option<(f32, f32)>, child: &mut impl FnMut(usize, &Node<M>, Rect, &mut Vec<Spot>), spots: &mut Vec<Spot>) {
        match &n.what {
            What::Card(inner, _) => {
                let (px, py) = self.card_padding(n);
                let r = Rect { x: rect.x + px, y: rect.y + py, w: (rect.w - 2.0 * px).max(0.0), h: (rect.h - 2.0 * py).max(0.0) };
                child(0, inner, r, spots)
            }
            What::Layers(items) => {
                for (i, item) in items.iter().enumerate() {
                    child(i, item, rect, spots);
                }
            }
            What::Aspect { inner, ratio } => {
                let w = rect.w.min(rect.h * ratio).floor();
                let h = (w / ratio).floor();
                child(0, inner, Rect { x: rect.x + ((rect.w - w) / 2.0).round(), y: rect.y + ((rect.h - h) / 2.0).round(), w, h }, spots);
            }
            What::Slide { inner, by, .. } => {
                let (w, mut h) = self.measure(inner, SLIDE_ROOM);
                if inner.height == Length::Fill {
                    h = rect.h;
                }
                child(0, inner, Rect { x: rect.x - self.px(*by).round(), y: rect.y + ((rect.h - h) / 2.0).round(), w, h }, spots);
            }
            What::ScrollX(inner) => {
                let (full, h) = room.unwrap_or_default();
                let off = self.scrolled.get(id).copied().unwrap_or(0.0).clamp(0.0, full - rect.w).round();
                child(0, inner, Rect { x: rect.x - off, w: full, h, ..rect }, spots);
            }
            What::Scroll(inner) => {
                let (w, full) = room.unwrap_or_default();
                let off = self.scrolled.get(id).copied().unwrap_or(0.0).clamp(0.0, full - rect.h).round();
                child(0, inner, Rect { y: rect.y - off, w, h: full, ..rect }, spots);
            }
            _ => {}
        }
    }

    fn place_placed<M>(&self, n: &Node<M>, rect: Rect, id: &str, child: &mut dyn FnMut(usize, &Node<M>, Rect, &mut Vec<Spot>), spots: &mut Vec<Spot>) {
        match &n.what {
            What::Placed { inner, at, stays, within } => {
                let (w, h) = if *within { self.measure_within(inner, (rect.w, rect.h)) } else { self.measure_floating(inner, rect.w) };
                let k = self.scale.max(f32::EPSILON);
                let (x, y) = at((w / k, h / k), (rect.w / k, rect.h / k));
                let (mut x, mut y) = ((x * k).round(), (y * k).round());
                if *within {
                    let m = self.px(WINDOW_MARGIN);
                    x = x.min(rect.w - w - m).max(m.min((rect.w - w).max(0.0)));
                    y = y.min(rect.h - h - m).max(m.min((rect.h - h).max(0.0)));
                }
                if let Some(kept) = self.stays.filter(|_| *stays) {
                    let was = kept.at(id, (x, y), (rect.w, rect.h));
                    x = was.0.min((rect.w - w).max(0.0));
                    y = was.1.min((rect.h - h).max(0.0));
                }
                child(0, inner, Rect { x: rect.x + x, y: rect.y + y, w, h }, spots);
            }
            _ => unreachable!(),
        }
    }

    fn place_column<M>(&self, n: &Node<M>, rect: Rect, child: &mut dyn FnMut(usize, &Node<M>, Rect, &mut Vec<Spot>), spots: &mut Vec<Spot>) {
        match &n.what {
            What::Column { items, gap, padding, centre } => {
                let (gap, inner) = (self.px(*gap), rect.inset(self.px(*padding)));
                let sizes = self.column_sizes(n, items, inner.w, gap);
                let fixed: f32 = items.iter().zip(&sizes).filter(|(i, _)| i.height != Length::Fill).map(|(_, s)| s.1).sum();
                let fills = items.iter().filter(|i| i.height == Length::Fill).count().max(1) as f32;
                let spare = (inner.h - fixed - gap * items.len().saturating_sub(1) as f32).max(0.0);
                let mut y = inner.y;
                for (i, (item, (w, h))) in items.iter().zip(sizes).enumerate() {
                    let h = if item.height == Length::Fill { (spare / fills).floor().max(h) } else { h };
                    let x = if *centre { inner.x + ((inner.w - w) / 2.0).round() } else { inner.x };
                    child(i, item, Rect { x, y, w, h }, spots);
                    y += h + gap;
                }
            }
            _ => unreachable!(),
        }
    }

    fn place_row<M>(&self, n: &Node<M>, rect: Rect, child: &mut dyn FnMut(usize, &Node<M>, Rect, &mut Vec<Spot>), spots: &mut Vec<Spot>) {
        match &n.what {
            What::Row { items, gap, padding } if n.stacks && self.row_stacks(items, rect.w - 2.0 * self.px(*padding), self.px(*gap)) => {
                let (gap, inner) = (self.px(*gap), rect.inset(self.px(*padding)));
                let mut y = inner.y;
                for (i, item) in items.iter().enumerate() {
                    if matches!(item.what, What::Space) {
                        continue;
                    }
                    let (w, h) = self.measure(item, inner.w);
                    let w = if item.width == Length::Fill { inner.w } else { w };
                    child(i, item, Rect { x: inner.x, y, w, h }, spots);
                    y += h + gap;
                }
            }
            What::Row { items, gap, padding } => {
                let (gap, inner) = (self.px(*gap), rect.inset(self.px(*padding)));
                if let Some(lines) = self.row_lines(items, inner.w, gap, n.wraps) {
                    let mut y = inner.y;
                    for line in &lines {
                        let tall = line_height(line);
                        let mut x = inner.x;
                        for &(i, w, h) in line {
                            child(i, &items[i], Rect { x, y: y + ((tall - h) / 2.0).round(), w, h }, spots);
                            x += w + gap;
                        }
                        y += tall + gap;
                    }
                } else {
                    let mut x = inner.x;
                    for (i, (item, w)) in items.iter().zip(self.row_widths(items, inner.w, gap)).enumerate() {
                        let h = if item.height == Length::Fill { inner.h } else { self.measure(item, w).1.min(inner.h) };
                        let y = inner.y + ((inner.h - h) / 2.0).round();
                        child(i, item, Rect { x, y, w, h }, spots);
                        x += w + gap;
                    }
                }
            }
            _ => unreachable!(),
        }
    }

    fn place_grid<M>(&self, n: &Node<M>, rect: Rect, #[cfg(any(test, feature = "testing"))] here: &Option<Vec<usize>>, child: &mut dyn FnMut(usize, &Node<M>, Rect, &mut Vec<Spot>), spots: &mut Vec<Spot>) {
        match &n.what {
            What::Grid { items, gap, flow: true, .. } => {
                for (i, (item, r)) in items.iter().zip(self.flowed(items, *gap, rect.w).0).enumerate() {
                    child(i, item, Rect { x: rect.x + r.x, y: rect.y + r.y, ..r }, spots);
                }
            }
            What::Grid { items, min, gap, even: false, puzzle, .. } => {
                let packed = self.packed(items, *min, *gap, rect.w, *puzzle);
                for i in packed.seen() {
                    let (item, r) = (&items[i], &packed.rects[i]);
                    let r = Rect { x: rect.x + r.x, y: rect.y + r.y, ..*r };
                    #[cfg(any(test, feature = "testing"))]
                    if let Some(here) = &here {
                        self.bento.note([here.as_slice(), &[i]].concat(), r);
                    }
                    child(i, item, r, spots);
                }
            }
            What::Grid { items, min, gap, even: true, grow: true, .. } => {
                let (cols, cell, high, gap) = self.grown_cells(items, rect.w, *min, *gap, Some(rect.h));
                for (r, line) in items.chunks(cols).enumerate() {
                    for (c, item) in line.iter().enumerate() {
                        let x = rect.x + c as f32 * (cell + gap);
                        child(r * cols + c, item, Rect { x, y: rect.y + r as f32 * (high + gap), w: cell, h: high }, spots);
                    }
                }
            }
            What::Grid { items, min, gap, even: true, .. } => {
                let (cols, cell, gap) = self.grid_cells(rect.w, *min, *gap, items.len());
                let mut y = rect.y;
                for (r, line) in items.chunks(cols).enumerate() {
                    let h = line.iter().map(|i| self.measure(i, cell).1).fold(0.0, f32::max);
                    for (c, item) in line.iter().enumerate() {
                        let x = rect.x + c as f32 * (cell + gap);
                        child(r * cols + c, item, Rect { x, y, w: cell, h }, spots);
                    }
                    y += h + gap;
                }
            }
            _ => unreachable!(),
        }
    }
}
