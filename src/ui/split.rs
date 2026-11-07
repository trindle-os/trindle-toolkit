//! A split's divider moved by the pointer and the keys, told to the screen reader, and its place saved.

use std::collections::BTreeMap;

use accesskit::{Node as Part, Orientation, Role};

use super::*;
use crate::common::plumbing::Ini;

#[derive(Default)]
pub(super) struct Splits {
    pub(super) places: HashMap<String, f32>,
    grab: f32,
}

struct Place {
    split: String,
    down: bool,
    from: f32,
    room: f32,
    at: f32,
    least: f32,
    most: f32,
}

impl<M: Clone> Ui<M> {
    pub(super) fn is_divider(&self, id: &str) -> bool {
        matches!(self.node(id).map(|n| &n.what), Some(What::Divider { .. }))
    }

    fn place_of(&self, divider: &str) -> Option<Place> {
        let d = self.spot(divider)?;
        let up = &d.path[..d.path.len().checked_sub(1)?];
        let What::Split { down, start, least, .. } = &self.tree.at(up)?.what else { return None };
        let s = self.spots.iter().find(|s| s.path == up)?;
        let (from, len) = if *down { (s.rect.y, s.rect.h) } else { (s.rect.x, s.rect.w) };
        let room = (len - self.px(theme::sheet().divider)).max(0.0);
        let side = |kept| w::first_side(kept, *start, *least, room, self.scale);
        Some(Place { split: s.id.clone(), down: *down, from, room, at: side(self.splits.places.get(&s.id).copied()), least: side(Some(0.0)), most: side(Some(f32::INFINITY)) })
    }

    fn set_first(&mut self, p: &Place, px: f32) -> bool {
        let to = px.min(p.most).max(p.least).round();
        if to == p.at {
            return false;
        }
        self.splits.places.insert(p.split.clone(), to / self.k());
        self.laid = None;
        true
    }

    pub(super) fn divider_press(&mut self, id: String, (x, y): (f32, f32), now: Instant) {
        let Some(p) = self.place_of(&id) else { return };
        let twice = matches!(&self.last_click, Some((at, last, 1)) if *last == id && now.saturating_duration_since(*at) < crate::host::host().double_click());
        self.last_click = Some((now, id.clone(), if twice { 2 } else { 1 }));
        if twice {
            if self.splits.places.remove(&p.split).is_some() {
                self.laid = None;
            }
            return;
        }
        self.splits.grab = if p.down { y } else { x } - p.from - p.at;
        self.dragging = Some(id);
    }

    pub(super) fn divider_to(&mut self, id: &str, (x, y): (f32, f32), reply: &mut Reply<M>) {
        let Some(p) = self.place_of(id) else { return };
        let to = if p.down { y } else { x } - p.from - self.splits.grab;
        reply.redraw |= self.set_first(&p, to);
    }

    pub(super) fn divider_key(&mut self, id: &str, sym: Keysym, reply: &mut Reply<M>) -> bool {
        let Some(p) = self.place_of(id) else { return false };
        let step = self.px(w::SPLIT_STEP);
        let to = match (sym, p.down) {
            (Keysym::Left | Keysym::KP_Left, false) | (Keysym::Up | Keysym::KP_Up, true) => p.at - step,
            (Keysym::Right | Keysym::KP_Right, false) | (Keysym::Down | Keysym::KP_Down, true) => p.at + step,
            (Keysym::Home | Keysym::KP_Home, _) => p.least,
            (Keysym::End | Keysym::KP_End, _) => p.most,
            _ => return false,
        };
        reply.redraw |= self.set_first(&p, to);
        true
    }

    pub(super) fn divider_cursor(&self, id: &str) -> Cursor {
        match self.node(id).map(|n| &n.what) {
            Some(What::Divider { down: true }) => Cursor::RowResize,
            _ => Cursor::ColResize,
        }
    }

    pub(super) fn paint_divider(&self, canvas: &mut PixmapMut, spot: &Spot, down: bool, t: &Theme, lit: bool) {
        let lit = lit || (self.keyboard_seen && self.focus.as_deref() == Some(&spot.id));
        let sheet = theme::sheet();
        let thick = self.px(if lit { sheet.divider_lit } else { sheet.divider }).max(1.0);
        let r = spot.rect;
        let line = if down {
            Rect { y: r.y + ((r.h - thick) / 2.0).round(), h: thick, ..r }
        } else {
            Rect { x: r.x + ((r.w - thick) / 2.0).round(), w: thick, ..r }
        };
        fill(canvas, line, 0.0, if lit { t.accent } else { t.side_line });
    }

    pub(super) fn tell_divider(&self, s: &Spot, down: bool, part: &mut Part) {
        part.set_role(Role::Splitter);
        part.set_label(crate::t!("kit-divider"));
        part.set_orientation(if down { Orientation::Horizontal } else { Orientation::Vertical });
        if let Some(p) = self.place_of(&s.id).filter(|p| p.room > 0.0) {
            let share = |v: f32| f64::from((v / p.room * 100.0).round());
            part.set_numeric_value(share(p.at));
            part.set_min_numeric_value(share(p.least));
            part.set_max_numeric_value(share(p.most));
        }
    }

    pub fn split_places(&self) -> BTreeMap<String, f32> {
        self.splits.places.iter().map(|(k, v)| (k.clone(), *v)).collect()
    }

    pub fn set_split_places(&mut self, places: BTreeMap<String, f32>) {
        self.splits.places = places.into_iter().filter(|(_, v)| v.is_finite() && *v >= 0.0).collect();
        self.laid = None;
    }
}

impl<A: App + 'static> Shown<A> where A::Msg: Clone + 'static {
    pub fn split_places(&self) -> BTreeMap<String, f32> {
        self.held.borrow().ui.split_places()
    }

    pub fn set_split_places(&self, places: BTreeMap<String, f32>) {
        self.held.borrow_mut().ui.set_split_places(places);
        self.window.redraw();
    }
}

pub fn save_splits(ini: &mut Ini, section: &str, places: &BTreeMap<String, f32>) -> bool {
    let line: Vec<String> = places.iter().map(|(k, v)| format!("{}={}", escaped(k), v.round())).collect();
    ini.set(section, "Splits", &line.join(";"))
}

pub fn load_splits(ini: &Ini, section: &str) -> BTreeMap<String, f32> {
    ini.get_list(section, "Splits").into_iter()
        .filter_map(|pair| pair.rsplit_once('=').and_then(|(k, v)| Some((unescaped(k), v.parse::<f32>().ok().filter(|v| v.is_finite() && *v >= 0.0)?))))
        .collect()
}

fn escaped(key: &str) -> String {
    let mut out = String::new();
    for c in key.chars() {
        if c == '%' || c == ';' || c.is_whitespace() || c.is_control() {
            let mut b = [0; 4];
            for byte in c.encode_utf8(&mut b).bytes() {
                out.push_str(&format!("%{byte:02X}"));
            }
        } else {
            out.push(c);
        }
    }
    out
}

fn unescaped(key: &str) -> String {
    let (bytes, mut out, mut i) = (key.as_bytes(), Vec::new(), 0);
    while i < bytes.len() {
        let hex = (bytes[i] == b'%').then(|| key.get(i + 1..i + 3)).flatten().filter(|h| h.bytes().all(|b| b.is_ascii_hexdigit())).and_then(|h| u8::from_str_radix(h, 16).ok());
        match hex {
            Some(b) => (out.push(b), i += 3),
            None => (out.push(bytes[i]), i += 1),
        };
    }
    String::from_utf8_lossy(&out).into_owned()
}
