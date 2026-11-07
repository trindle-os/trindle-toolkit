//! The eye in a secret box: shows what is typed, and hides it again when the box leaves the screen, the window loses the keyboard, a try is made, the program empties the box, or a guarded box sits idle for half a minute. What shows is never copied, written down or given to the input method, and the screen reader is only told shown or hidden, never the characters.

use super::*;
use crate::common::eye::{rested, HIDE_ICON, REST, SHOW_ICON};
use super::super::widget::Secret;

#[derive(Default)]
pub(super) struct Eyes {
    open: HashMap<String, Instant>,
    pub(super) on: Option<String>,
    pub(super) over: Option<String>,
    pub(super) down: Option<String>,
    held: Vec<String>,
    turns: HashMap<String, u32>,
}

impl<M: Clone> Ui<M> {
    pub(super) fn eye_of(&self, id: &str) -> Option<(Secret, bool)> {
        match self.node(id).filter(|n| !n.off).map(|n| &n.what) {
            Some(What::Entry { secret: true, kind, guarded, .. }) => Some((*kind, *guarded)),
            _ => None,
        }
    }

    pub(super) fn eye_open(&self, id: &str) -> bool {
        self.eyes.open.contains_key(id)
    }

    pub(super) fn eyes_open(&self) -> usize {
        self.eyes.open.len()
    }

    pub(super) fn entry_dots(&self, id: &str) -> bool {
        self.entry_secret(id) && !self.eye_open(id)
    }

    pub(super) fn on_eye(&self, id: &str) -> bool {
        self.eyes.on.as_deref() == Some(id) && self.focus.as_deref() == Some(id)
    }

    pub(super) fn eye_rect(&self, r: Rect) -> Rect {
        let size = self.px(theme::sheet().entry_eye).min(r.h);
        let inset = ((r.h - size) / 2.0).round();
        Rect { x: r.x + r.w - inset - size, y: r.y + inset, w: size, h: size }
    }

    pub(super) fn eye_room(&self, r: Rect) -> f32 {
        r.x + r.w - self.eye_rect(r).x
    }

    fn eye_reach(&self, r: Rect) -> Rect {
        let w = r.h.max(self.eye_room(r)).min(r.w);
        Rect { x: r.x + r.w - w, y: r.y, w, h: r.h }
    }

    pub(super) fn eye_at(&self, x: f32, y: f32) -> Option<String> {
        let id = self.hit(x, y)?;
        self.eye_of(&id)?;
        let spot = self.spot(&id)?;
        self.eye_reach(spot.rect).contains(x, y).then_some(id)
    }

    pub(super) fn eye_name(&self, id: &str) -> Option<&'static str> {
        let (kind, _) = self.eye_of(id)?;
        Some(if self.eye_open(id) { kind.hide() } else { kind.show() })
    }

    pub(super) fn eye_press(&mut self, id: &str, now: Instant, reply: &mut Reply<M>) {
        let Some((kind, _)) = self.eye_of(id) else { return };
        let shown = self.eyes.open.remove(id).is_none();
        if shown {
            self.eyes.open.insert(id.to_string(), now);
        }
        self.news = kind.told(shown).to_string();
        reply.redraw = true;
    }

    pub fn hide_secrets(&mut self) -> bool {
        let any = !self.eyes.open.is_empty();
        self.eyes.open.clear();
        any
    }

    pub(super) fn eyes_before_view(&mut self) {
        let held: Vec<String> = self.eyes.open.keys()
            .filter(|id| matches!(self.node(id).map(|n| &n.what), Some(What::Entry { text, .. }) if !text.is_empty()))
            .cloned().collect();
        self.eyes.held.extend(held);
    }

    pub(super) fn eyes_settle(&mut self) {
        #[cfg(any(test, feature = "testing"))]
        for s in &self.spots {
            if let Some(n) = self.tree.at(&s.path) {
                assert!(!matches!(n.what, What::Entry { secret: true, .. }) || n.key.is_some(), "the secret box at {} has no key", s.id);
            }
        }
        let held = std::mem::take(&mut self.eyes.held);
        let emptied = |ui: &Self, id: &str| held.iter().any(|h| h == id)
            && matches!(ui.node(id).map(|n| &n.what), Some(What::Entry { text, .. }) if text.is_empty())
            && ui.edits.get(id).is_none_or(|e| !e.text.is_empty());
        let gone: Vec<String> = self.eyes.open.keys().filter(|id| self.eye_of(id).is_none() || emptied(self, id)).cloned().collect();
        for id in gone {
            self.eyes.open.remove(&id);
        }
        let asked: Vec<(String, u32)> = self.spots.iter().filter_map(|s| match self.tree.at(&s.path).filter(|n| !n.off).map(|n| &n.what) {
            Some(What::Entry { secret: true, show, .. }) if *show > self.eyes.turns.get(&s.id).copied().unwrap_or(0) => Some((s.id.clone(), *show)),
            _ => None,
        }).collect();
        for (id, turn) in asked {
            self.eyes.turns.insert(id.clone(), turn);
            self.eyes.open.insert(id, Instant::now());
        }
        let live = |ui: &Self, k: &Option<String>| k.clone().filter(|id| ui.eye_of(id).is_some());
        let (on, over, down) = (live(self, &self.eyes.on), live(self, &self.eyes.over), live(self, &self.eyes.down));
        self.eyes.on = on.filter(|id| self.focus.as_deref() == Some(id));
        self.eyes.over = over;
        self.eyes.down = down;
    }

    pub(super) fn eyes_key(&mut self, now: Instant) {
        for at in self.eyes.open.values_mut() {
            *at = now;
        }
    }

    pub(super) fn eye_step(&mut self, forward: bool) -> bool {
        let Some(id) = self.focus.clone().filter(|id| self.eye_of(id).is_some()) else {
            self.eyes.on = None;
            return false;
        };
        let on = self.on_eye(&id);
        self.eyes.on = None;
        if forward && !on {
            self.eyes.on = Some(id);
            return true;
        }
        !forward && on
    }

    pub(super) fn eye_land(&mut self, id: &str) {
        if self.eye_of(id).is_some() {
            self.eyes.on = Some(id.to_string());
        }
    }

    pub(super) fn eye_key(&mut self, id: &str, sym: Keysym, text: Option<&str>, mods: Mods, now: Instant, reply: &mut Reply<M>) -> bool {
        if !self.on_eye(id) {
            return false;
        }
        if matches!(sym, Keysym::space | Keysym::Return | Keysym::KP_Enter | Keysym::KP_Space) {
            self.eye_press(id, now, reply);
            return true;
        }
        let typed = text.is_some_and(|t| !mods.ctrl && !mods.alt && !mods.logo && !t.chars().any(char::is_control));
        let edits = matches!(sym, Keysym::BackSpace | Keysym::Delete | Keysym::KP_Delete | Keysym::Left | Keysym::Right | Keysym::Home
            | Keysym::End | Keysym::KP_Left | Keysym::KP_Right | Keysym::KP_Home | Keysym::KP_End);
        if typed || edits {
            self.eyes.on = None;
            reply.redraw = true;
        }
        false
    }

    pub(super) fn eyes_shut(&mut self, reply: &mut Reply<M>) {
        if self.hide_secrets() {
            reply.redraw = true;
        }
    }

    pub(super) fn eyes_tick(&mut self, now: Instant) -> bool {
        let left: Vec<(String, Secret)> = self.eyes.open.iter().filter(|(_, at)| rested(**at, now))
            .filter_map(|(id, _)| self.eye_of(id).filter(|(_, guarded)| *guarded).map(|(kind, _)| (id.clone(), kind))).collect();
        for (id, kind) in &left {
            self.eyes.open.remove(id);
            self.news = kind.told(false).to_string();
        }
        !left.is_empty()
    }

    pub(super) fn eyes_wake(&self, now: Instant) -> Option<Duration> {
        self.eyes.open.iter().filter(|(id, _)| self.eye_of(id).is_some_and(|(_, guarded)| guarded))
            .map(|(_, at)| (*at + REST).saturating_duration_since(now)).min()
    }

    pub(super) fn paint_eye(&self, canvas: &mut PixmapMut, spot: &Spot, faded: bool) {
        let dim;
        let t = if faded { dim = self.theme.faded(); &dim } else { &self.theme };
        let b = self.eye_rect(spot.rect);
        let open = self.eye_open(&spot.id);
        let pointed = !faded && (self.eyes.over.as_deref() == Some(&spot.id) || self.eyes.down.as_deref() == Some(&spot.id));
        let lit = open || pointed;
        fill(canvas, b, self.corner(theme::Part::IconButton), if lit { t.picked } else { t.control });
        let ink = if lit { t.on_picked } else if faded { t.muted } else { t.fg };
        let across = b.w / self.scale.max(f32::EPSILON);
        let i = self.px((across * theme::icon() / theme::icon_button()).round());
        let (ix, iy) = ((b.x + (b.w - i) / 2.0) as i32, (b.y + (b.w - i) / 2.0) as i32);
        self.icons.draw(canvas, if open { HIDE_ICON } else { SHOW_ICON }, ix, iy, i as u32, 1, ink);
    }
}

#[cfg(any(test, feature = "testing"))]
impl<M: Clone> Ui<M> {
    pub fn shows_secret(&self, id: &str) -> bool {
        self.eye_open(id)
    }

    pub fn press_eye(&mut self, id: &str) {
        self.settle();
        assert!(self.eye_of(id).is_some(), "{id} is not a secret box with an eye");
        self.eye_press(id, Instant::now(), &mut Reply::new());
    }

    pub fn eye_place(&mut self, id: &str) -> Option<Rect> {
        self.settle();
        self.eye_of(id)?;
        self.spot(id).map(|s| self.eye_rect(s.rect))
    }
}
