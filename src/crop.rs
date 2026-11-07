//! A crop box on a picture a program paints itself: handles, pulls, keyboard moves and the lines to paint.

use tiny_skia::PixmapMut;

use super::colour::Colour;
use super::draw::fill_on;
use super::paint::Painter;
use super::input::Cursor;
use super::widget::{Pen, Pointer, Rect};
use super::zoom::View;

pub const GRAB: f32 = 12.0;
const LEAST: f32 = 16.0;
const HANDLE: f32 = 12.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Handle {
    Body,
    N,
    Ne,
    E,
    Se,
    S,
    Sw,
    W,
    Nw,
}

impl Handle {
    pub fn cursor(self) -> Cursor {
        match self {
            Handle::Body => Cursor::Move,
            Handle::N => Cursor::NResize,
            Handle::Ne => Cursor::NeResize,
            Handle::E => Cursor::EResize,
            Handle::Se => Cursor::SeResize,
            Handle::S => Cursor::SResize,
            Handle::Sw => Cursor::SwResize,
            Handle::W => Cursor::WResize,
            Handle::Nw => Cursor::NwResize,
        }
    }

    pub fn is_corner(self) -> bool {
        matches!(self, Handle::Ne | Handle::Se | Handle::Sw | Handle::Nw)
    }

    fn left(self) -> bool {
        matches!(self, Handle::W | Handle::Nw | Handle::Sw)
    }
    fn right(self) -> bool {
        matches!(self, Handle::E | Handle::Ne | Handle::Se)
    }
    fn top(self) -> bool {
        matches!(self, Handle::N | Handle::Nw | Handle::Ne)
    }
    fn bottom(self) -> bool {
        matches!(self, Handle::S | Handle::Sw | Handle::Se)
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Crop {
    bounds: Rect,
    rect: Rect,
    aspect: Option<f32>,
    least: f32,
}

impl Crop {
    pub fn new(bounds: Rect) -> Self {
        Crop { bounds, rect: bounds, aspect: None, least: LEAST.min(bounds.w).min(bounds.h) }
    }

    pub fn rect(&self) -> Rect {
        self.rect
    }

    pub fn set_rect(&mut self, r: Rect) {
        self.rect = self.fitted(r);
    }

    pub fn reset(&mut self) {
        self.rect = self.fitted(self.bounds);
    }

    pub fn set_aspect(&mut self, aspect: Option<f32>) {
        self.aspect = aspect.filter(|a| a.is_finite() && *a > 0.0);
        self.rect = self.fitted(self.rect);
    }

    pub fn turn_right(&mut self) {
        let (b, r) = (self.bounds, self.rect);
        self.bounds = Rect { x: b.x, y: b.y, w: b.h, h: b.w };
        self.rect = Rect { x: b.x + b.h - (r.y - b.y) - r.h, y: b.y + (r.x - b.x), w: r.h, h: r.w };
        self.aspect = self.aspect.map(|a| 1.0 / a);
    }

    pub fn turn_left(&mut self) {
        let (b, r) = (self.bounds, self.rect);
        self.bounds = Rect { x: b.x, y: b.y, w: b.h, h: b.w };
        self.rect = Rect { x: b.x + (r.y - b.y), y: b.y + b.w - (r.x - b.x) - r.w, w: r.h, h: r.w };
        self.aspect = self.aspect.map(|a| 1.0 / a);
    }

    fn fitted(&self, r: Rect) -> Rect {
        let b = self.bounds;
        let (mut w, mut h) = (r.w.max(self.least).min(b.w), r.h.max(self.least).min(b.h));
        if let Some(a) = self.aspect {
            if w / h > a {
                w = h * a;
            } else {
                h = w / a;
            }
            if w < self.least {
                (w, h) = (self.least, self.least / a);
            }
            if h < self.least {
                (w, h) = (self.least * a, self.least);
            }
            if w > b.w || h > b.h {
                let k = (b.w / w).min(b.h / h);
                (w, h) = (w * k, h * k);
            }
        }
        let (cx, cy) = (r.x + r.w / 2.0, r.y + r.h / 2.0);
        let x = (cx - w / 2.0).min(b.x + b.w - w).max(b.x);
        let y = (cy - h / 2.0).min(b.y + b.h - h).max(b.y);
        Rect { x, y, w, h }
    }

    pub fn handle_at(&self, x: f32, y: f32, grab: f32) -> Option<Handle> {
        let r = self.rect;
        let grab = grab.min(r.w / 3.0).min(r.h / 3.0);
        let near = |p: f32, edge: f32| (p - edge).abs() <= grab;
        let (l, rt, t, bt) = (near(x, r.x), near(x, r.x + r.w), near(y, r.y), near(y, r.y + r.h));
        let within_x = x >= r.x - grab && x <= r.x + r.w + grab;
        let within_y = y >= r.y - grab && y <= r.y + r.h + grab;
        Some(if t && l {
            Handle::Nw
        } else if t && rt {
            Handle::Ne
        } else if bt && l {
            Handle::Sw
        } else if bt && rt {
            Handle::Se
        } else if t && within_x {
            Handle::N
        } else if bt && within_x {
            Handle::S
        } else if l && within_y {
            Handle::W
        } else if rt && within_y {
            Handle::E
        } else if r.contains(x, y) {
            Handle::Body
        } else {
            return None;
        })
    }

    pub fn pull(&mut self, handle: Handle, x: f32, y: f32) {
        if handle == Handle::Body {
            return;
        }
        let (b, r) = (self.bounds, self.rect);
        let x = x.clamp(b.x, b.x + b.w);
        let y = y.clamp(b.y, b.y + b.h);
        let (mut l, mut t, mut rt, mut bt) = (r.x, r.y, r.x + r.w, r.y + r.h);
        match self.aspect {
            None => {
                if handle.left() {
                    l = x.min(rt - self.least);
                }
                if handle.right() {
                    rt = x.max(l + self.least);
                }
                if handle.top() {
                    t = y.min(bt - self.least);
                }
                if handle.bottom() {
                    bt = y.max(t + self.least);
                }
                self.rect = Rect { x: l, y: t, w: rt - l, h: bt - t };
            }
            Some(a) => self.rect = self.pull_held(handle, x, y, a),
        }
    }

    fn pull_held(&self, handle: Handle, x: f32, y: f32, a: f32) -> Rect {
        let (b, r) = (self.bounds, self.rect);
        let low_w = self.least.max(self.least * a);
        if handle.is_corner() {
            let (ax, ay) = (if handle.left() { r.x + r.w } else { r.x }, if handle.top() { r.y + r.h } else { r.y });
            let (sx, sy) = (if handle.left() { -1.0 } else { 1.0 }, if handle.top() { -1.0 } else { 1.0 });
            let room_w = if sx < 0.0 { ax - b.x } else { b.x + b.w - ax };
            let room_h = if sy < 0.0 { ay - b.y } else { b.y + b.h - ay };
            let (pw, ph) = (((x - ax) * sx).max(0.0), ((y - ay) * sy).max(0.0));
            let top = room_w.min(room_h * a);
            let w = pw.min(ph * a).clamp(low_w.min(top), top);
            let h = w / a;
            Rect { x: if sx < 0.0 { ax - w } else { ax }, y: if sy < 0.0 { ay - h } else { ay }, w, h }
        } else if handle.left() || handle.right() {
            let fixed = if handle.left() { r.x + r.w } else { r.x };
            let room = if handle.left() { fixed - b.x } else { b.x + b.w - fixed };
            let cy = r.y + r.h / 2.0;
            let tall = 2.0 * (cy - b.y).min(b.y + b.h - cy);
            let top = room.min(tall * a);
            let w = ((x - fixed).abs()).clamp(low_w.min(top), top);
            let h = w / a;
            Rect { x: if handle.left() { fixed - w } else { fixed }, y: cy - h / 2.0, w, h }
        } else {
            let fixed = if handle.top() { r.y + r.h } else { r.y };
            let room = if handle.top() { fixed - b.y } else { b.y + b.h - fixed };
            let cx = r.x + r.w / 2.0;
            let wide = 2.0 * (cx - b.x).min(b.x + b.w - cx);
            let top = room.min(wide / a);
            let low_h = low_w / a;
            let h = ((y - fixed).abs()).clamp(low_h.min(top), top);
            let w = h * a;
            Rect { x: cx - w / 2.0, y: if handle.top() { fixed - h } else { fixed }, w, h }
        }
    }

    pub fn move_by(&mut self, dx: f32, dy: f32) {
        let (b, r) = (self.bounds, self.rect);
        self.rect.x = (r.x + dx).min(b.x + b.w - r.w).max(b.x);
        self.rect.y = (r.y + dy).min(b.y + b.h - r.h).max(b.y);
    }

    pub fn grow(&mut self, dw: f32, dh: f32) {
        let r = self.rect;
        self.pull(Handle::Se, r.x + r.w + dw, r.y + r.h + dh);
    }
}

#[derive(Debug, Default)]
pub struct Gesture {
    held: Option<Held>,
}

#[derive(Clone, Copy, Debug)]
struct Held {
    handle: Handle,
    from: (f32, f32),
    start: Rect,
}

impl Gesture {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn pointer(&mut self, crop: &mut Crop, view: &View, event: Pointer) -> bool {
        match event {
            Pointer::Down(x, y) => {
                let (cx, cy) = view.to_content(x, y);
                self.held = crop.handle_at(cx, cy, GRAB / view.scale()).map(|handle| Held { handle, from: (cx, cy), start: crop.rect() });
                false
            }
            Pointer::Move(x, y) => {
                let Some(h) = self.held else { return false };
                let (cx, cy) = view.to_content(x, y);
                let before = crop.rect();
                if h.handle == Handle::Body {
                    crop.rect = h.start;
                    crop.move_by(cx - h.from.0, cy - h.from.1);
                } else {
                    crop.pull(h.handle, cx, cy);
                }
                crop.rect != before
            }
            Pointer::Up(..) => {
                self.held = None;
                false
            }
            _ => false,
        }
    }

    pub fn holding(&self) -> Option<Handle> {
        self.held.map(|h| h.handle)
    }

    pub fn cursor(&self, crop: &Crop, view: &View, x: f32, y: f32) -> Cursor {
        if let Some(h) = self.holding() {
            return h.cursor();
        }
        let (cx, cy) = view.to_content(x, y);
        crop.handle_at(cx, cy, GRAB / view.scale()).map_or(Cursor::Arrow, Handle::cursor)
    }
}

pub fn paint_crop(canvas: &mut PixmapMut, pen: &Pen, outer: Rect, inner: Rect) {
    let mut p = Painter::default();
    paint_crop_on(&mut p, pen, outer, inner);
    p.play(canvas);
}

pub fn paint_crop_on(p: &mut Painter, pen: &Pen, outer: Rect, inner: Rect) {
    let dim = Colour { a: 0.55, ..Colour::rgb(0.0, 0.0, 0.0) };
    let (ox, oy, ow, oh) = (outer.x, outer.y, outer.w, outer.h);
    let shade = [
        Rect { x: ox, y: oy, w: ow, h: inner.y - oy },
        Rect { x: ox, y: inner.y + inner.h, w: ow, h: oy + oh - inner.y - inner.h },
        Rect { x: ox, y: inner.y, w: inner.x - ox, h: inner.h },
        Rect { x: inner.x + inner.w, y: inner.y, w: ox + ow - inner.x - inner.w, h: inner.h },
    ];
    for r in shade {
        fill_on(p, pen, r, dim);
    }
    let edge = 2.0;
    let accent = pen.theme.accent;
    for e in [Rect { h: edge, ..inner }, Rect { y: inner.y + inner.h - edge, h: edge, ..inner }, Rect { w: edge, ..inner }, Rect { x: inner.x + inner.w - edge, w: edge, ..inner }] {
        fill_on(p, pen, e, accent);
    }
    let (mx, my) = (inner.x + inner.w / 2.0, inner.y + inner.h / 2.0);
    let (rx, by) = (inner.x + inner.w, inner.y + inner.h);
    for (x, y) in [(inner.x, inner.y), (mx, inner.y), (rx, inner.y), (rx, my), (rx, by), (mx, by), (inner.x, by), (inner.x, my)] {
        let rim = Rect { x: x - HANDLE / 2.0, y: y - HANDLE / 2.0, w: HANDLE, h: HANDLE };
        fill_on(p, pen, rim, Colour { a: 0.85, ..Colour::rgb(0.0, 0.0, 0.0) });
        fill_on(p, pen, rim.inset(2.0), accent);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn crop() -> Crop {
        let mut c = Crop::new(Rect { x: 0.0, y: 0.0, w: 400.0, h: 300.0 });
        c.set_rect(Rect { x: 100.0, y: 100.0, w: 200.0, h: 100.0 });
        c
    }

    fn inside(c: &Crop) -> bool {
        let r = c.rect();
        r.x >= 0.0 && r.y >= 0.0 && r.x + r.w <= 400.0 + 0.01 && r.y + r.h <= 300.0 + 0.01
    }

    #[test]
    fn turning_the_box_keeps_the_same_part_of_the_picture() {
        let mut c = crop();
        c.set_aspect(Some(2.0));
        let start = c.rect();
        c.turn_right();
        assert_eq!(c.rect(), Rect { x: 100.0, y: 100.0, w: 100.0, h: 200.0 });
        c.set_aspect(Some(0.5));
        assert_eq!(c.rect(), Rect { x: 100.0, y: 100.0, w: 100.0, h: 200.0 }, "the ratio turned with it");
        c.turn_left();
        assert_eq!(c.rect(), start);
        for _ in 0..4 {
            c.turn_right();
        }
        assert_eq!(c.rect(), start);
        let mut corner = Crop::new(Rect { x: 0.0, y: 0.0, w: 400.0, h: 300.0 });
        corner.set_rect(Rect { x: 0.0, y: 0.0, w: 100.0, h: 50.0 });
        corner.turn_right();
        assert_eq!(corner.rect(), Rect { x: 250.0, y: 0.0, w: 50.0, h: 100.0 });
    }

    #[test]
    fn a_new_box_covers_the_picture() {
        let c = Crop::new(Rect { x: 0.0, y: 0.0, w: 400.0, h: 300.0 });
        assert_eq!(c.rect(), Rect { x: 0.0, y: 0.0, w: 400.0, h: 300.0 });
    }

    #[test]
    fn an_awkward_ratio_never_leaves_the_bounds_or_panics() {
        for a in [0.0001, 0.3, 1.0 / 3.0, 7.7, 5000.0] {
            for (w, h) in [(400.0, 300.0), (7.0, 900.0), (31.0, 31.0)] {
                let mut c = Crop::new(Rect { x: 5.0, y: 9.0, w, h });
                c.set_aspect(Some(a));
                let r = c.rect();
                assert!(r.x >= 5.0 && r.y >= 9.0 && r.w.is_finite() && r.h.is_finite(), "{a} {w} {h}");
            }
        }
    }

    #[test]
    fn each_part_is_found_corners_first() {
        let c = crop();
        assert_eq!(c.handle_at(101.0, 101.0, 10.0), Some(Handle::Nw));
        assert_eq!(c.handle_at(299.0, 199.0, 10.0), Some(Handle::Se));
        assert_eq!(c.handle_at(200.0, 98.0, 10.0), Some(Handle::N));
        assert_eq!(c.handle_at(302.0, 150.0, 10.0), Some(Handle::E));
        assert_eq!(c.handle_at(200.0, 150.0, 10.0), Some(Handle::Body));
        assert_eq!(c.handle_at(10.0, 10.0, 10.0), None);
        assert_eq!(Handle::Sw.cursor(), Cursor::SwResize);
    }

    #[test]
    fn a_tiny_box_keeps_a_middle_to_move_it_by() {
        let mut c = crop();
        c.set_rect(Rect { x: 100.0, y: 100.0, w: 16.0, h: 16.0 });
        assert_eq!(c.handle_at(108.0, 108.0, 12.0), Some(Handle::Body));
    }

    #[test]
    fn a_free_pull_moves_only_its_edges_and_stops_at_the_bounds_and_least() {
        let mut c = crop();
        c.pull(Handle::E, 350.0, 5.0);
        assert_eq!(c.rect(), Rect { x: 100.0, y: 100.0, w: 250.0, h: 100.0 });
        c.pull(Handle::Nw, -50.0, -50.0);
        assert_eq!(c.rect(), Rect { x: 0.0, y: 0.0, w: 350.0, h: 200.0 });
        c.pull(Handle::W, 9000.0, 0.0);
        assert_eq!(c.rect().w, 16.0, "never under the least");
        assert!(inside(&c));
        c.pull(Handle::S, 9000.0, 9000.0);
        assert!(inside(&c));
    }

    #[test]
    fn a_held_ratio_keeps_to_it_whichever_handle_is_pulled() {
        for h in [Handle::Nw, Handle::N, Handle::Ne, Handle::E, Handle::Se, Handle::S, Handle::Sw, Handle::W] {
            for to in [(0.0, 0.0), (400.0, 300.0), (150.0, 130.0), (390.0, 20.0), (-9.0, 500.0)] {
                let mut c = crop();
                c.set_aspect(Some(1.0));
                c.pull(h, to.0, to.1);
                let r = c.rect();
                assert!((r.w / r.h - 1.0).abs() < 0.001, "{h:?} to {to:?} gave {r:?}");
                assert!(inside(&c), "{h:?} to {to:?} gave {:?}", c.rect());
                assert!(r.w >= 15.99 && r.h >= 15.99, "{h:?} to {to:?} gave {r:?}");
            }
        }
    }

    #[test]
    fn a_corner_pull_with_a_ratio_leaves_the_opposite_corner_still() {
        let mut c = crop();
        c.set_aspect(Some(2.0));
        let before = c.rect();
        c.pull(Handle::Se, 280.0, 400.0);
        let r = c.rect();
        assert_eq!((r.x, r.y), (before.x, before.y));
        assert!((r.w / r.h - 2.0).abs() < 0.001);
    }

    #[test]
    fn setting_a_ratio_shrinks_the_box_about_its_middle() {
        let mut c = crop();
        c.set_aspect(Some(1.0));
        assert_eq!(c.rect(), Rect { x: 150.0, y: 100.0, w: 100.0, h: 100.0 });
        c.set_aspect(None);
        assert_eq!(c.rect().w, 100.0, "freed, it stays as it is");
        c.set_aspect(Some(f32::NAN));
        assert_eq!(c.aspect, None);
        c.reset();
        assert_eq!(c.rect().w, 400.0);
    }

    #[test]
    fn the_keyboard_moves_and_grows_the_box_inside_the_bounds() {
        let mut c = crop();
        c.move_by(1000.0, -1000.0);
        assert_eq!(c.rect(), Rect { x: 200.0, y: 0.0, w: 200.0, h: 100.0 });
        c.grow(10.0, 10.0);
        assert_eq!((c.rect().w, c.rect().h), (200.0, 110.0), "no wider at the right edge, taller");
        c.move_by(-100.0, 0.0);
        c.grow(10.0, 10.0);
        assert_eq!(c.rect(), Rect { x: 100.0, y: 0.0, w: 210.0, h: 120.0 });
        c.grow(-1000.0, -1000.0);
        assert_eq!((c.rect().w, c.rect().h), (16.0, 16.0));
    }

    #[test]
    fn the_pointer_pulls_a_handle_and_moves_the_body_through_the_zoom() {
        let mut g = Gesture::new();
        let view = View::new((800.0, 600.0), (200.0, 150.0));
        assert_eq!(view.scale(), 0.25);
        let mut c2 = Crop::new(Rect { x: 0.0, y: 0.0, w: 800.0, h: 600.0 });
        c2.set_rect(Rect { x: 200.0, y: 200.0, w: 400.0, h: 200.0 });
        assert!(!g.pointer(&mut c2, &view, Pointer::Down(50.0, 50.0)), "the press changes nothing");
        assert_eq!(g.holding(), Some(Handle::Nw));
        assert_eq!(g.cursor(&c2, &view, 50.0, 50.0), Cursor::NwResize);
        assert!(g.pointer(&mut c2, &view, Pointer::Move(25.0, 25.0)));
        assert_eq!(c2.rect(), Rect { x: 100.0, y: 100.0, w: 500.0, h: 300.0 });
        g.pointer(&mut c2, &view, Pointer::Up(25.0, 25.0));
        assert_eq!(g.holding(), None);
        g.pointer(&mut c2, &view, Pointer::Down(90.0, 60.0));
        assert_eq!(g.holding(), Some(Handle::Body));
        g.pointer(&mut c2, &view, Pointer::Move(110.0, 60.0));
        assert_eq!(c2.rect().x, 180.0);
        g.pointer(&mut c2, &view, Pointer::Move(9000.0, 60.0));
        assert_eq!(c2.rect().x, 300.0, "stops at the picture's edge");
        g.pointer(&mut c2, &view, Pointer::Up(0.0, 0.0));
        assert!(!g.pointer(&mut c2, &view, Pointer::Down(0.0, 149.0)));
        assert_eq!(g.holding(), None, "a press off the box holds nothing");
    }

    #[test]
    fn the_box_is_painted_dimmed_outside_and_handled() {
        use super::super::theme::Theme;
        let theme = Theme::new(true, "#fbc711");
        let (fonts, icons) = (super::super::text::Fonts::new(), super::super::icon::Icons::new());
        let pen = Pen { theme: &theme, fonts: &fonts, icons: &icons, scale: 1.0 };
        let mut data = vec![0u8; 200 * 150 * 4];
        let mut canvas = PixmapMut::from_bytes(&mut data, 200, 150).unwrap();
        let outer = Rect { x: 0.0, y: 0.0, w: 200.0, h: 150.0 };
        paint_crop(&mut canvas, &pen, outer, Rect { x: 50.0, y: 40.0, w: 100.0, h: 70.0 });
        let a = |canvas: &mut PixmapMut, x: usize, y: usize| canvas.pixels_mut()[y * 200 + x].alpha();
        assert!(a(&mut canvas, 5, 5) > 100, "outside is dimmed");
        assert_eq!(a(&mut canvas, 100, 75), 0, "inside is clear");
        assert!(a(&mut canvas, 100, 41) > 200, "the frame");
        assert!(a(&mut canvas, 50, 40) > 200, "a corner handle");
    }
}
