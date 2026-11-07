//! Soft shadows as coverage masks, blurred, made once and kept as a ready picture in their colour; a mask is made only for a box small enough to hold its corners and stretched across the middle.

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use tiny_skia::{FillRule, Mask, PixmapMut, Stroke, Transform};

use crate::paint::{played, Painter};

use super::colour::Colour;
use super::theme::{Fall, Theme};
use super::ui::rounded;
use super::widget::Rect;

const MOST: usize = 48;

pub(crate) struct Made {
    w: usize,
    h: usize,
    at: (usize, usize),
    boxed: (usize, usize),
    step: usize,
    cover: Box<[u8]>,
    ready: RefCell<Option<([u8; 4], Rc<[[u8; 4]]>)>>,
}

impl Made {
    fn new(w: usize, h: usize, at: (usize, usize), boxed: (usize, usize), step: usize, cover: Box<[u8]>) -> Made {
        Made { w, h, at, boxed, step, cover, ready: RefCell::default() }
    }

    fn ready(&self, c: [u8; 4]) -> Rc<[[u8; 4]]> {
        if let Some((was, pic)) = &*self.ready.borrow() {
            if *was == c {
                return pic.clone();
            }
        }
        let tones: Vec<[u8; 4]> = (0..=255u32).map(|v| {
            let a = v * u32::from(c[3]) / 255;
            let k = |n: u8| (u32::from(n) * a / 255) as u8;
            [k(c[0]), k(c[1]), k(c[2]), a as u8]
        }).collect();
        let across = self.w.div_ceil(self.step);
        let pic: Rc<[[u8; 4]]> = self.cover.chunks_exact(across)
            .flat_map(|row| (0..self.w).map(|x| tones[usize::from(row[x / self.step])])).collect();
        *self.ready.borrow_mut() = Some((c, pic.clone()));
        pic
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
struct Key {
    inside: bool,
    w: u32,
    h: u32,
    radius: u32,
    falls: [(i32, i32, i32, i32); 3],
}

#[derive(Default)]
pub struct Shadows {
    made: RefCell<HashMap<Key, Rc<Made>>>,
    allowed: std::cell::Cell<Option<u64>>,
}

impl Shadows {
    pub fn below(&self, canvas: &mut PixmapMut, r: Rect, radius: f32, falls: &[Fall], colour: Colour) {
        if super::theme::drawn_by_processor() {
            return;
        }
        let (w, h) = (r.w.round().max(1.0) as usize, r.h.round().max(1.0) as usize);
        let Some(made) = self.get(false, w, h, radius, falls) else { return };
        lay(canvas, &made, (whole(r.x), whole(r.y)), (w, h), colour, Some(radius.round() as usize));
    }

    pub(crate) fn inside(&self, canvas: &mut PixmapMut, r: Rect, radius: f32, fall: Fall, colour: Colour) {
        if super::theme::drawn_by_processor() {
            return;
        }
        let (w, h) = (r.w.round().max(1.0) as usize, r.h.round().max(1.0) as usize);
        let Some(made) = self.get(true, w, h, radius, &[fall]) else { return };
        lay(canvas, &made, (whole(r.x), whole(r.y)), (w, h), colour, None);
    }

    fn get(&self, inside: bool, w: usize, h: usize, radius: f32, falls: &[Fall]) -> Option<Rc<Made>> {
        let (bw, bh) = kept_box(w, h, radius, falls);
        let falls = &falls[..falls.len().min(3)];
        let tenths = |v: f32| (v * 10.0).round() as i32;
        let mut numbers = [(0, 0, 0, 0); 3];
        for (n, f) in numbers.iter_mut().zip(falls) {
            *n = (tenths(f.y), tenths(f.blur), tenths(f.spread), (f.alpha * 1000.0).round() as i32);
        }
        let key = Key { inside, w: bw as u32, h: bh as u32, radius: (radius * 10.0).round() as u32, falls: numbers };
        if let Some(m) = self.made.borrow().get(&key) {
            return Some(m.clone());
        }
        if let Some(left) = self.allowed.get() {
            let pixels = mask_pixels(inside, Rect { x: 0.0, y: 0.0, w: bw as f32, h: bh as f32 }, radius, falls);
            if pixels > left {
                return None;
            }
            self.allowed.set(Some(left - pixels));
        }
        let made = Rc::new(if inside { make_inside(bw, bh, radius, falls[0])? } else { make_below(bw, bh, radius, falls)? });
        let mut all = self.made.borrow_mut();
        if all.len() >= MOST {
            all.clear();
        }
        all.insert(key, made.clone());
        Some(made)
    }
}

fn kept_box(w: usize, h: usize, radius: f32, falls: &[Fall]) -> (usize, usize) {
    let reach = falls.iter().map(|f| f.y.abs() + f.spread.abs() + 3.0 * blur_radius(f.blur) as f32).fold(0.0, f32::max);
    let edge = (reach + radius).ceil() as usize + 2;
    (w.min(2 * edge + 1), h.min(2 * edge + 1))
}

pub(crate) fn mask_pixels(inside: bool, r: Rect, radius: f32, falls: &[Fall]) -> u64 {
    let falls = &falls[..falls.len().min(if inside { 1 } else { 3 })];
    let (w, h) = kept_box(r.w.round().max(1.0) as usize, r.h.round().max(1.0) as usize, radius, falls);
    let (w, h) = (w as u64, h as u64);
    if inside {
        let room = falls.first().map_or(0, |f| 3 * blur_radius(f.blur) as u64 + f.y.abs().ceil() as u64 + 1);
        return (w + 2 * room) * (h + 2 * room);
    }
    let side = falls.iter().map(|f| (f.spread + 3.0 * blur_radius(f.blur) as f32).max(0.0)).fold(0.0, f32::max).ceil() as u64 + 1;
    let down = falls.iter().map(|f| f.y.max(0.0)).fold(0.0, f32::max).ceil() as u64;
    let step = (falls.iter().map(|f| blur_radius(f.blur)).min().unwrap_or(0) / 4).clamp(1, 4) as u64;
    (w + 2 * side).div_ceil(step) * (h + 2 * side + down).div_ceil(step)
}

impl Shadows {
    pub(crate) fn allow(&self, pixels: u64) {
        self.allowed.set(Some(pixels));
    }

    pub(crate) fn bytes(&self) -> usize {
        self.made.borrow().values().map(|m| m.cover.len()).sum()
    }

    pub(crate) fn clear(&self) {
        self.made.borrow_mut().clear();
    }
}

impl Made {
    pub(crate) fn parts(&self) -> (usize, usize, (usize, usize), (usize, usize), usize, &[u8]) {
        (self.w, self.h, self.at, self.boxed, self.step, &self.cover)
    }
}

impl Shadows {
    pub(crate) fn plan(&self, inside: bool, r: Rect, radius: f32, falls: &[Fall]) -> Option<(Rc<Made>, (i32, i32), (usize, usize), Option<usize>)> {
        if super::theme::drawn_by_processor() || falls.is_empty() {
            return None;
        }
        let (w, h) = (r.w.round().max(1.0) as usize, r.h.round().max(1.0) as usize);
        let made = self.get(inside, w, h, radius, if inside { &falls[..1] } else { falls })?;
        Some((made, (whole(r.x), whole(r.y)), (w, h), (!inside).then(|| radius.round() as usize)))
    }
}

fn whole(v: f32) -> i32 {
    (v + 0.5).floor() as i32
}

thread_local! {
    static FOR_DRAWINGS: Shadows = Shadows::default();
}

pub fn below_for_drawing(canvas: &mut PixmapMut, theme: &Theme, list: &[Fall], r: Rect, radius: f32, scale: f32) {
    FOR_DRAWINGS.with(|s| played(canvas, s, |p| below_on(p, theme, list, r, radius, scale)));
}

pub(crate) fn below_on(p: &mut Painter, theme: &Theme, list: &[Fall], r: Rect, radius: f32, scale: f32) {
    let falls: Vec<Fall> = list.iter()
        .map(|f| Fall { y: f.y * scale, blur: f.blur * scale, spread: f.spread * scale, alpha: f.alpha }).collect();
    if !falls.is_empty() {
        p.shadow(r, radius, &falls, theme.shadow);
    }
}

pub fn edge_round(canvas: &mut PixmapMut, theme: &Theme, r: Rect, scale: f32) {
    FOR_DRAWINGS.with(|s| played(canvas, s, |p| edge_round_on(p, theme, r, scale)));
}

pub(crate) fn edge_round_on(p: &mut Painter, theme: &Theme, r: Rect, scale: f32) {
    let radius = crate::theme::corner_px(crate::theme::Part::Picture, r.w, scale);
    below_on(p, theme, &theme.raised_falls, r, radius, scale);
    let line = scale.round().max(1.0);
    if let Some(path) = crate::ui::rounded(r.inset(-line / 2.0), radius + line / 2.0) {
        p.stroke_path(path, Stroke { width: line, ..Stroke::default() }, theme.card_edge);
    }
}

fn blur_radius(blur: f32) -> usize {
    (blur / 2.0).round().max(0.0) as usize
}

fn shape(mw: usize, mh: usize, x: f32, y: f32, w: f32, h: f32, radius: f32) -> Option<Vec<f32>> {
    let mut mask = Mask::new(mw as u32, mh as u32)?;
    if w > 0.0 && h > 0.0 {
        let path = rounded(Rect { x, y, w, h }, radius.max(0.0))?;
        mask.fill_path(&path, FillRule::Winding, true, Transform::identity());
    }
    Some(mask.data().iter().map(|&a| a as f32 / 255.0).collect())
}

fn make_below(w: usize, h: usize, radius: f32, falls: &[Fall]) -> Option<Made> {
    let side = falls.iter().map(|f| (f.spread + 3.0 * blur_radius(f.blur) as f32).max(0.0)).fold(0.0, f32::max).ceil() as usize + 1;
    let down = falls.iter().map(|f| f.y.max(0.0)).fold(0.0, f32::max).ceil() as usize;
    let (mw, mh) = (w + 2 * side, h + 2 * side + down);
    let step = (falls.iter().map(|f| blur_radius(f.blur)).min().unwrap_or(0) / 4).clamp(1, 4);
    let k = step as f32;
    let (sw, sh) = (mw.div_ceil(step), mh.div_ceil(step));
    let mut total = vec![0.0f32; sw * sh];
    for f in falls {
        let mut a = shape(sw, sh, (side as f32 - f.spread) / k, (side as f32 + f.y - f.spread) / k,
                          (w as f32 + 2.0 * f.spread) / k, (h as f32 + 2.0 * f.spread) / k, (radius + f.spread) / k)?;
        blur(&mut a, sw, sh, (blur_radius(f.blur) as f32 / k).round() as usize);
        for (t, v) in total.iter_mut().zip(&a) {
            *t = 1.0 - (1.0 - *t) * (1.0 - v * f.alpha);
        }
    }
    let cover = total.iter().map(|v| (v * 255.0).round().clamp(0.0, 255.0) as u8).collect();
    Some(Made::new(mw, mh, (side, side), (w, h), step, cover))
}

fn make_inside(w: usize, h: usize, radius: f32, f: Fall) -> Option<Made> {
    let room = 3 * blur_radius(f.blur) + f.y.abs().ceil() as usize + 1;
    let (mw, mh) = (w + 2 * room, h + 2 * room);
    let mut a: Vec<f32> = shape(mw, mh, room as f32, room as f32 + f.y, w as f32, h as f32, radius)?.iter().map(|v| 1.0 - v).collect();
    blur(&mut a, mw, mh, blur_radius(f.blur));
    let own = shape(mw, mh, room as f32, room as f32, w as f32, h as f32, radius)?;
    let mut cover = vec![0u8; w * h];
    for y in 0..h {
        for x in 0..w {
            let i = (y + room) * mw + x + room;
            cover[y * w + x] = (a[i] * own[i] * f.alpha * 255.0).round().clamp(0.0, 255.0) as u8;
        }
    }
    Some(Made::new(w, h, (0, 0), (w, h), 1, cover.into()))
}

pub(crate) fn blur(a: &mut [f32], w: usize, h: usize, r: usize) {
    if r == 0 {
        return;
    }
    let mut line = vec![0.0f32; w.max(h)];
    for _ in 0..3 {
        for y in 0..h {
            pass(&mut a[y * w..(y + 1) * w], 1, w, r, &mut line);
        }
        for x in 0..w {
            pass(&mut a[x..], w, h, r, &mut line);
        }
    }
}

fn pass(a: &mut [f32], step: usize, n: usize, r: usize, line: &mut [f32]) {
    for i in 0..n {
        line[i] = a[i * step];
    }
    let k = 1.0 / (2 * r + 1) as f32;
    let get = |i: isize| if i < 0 || i as usize >= n { 0.0 } else { line[i as usize] };
    let mut sum: f32 = (-(r as isize)..=r as isize).map(get).sum();
    for i in 0..n {
        a[i * step] = sum * k;
        let i = i as isize;
        sum += get(i + r as isize + 1) - get(i - r as isize);
    }
}

fn lay(canvas: &mut PixmapMut, made: &Made, at: (i32, i32), size: (usize, usize), colour: Colour, hollow: Option<usize>) {
    let (cw, ch) = (canvas.width() as i32, canvas.height() as i32);
    let extra = (size.0.saturating_sub(made.boxed.0), size.1.saturating_sub(made.boxed.1));
    let (w, h) = (made.w + extra.0, made.h + extra.1);
    let mid = (made.at.0 + made.boxed.0 / 2, made.at.1 + made.boxed.1 / 2);
    let pick = |u: usize, mid: usize, extra: usize| if u < mid { u } else if u < mid + extra { mid } else { u - extra };
    let (ox, oy) = (at.0 - made.at.0 as i32, at.1 - made.at.1 as i32);
    let c = colour.to_skia().premultiply().to_color_u8();
    let pic = made.ready([c.red(), c.green(), c.blue(), c.alpha()]);
    let data = canvas.data_mut();
    let (u0, u1) = ((-ox).clamp(0, w as i32) as usize, (cw - ox).clamp(0, w as i32) as usize);
    if u0 >= u1 {
        return;
    }
    let stretch = (mid.0, mid.0 + extra.0);
    for v in 0..h {
        let y = oy + v as i32;
        if y < 0 || y >= ch {
            continue;
        }
        let row = &pic[(pick(v, mid.1, extra.1) / made.step) * made.w..][..made.w];
        let line = &mut data[(y * cw + ox.max(0)) as usize * 4..][..(u1 - u0) * 4];
        let inner = hollow.and_then(|r| {
            let top = made.at.1 + r;
            let bottom = (made.at.1 + size.1).saturating_sub(r);
            (v >= top && v < bottom).then_some((made.at.0, made.at.0 + size.0))
        });
        let (skip_from, skip_to) = inner.map_or((u1, u1), |(a, b)| (a.clamp(u0, u1), b.clamp(u0, u1)));
        for (s, e) in [(u0, skip_from), (skip_to, u1)] {
            let px = |a: usize, b: usize| (a - u0) * 4..(b - u0) * 4;
            let (a, b) = (s, e.min(stretch.0));
            if a < b {
                over(&mut line[px(a, b)], &row[a..b]);
            }
            let (a, b) = (s.max(stretch.0), e.min(stretch.1));
            if a < b {
                even(&mut line[px(a, b)], row[mid.0]);
            }
            let (a, b) = (s.max(stretch.1), e);
            if a < b {
                over(&mut line[px(a, b)], &row[a - extra.0..b - extra.0]);
            }
        }
    }
}

fn over(dst: &mut [u8], src: &[[u8; 4]]) {
    for (d, s) in dst.chunks_exact_mut(4).zip(src) {
        mix(d, *s);
    }
}

fn even(dst: &mut [u8], s: [u8; 4]) {
    let mut last: Option<([u8; 4], [u8; 4])> = None;
    for d in dst.chunks_exact_mut(4) {
        let was = [d[0], d[1], d[2], d[3]];
        match last {
            Some((under, now)) if under == was => d.copy_from_slice(&now),
            _ => {
                mix(d, s);
                last = Some((was, [d[0], d[1], d[2], d[3]]));
            }
        }
    }
}

#[inline(always)]
fn mix(d: &mut [u8], s: [u8; 4]) {
    const LOW: u64 = 0x00ff_00ff_00ff_00ff;
    const NINTH: u64 = 0x0100_0100_0100_0100;
    let wide = |b: [u8; 4]| b.iter().rev().fold(0u64, |n, &v| (n << 16) | u64::from(v));
    let Ok(px) = <&mut [u8; 4]>::try_from(d) else { return };
    let y = wide(*px) * (255 - u64::from(s[3])) + 0x0080_0080_0080_0080;
    let r = (((y + ((y >> 8) & LOW)) >> 8) & LOW) + wide(s);
    let high = r & NINTH;
    let r = (r | (high - (high >> 8))) & LOW;
    *px = [r as u8, (r >> 16) as u8, (r >> 32) as u8, (r >> 48) as u8];
}

#[cfg(test)]
mod tests {
    use super::*;
    use tiny_skia::Pixmap;

    fn falls() -> Vec<Fall> {
        vec![Fall { y: 10.0, blur: 16.0, spread: 0.0, alpha: 0.6 }, Fall { y: 34.0, blur: 60.0, spread: -8.0, alpha: 0.85 }]
    }

    #[test]
    fn a_shadow_is_soft_falls_below_and_costs_the_same_for_any_size() {
        let s = Shadows::default();
        let mut pix = Pixmap::new(800, 600).unwrap();
        pix.fill(tiny_skia::Color::WHITE);
        let r = Rect { x: 200.0, y: 100.0, w: 400.0, h: 300.0 };
        s.below(&mut pix.as_mut(), r, 22.0, &falls(), Colour::rgb(0.0, 0.0, 0.0));
        let at = |x: u32, y: u32| pix.pixel(x, y).unwrap().red();
        assert!(at(400, 410) < at(400, 440) && at(400, 440) < at(400, 520), "{} {} {}", at(400, 410), at(400, 440), at(400, 520));
        assert_eq!(at(400, 40), 255);
        assert!(at(400, 410) < 200);
        assert_eq!(at(400, 250), 255);
        let mut big = Pixmap::new(1600, 600).unwrap();
        s.below(&mut big.as_mut(), Rect { w: 1200.0, ..r }, 22.0, &falls(), Colour::rgb(0.0, 0.0, 0.0));
        assert_eq!(s.made.borrow().len(), 1);
        let (wide, narrow) = (big.pixel(800, 430).unwrap().alpha() as i32, 255 - pix.pixel(400, 430).unwrap().red() as i32);
        assert!((wide - narrow).abs() <= 1, "{wide} {narrow}");
    }

    #[test]
    fn an_edged_picture_plays_as_it_was_drawn_straight() {
        let theme = Theme::chosen();
        let r = Rect { x: 20.3, y: 18.6, w: 60.0, h: 60.0 };
        let mut played = Pixmap::new(120, 120).unwrap();
        played.fill(tiny_skia::Color::WHITE);
        let mut direct = played.clone();
        edge_round(&mut played.as_mut(), &theme, r, 1.5);
        let radius = crate::theme::corner_px(crate::theme::Part::Picture, r.w, 1.5);
        let falls: Vec<Fall> = theme.raised_falls.iter().map(|f| Fall { y: f.y * 1.5, blur: f.blur * 1.5, spread: f.spread * 1.5, alpha: f.alpha }).collect();
        if !falls.is_empty() {
            Shadows::default().below(&mut direct.as_mut(), r, radius, &falls, theme.shadow);
        }
        let path = rounded(r.inset(-1.0), radius + 1.0).unwrap();
        direct.stroke_path(&path, &crate::colour::paint(theme.card_edge), &Stroke { width: 2.0, ..Stroke::default() }, Transform::identity(), None);
        assert!(played.data() == direct.data());
    }

    #[test]
    fn a_sunk_box_is_darkest_under_its_top_edge() {
        let s = Shadows::default();
        let mut pix = Pixmap::new(300, 60).unwrap();
        pix.fill(tiny_skia::Color::WHITE);
        let r = Rect { x: 10.0, y: 10.0, w: 280.0, h: 36.0 };
        s.inside(&mut pix.as_mut(), r, 14.0, Fall { y: 2.0, blur: 5.0, spread: 0.0, alpha: 0.5 }, Colour::rgb(0.0, 0.0, 0.0));
        let at = |y: u32| pix.pixel(150, y).unwrap().red();
        assert!(at(10) < at(14) && at(14) < at(30), "{} {} {}", at(10), at(14), at(30));
        assert_eq!(at(5), 255);
        assert_eq!(at(55), 255);
    }

    #[test]
    #[ignore = "measures time; it checks nothing"]
    fn measure_card_shadows() {
        use crate::widget::{card, grid, space, Length};
        let cpu = crate::sys::os::thread_time;
        let median = |mut v: Vec<f64>| { v.sort_by(|a, b| a.partial_cmp(b).unwrap()); v[v.len() / 2] };
        let cards = (0..24).map(|_| card(space::<()>()).height(Length::Fixed(220.0))).collect();
        let mut ui = crate::ui::Ui::new(grid(280.0, cards), Theme::chosen());
        ui.lay_at(1920, 1080);
        let mut pix = Pixmap::new(1920, 1080).unwrap();
        let paints: Vec<f64> = (0..60).map(|_| { let t = cpu(); ui.paint(&mut pix.as_mut(), 1.0); (cpu() - t).as_secs_f64() * 1000.0 }).collect();
        let theme = Theme::chosen();
        let s = Shadows::default();
        let r = Rect { x: 100.0, y: 100.0, w: 440.0, h: 220.0 };
        let lays: Vec<f64> = (0..60).map(|_| {
            let t = cpu();
            for _ in 0..24 {
                s.below(&mut pix.as_mut(), r, 16.0, &theme.panel_falls, theme.shadow);
            }
            (cpu() - t).as_secs_f64() * 1000.0
        }).collect();
        println!("24 empty cards, whole paint: median {:.2} ms; 24 card shadows alone: median {:.2} ms", median(paints), median(lays));
    }
}
