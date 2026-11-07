//! Zooming and panning a picture a program paints itself, in points from the drawing's top left.

use std::time::{Duration, Instant};

use super::widget::{Pointer, Rect};

const MOST: f32 = 16.0;
const STEP: f32 = 1.25;
const LINE: f32 = 1.1;
const SLOP: f32 = 3.0;
const GLIDE: f32 = 0.3;
const FLICK: f32 = 120.0;
const STILL: f32 = 12.0;
const PAUSE: Duration = Duration::from_millis(80);

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct View {
    content: (f32, f32),
    area: (f32, f32),
    zoom: Option<f32>,
    offset: (f32, f32),
    turns: u8,
}

impl View {
    pub fn new(content: (f32, f32), area: (f32, f32)) -> Self {
        let mut v = View { content, area, zoom: None, offset: (0.0, 0.0), turns: 0 };
        v.settle();
        v
    }

    pub fn set_area(&mut self, area: (f32, f32)) {
        self.area = area;
        self.settle();
    }

    pub fn set_content(&mut self, content: (f32, f32)) {
        self.content = content;
        self.turns = 0;
        self.zoom = None;
        self.settle();
    }

    pub fn turn_right(&mut self) {
        self.turns = (self.turns + 1) % 4;
        self.fit();
    }

    pub fn turn_left(&mut self) {
        self.turns = (self.turns + 3) % 4;
        self.fit();
    }

    pub fn turns(&self) -> u8 {
        self.turns
    }

    fn shown(&self) -> (f32, f32) {
        if self.turns % 2 == 1 { (self.content.1, self.content.0) } else { self.content }
    }

    pub fn fit_scale(&self) -> f32 {
        let c = self.shown();
        if c.0 <= 0.0 || c.1 <= 0.0 || self.area.0 <= 0.0 || self.area.1 <= 0.0 {
            return 1.0;
        }
        (self.area.0 / c.0).min(self.area.1 / c.1).min(1.0)
    }

    pub fn scale(&self) -> f32 {
        self.zoom.unwrap_or_else(|| self.fit_scale())
    }

    pub fn is_fitted(&self) -> bool {
        self.zoom.is_none()
    }

    pub fn can_pan(&self) -> bool {
        let (s, c) = (self.scale(), self.shown());
        c.0 * s > self.area.0 + 0.5 || c.1 * s > self.area.1 + 0.5
    }

    pub fn fit(&mut self) {
        self.zoom = None;
        self.settle();
    }

    pub fn actual(&mut self, about: (f32, f32)) {
        self.zoom_to(1.0, about);
    }

    pub fn zoom_in(&mut self, about: (f32, f32)) {
        self.zoom_to(self.scale() * STEP, about);
    }

    pub fn zoom_out(&mut self, about: (f32, f32)) {
        self.zoom_to(self.scale() / STEP, about);
    }

    pub fn wheel(&mut self, lines: f32, about: (f32, f32)) {
        self.zoom_to(self.scale() * LINE.powf(-lines.clamp(-100.0, 100.0)), about);
    }

    pub fn zoom_to(&mut self, scale: f32, about: (f32, f32)) {
        let fit = self.fit_scale();
        let top = MOST.max(fit);
        let scale = if scale.is_finite() { scale.clamp(fit, top) } else { self.scale() };
        let unit = self.to_content(about.0, about.1);
        if scale <= fit * 1.0001 {
            self.fit();
            return;
        }
        self.zoom = Some(scale);
        self.offset = (about.0 - unit.0 * scale, about.1 - unit.1 * scale);
        self.settle();
    }

    pub fn pan_by(&mut self, dx: f32, dy: f32) -> (f32, f32) {
        if self.zoom.is_none() {
            return (0.0, 0.0);
        }
        let before = self.offset;
        self.offset = (before.0 + dx, before.1 + dy);
        self.settle();
        (self.offset.0 - before.0, self.offset.1 - before.1)
    }

    pub fn content_rect(&self) -> Rect {
        let (s, c) = (self.scale(), self.shown());
        Rect { x: self.offset.0, y: self.offset.1, w: c.0 * s, h: c.1 * s }
    }

    pub fn visible(&self) -> Rect {
        let s = self.scale();
        let shown = self.content_rect().intersect(&Rect { x: 0.0, y: 0.0, w: self.area.0, h: self.area.1 });
        Rect { x: (shown.x - self.offset.0) / s, y: (shown.y - self.offset.1) / s, w: shown.w / s, h: shown.h / s }
    }

    pub fn to_content(&self, x: f32, y: f32) -> (f32, f32) {
        let s = self.scale();
        ((x - self.offset.0) / s, (y - self.offset.1) / s)
    }

    pub fn to_screen(&self, x: f32, y: f32) -> (f32, f32) {
        let s = self.scale();
        (x * s + self.offset.0, y * s + self.offset.1)
    }

    pub fn rect_to_screen(&self, r: Rect) -> Rect {
        let s = self.scale();
        let (x, y) = self.to_screen(r.x, r.y);
        Rect { x, y, w: r.w * s, h: r.h * s }
    }

    fn settle(&mut self) {
        let (s, c) = (self.scale(), self.shown());
        let axis = |content: f32, area: f32, at: f32| {
            let size = content * s;
            if size <= area { (area - size) / 2.0 } else { at.clamp(area - size, 0.0) }
        };
        self.offset = (axis(c.0, self.area.0, self.offset.0), axis(c.1, self.area.1, self.offset.1));
    }
}

#[derive(Debug, Default)]
pub struct Drag {
    from: Option<(f32, f32)>,
    at: (f32, f32),
    moved: bool,
    last: Option<Instant>,
    speed: (f32, f32),
    flung: (f32, f32),
}

impl Drag {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn pointer(&mut self, view: &mut View, event: Pointer) -> bool {
        self.pointer_at(view, event, Instant::now())
    }

    pub fn pointer_at(&mut self, view: &mut View, event: Pointer, now: Instant) -> bool {
        match event {
            Pointer::Down(x, y) => {
                self.from = view.can_pan().then_some((x, y));
                self.at = (x, y);
                self.moved = false;
                self.last = Some(now);
                self.speed = (0.0, 0.0);
                self.flung = (0.0, 0.0);
                false
            }
            Pointer::Move(x, y) => {
                let Some(from) = self.from else { return false };
                if !self.moved && (x - from.0).hypot(y - from.1) <= SLOP {
                    return false;
                }
                self.moved = true;
                let (dx, dy) = (x - self.at.0, y - self.at.1);
                self.at = (x, y);
                let secs = self.last.map_or(0.0, |t| now.saturating_duration_since(t).as_secs_f32()).max(0.001);
                self.last = Some(now);
                self.speed = (self.speed.0 * 0.4 + dx / secs * 0.6, self.speed.1 * 0.4 + dy / secs * 0.6);
                view.pan_by(dx, dy);
                true
            }
            Pointer::Up(..) => {
                let fast = self.speed.0.hypot(self.speed.1) >= FLICK;
                let fresh = self.last.is_some_and(|t| now.saturating_duration_since(t) <= PAUSE);
                if self.from.is_some() && self.moved && fast && fresh && !super::motion::reduced() {
                    self.flung = self.speed;
                }
                self.from = None;
                false
            }
            _ => false,
        }
    }

    pub fn holding(&self) -> bool {
        self.from.is_some()
    }

    pub fn glide(&mut self, view: &mut View, dt: Duration) -> bool {
        let secs = dt.as_secs_f32();
        let (want_x, want_y) = (self.flung.0 * secs, self.flung.1 * secs);
        let (got_x, got_y) = view.pan_by(want_x, want_y);
        if want_x != 0.0 && got_x == 0.0 {
            self.flung.0 = 0.0;
        }
        if want_y != 0.0 && got_y == 0.0 {
            self.flung.1 = 0.0;
        }
        let slow = (-secs / GLIDE).exp();
        self.flung = (self.flung.0 * slow, self.flung.1 * slow);
        if self.flung.0.hypot(self.flung.1) < STILL {
            self.flung = (0.0, 0.0);
        }
        got_x != 0.0 || got_y != 0.0
    }

    pub fn gliding(&self) -> bool {
        self.flung != (0.0, 0.0)
    }

    pub fn was_drag(&self) -> bool {
        self.moved
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn view() -> View {
        View::new((2000.0, 1000.0), (500.0, 400.0))
    }

    #[test]
    fn a_large_picture_is_fitted_whole_and_centred() {
        let v = view();
        assert_eq!((v.scale(), v.is_fitted()), (0.25, true));
        assert_eq!(v.content_rect(), Rect { x: 0.0, y: 75.0, w: 500.0, h: 250.0 });
        assert!(!v.can_pan());
    }

    #[test]
    fn a_small_picture_is_not_enlarged_to_fit() {
        let v = View::new((100.0, 50.0), (500.0, 400.0));
        assert_eq!(v.scale(), 1.0);
        assert_eq!(v.content_rect(), Rect { x: 200.0, y: 175.0, w: 100.0, h: 50.0 });
    }

    #[test]
    fn zoom_keeps_the_place_under_the_pointer() {
        let mut v = view();
        let about = (250.0, 200.0);
        let before = v.to_content(about.0, about.1);
        v.zoom_in(about);
        assert!(v.scale() > 0.25 && !v.is_fitted());
        let after = v.to_content(about.0, about.1);
        assert!((before.0 - after.0).abs() < 0.01 && (before.1 - after.1).abs() < 0.01);
        v.actual(about);
        assert_eq!(v.scale(), 1.0);
        assert!(v.can_pan());
    }

    #[test]
    fn zoom_out_to_the_fit_refits_and_never_goes_under() {
        let mut v = view();
        v.zoom_in((10.0, 10.0));
        for _ in 0..40 {
            v.zoom_out((10.0, 10.0));
        }
        assert!(v.is_fitted());
        assert_eq!(v.scale(), 0.25);
        v.wheel(-500.0, (0.0, 0.0));
        assert_eq!(v.scale(), MOST, "and never past the most");
        v.wheel(f32::NAN, (0.0, 0.0));
        assert_eq!(v.scale(), MOST, "a bad number changes nothing");
        v.wheel(1000.0, (0.0, 0.0));
        assert_eq!(v.scale(), 0.25);
        v.wheel(-1000.0, (0.0, 0.0));
        assert_eq!(v.scale(), MOST, "a huge turn goes to the most");
    }

    #[test]
    fn panning_stops_at_the_edges() {
        let mut v = view();
        v.actual((250.0, 200.0));
        v.pan_by(100000.0, 100000.0);
        let r = v.content_rect();
        assert_eq!((r.x, r.y), (0.0, 0.0), "top left of the picture at the top left");
        v.pan_by(-100000.0, -100000.0);
        let r = v.content_rect();
        assert_eq!((r.x + r.w, r.y + r.h), (500.0, 400.0));
        let seen = v.visible();
        assert_eq!((seen.w, seen.h), (500.0, 400.0));
    }

    #[test]
    fn a_resized_drawing_refits_a_fitted_picture_and_keeps_a_zoomed_one() {
        let mut v = view();
        v.set_area((1000.0, 800.0));
        assert_eq!(v.scale(), 0.5);
        v.actual((500.0, 400.0));
        v.set_area((600.0, 400.0));
        assert_eq!(v.scale(), 1.0);
        v.set_content((10.0, 10.0));
        assert!(v.is_fitted());
    }

    #[test]
    fn a_drag_moves_a_zoomed_picture_and_a_shake_does_not() {
        let (mut v, mut d) = (view(), Drag::new());
        d.pointer(&mut v, Pointer::Down(100.0, 100.0));
        assert!(!d.holding(), "a whole picture has nothing to move");
        v.actual((250.0, 200.0));
        let start = v.content_rect();
        d.pointer(&mut v, Pointer::Down(100.0, 100.0));
        assert!(d.holding());
        assert!(!d.pointer(&mut v, Pointer::Move(101.0, 101.0)), "a shake is no drag");
        assert!(d.pointer(&mut v, Pointer::Move(120.0, 90.0)));
        assert!(d.was_drag());
        let now = v.content_rect();
        assert_eq!((now.x - start.x, now.y - start.y), (20.0, -10.0));
        d.pointer(&mut v, Pointer::Up(120.0, 90.0));
        assert!(!d.holding());
    }

    #[test]
    fn a_turned_picture_swaps_its_sides_and_is_fitted_again() {
        let mut v = view();
        v.actual((250.0, 200.0));
        v.turn_right();
        assert_eq!((v.turns(), v.is_fitted()), (1, true));
        assert_eq!(v.scale(), 0.2);
        assert_eq!(v.content_rect(), Rect { x: 150.0, y: 0.0, w: 200.0, h: 400.0 });
        v.turn_left();
        v.turn_left();
        assert_eq!(v.turns(), 3);
        assert_eq!(v.scale(), 0.2);
        v.turn_right();
        assert_eq!((v.turns(), v.scale()), (0, 0.25));
        v.turn_right();
        v.set_content((10.0, 10.0));
        assert_eq!(v.turns(), 0, "a new picture is upright");
    }

    #[test]
    fn a_zoomed_turned_picture_pans_over_its_turned_size() {
        let mut v = view();
        v.turn_right();
        v.actual((250.0, 200.0));
        v.pan_by(-100000.0, -100000.0);
        let r = v.content_rect();
        assert_eq!((r.w, r.h, r.x + r.w, r.y + r.h), (1000.0, 2000.0, 500.0, 400.0));
    }

    #[test]
    fn panning_says_how_far_it_went() {
        let mut v = view();
        assert_eq!(v.pan_by(10.0, 10.0), (0.0, 0.0), "a whole picture does not move");
        v.actual((250.0, 200.0));
        v.pan_by(100000.0, 0.0);
        assert_eq!(v.pan_by(5.0, -7.0), (0.0, -7.0), "stopped at the left edge only");
    }

    fn fling(speed: f32) -> (View, Drag, Instant) {
        let (mut v, mut d) = (view(), Drag::new());
        v.actual((250.0, 200.0));
        let t = Instant::now();
        d.pointer_at(&mut v, Pointer::Down(250.0, 200.0), t);
        let mut now = t;
        for i in 1..=4 {
            now += Duration::from_millis(10);
            d.pointer_at(&mut v, Pointer::Move(250.0 - speed * 0.01 * i as f32, 200.0), now);
        }
        d.pointer_at(&mut v, Pointer::Up(0.0, 0.0), now + Duration::from_millis(5));
        (v, d, now)
    }

    #[test]
    fn a_fling_glides_on_slows_and_stops() {
        let (mut v, mut d, _) = fling(1000.0);
        assert!(d.gliding());
        let before = v.content_rect().x;
        let mut frames = 0;
        let mut last = before;
        while d.gliding() {
            assert!(d.glide(&mut v, Duration::from_millis(16)));
            let x = v.content_rect().x;
            assert!(x < last, "keeps going left");
            last = x;
            frames += 1;
            assert!(frames < 1000, "it stops of itself");
        }
        assert!(before - last > 100.0, "a fast fling goes a good way");
        assert!(!d.glide(&mut v, Duration::from_millis(16)));
    }

    #[test]
    fn a_slow_release_a_pause_a_new_press_and_less_motion_all_stop_a_glide() {
        let (_, d, _) = fling(50.0);
        assert!(!d.gliding(), "too slow to be a fling");
        let (mut v, mut d, now) = fling(1000.0);
        d.pointer_at(&mut v, Pointer::Down(1.0, 1.0), now + Duration::from_secs(1));
        assert!(!d.gliding(), "a new press catches it");
        let (mut v, mut d) = (view(), Drag::new());
        v.actual((250.0, 200.0));
        let t = Instant::now();
        d.pointer_at(&mut v, Pointer::Down(250.0, 200.0), t);
        d.pointer_at(&mut v, Pointer::Move(150.0, 200.0), t + Duration::from_millis(10));
        d.pointer_at(&mut v, Pointer::Up(150.0, 200.0), t + Duration::from_millis(500));
        assert!(!d.gliding(), "held still before letting go is a pause, not a fling");
        super::super::motion::reduce(true);
        let (_, d, _) = fling(1000.0);
        super::super::motion::reduce(false);
        assert!(!d.gliding(), "reduced motion: the picture stays where it was let go");
    }

    #[test]
    fn a_glide_stops_at_the_edge_of_the_picture() {
        let (mut v, mut d, _) = fling(4000.0);
        for _ in 0..600 {
            d.glide(&mut v, Duration::from_millis(16));
        }
        let r = v.content_rect();
        assert_eq!(r.x + r.w, 500.0, "its right edge against the drawing's");
        assert!(!d.gliding());
    }

    #[test]
    fn places_go_between_the_picture_and_the_drawing() {
        let mut v = view();
        v.actual((250.0, 200.0));
        let p = v.to_screen(1000.0, 500.0);
        assert_eq!(v.to_content(p.0, p.1), (1000.0, 500.0));
        let r = v.rect_to_screen(Rect { x: 0.0, y: 0.0, w: 10.0, h: 10.0 });
        assert_eq!((r.w, r.h), (10.0, 10.0));
    }
}
