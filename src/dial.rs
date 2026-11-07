//! Controls for a number: a dial turned by dragging round it, a level bar, a slider with marks under it and a volume button.

use std::f32::consts::{FRAC_PI_4, PI};
use std::rc::Rc;

use tiny_skia::{LineCap, PathBuilder, PixmapMut, Stroke, Transform};

use super::colour::{paint, Colour};
use super::input::{Input, Keysym};
use super::role::Role;
use super::theme;
use super::widget::{button, column, drawing, layers, note, placed, row, slider, Length, Node, Pointer, Rect};
use crate::t;

pub const DIAL_SIZE: f32 = 48.0;

const START: f32 = -3.0 * FRAC_PI_4;
const SWEEP: f32 = 1.5 * PI;

pub fn snap(value: f32, min: f32, max: f32, step: f32) -> f32 {
    let v = if step > 0.0 { min + ((value - min) / step).round() * step } else { value };
    v.clamp(min.min(max), max.max(min))
}

fn share(value: f32, min: f32, max: f32) -> f32 {
    if max > min { ((value - min) / (max - min)).clamp(0.0, 1.0) } else { 0.0 }
}

pub fn value_at(x: f32, y: f32, size: f32, min: f32, max: f32, step: f32) -> f32 {
    let (dx, dy) = (x - size / 2.0, y - size / 2.0);
    let angle = dx.atan2(-dy).clamp(START, START + SWEEP);
    snap(min + (angle - START) / SWEEP * (max - min), min, max, step)
}

pub fn value_after_key(input: &Input, value: f32, min: f32, max: f32, step: f32) -> Option<f32> {
    let Input::Key { sym, .. } = input else { return None };
    let to = match *sym {
        Keysym::Up | Keysym::Right => value + step,
        Keysym::Down | Keysym::Left => value - step,
        Keysym::Page_Up => value + step * 10.0,
        Keysym::Page_Down => value - step * 10.0,
        Keysym::Home => min,
        Keysym::End => max,
        _ => return None,
    };
    Some(snap(to, min, max, step))
}

fn arc(cx: f32, cy: f32, radius: f32, from: f32, to: f32) -> Option<tiny_skia::Path> {
    let mut pb = PathBuilder::new();
    let steps = (((to - from).abs() / PI * 24.0).ceil() as usize).max(1);
    for i in 0..=steps {
        let a = from + (to - from) * i as f32 / steps as f32;
        let (x, y) = (cx + radius * a.sin(), cy - radius * a.cos());
        if i == 0 { pb.move_to(x, y) } else { pb.line_to(x, y) }
    }
    pb.finish()
}

fn stroke(canvas: &mut PixmapMut, path: Option<tiny_skia::Path>, width: f32, colour: Colour) {
    if let Some(path) = path {
        let line = Stroke { width, line_cap: LineCap::Round, ..Stroke::default() };
        canvas.stroke_path(&path, &paint(colour), &line, Transform::identity(), None);
    }
}

pub fn dial<M: 'static>(value: f32, min: f32, max: f32, step: f32, tip: impl Into<String>, change: impl Fn(f32) -> M + 'static) -> Node<M> {
    let change = Rc::new(change);
    let (by_key, by_wheel) = (change.clone(), change.clone());
    let at = share(value, min, max);
    drawing((DIAL_SIZE, DIAL_SIZE), tip, move |canvas, pen| {
        let s = pen.scale;
        let side = canvas.width().min(canvas.height()) as f32;
        let (k, track) = (theme::sheet().knob * s, theme::sheet().track * s);
        let (c, radius) = (side / 2.0, side / 2.0 - k / 2.0);
        stroke(canvas, arc(c, c, radius, START, START + SWEEP), track, pen.theme.control);
        if at > 0.0 {
            stroke(canvas, arc(c, c, radius, START, START + SWEEP * at), track, pen.theme.soft);
        }
        let a = START + SWEEP * at;
        let (x, y) = (c + radius * a.sin(), c - radius * a.cos());
        crate::ui::knob(canvas, Rect { x: x - k / 2.0, y: y - k / 2.0, w: k, h: k }, pen.theme, s);
    })
    .on_pointer_when(move |p| match p {
        Pointer::Down(x, y) | Pointer::Move(x, y) => Some(change(value_at(x, y, DIAL_SIZE, min, max, step))),
        _ => None,
    })
    .on_keys(move |input| value_after_key(input, value, min, max, step).map(|v| by_key(v)))
    .on_scroll(move |lines| {
        let dir = if lines == 0.0 { 0.0 } else { lines.signum() };
        by_wheel(snap(value - dir * step, min, max, step))
    })
    .role(Role::Slider { value: f64::from(value), min: f64::from(min), max: f64::from(max) })
}

pub fn level_bar<M>(value: f32, min: f32, max: f32, tip: impl Into<String>) -> Node<M> {
    let at = share(value, min, max);
    drawing((theme::SLIDER_MIN_WIDTH, theme::sheet().track * 2.0), tip, move |canvas, pen| {
        let (w, h) = (canvas.width() as f32, canvas.height() as f32);
        let corner = theme::corner_px(theme::Part::Slider, h, pen.scale);
        crate::ui::fill(canvas, Rect { x: 0.0, y: 0.0, w, h }, corner, pen.theme.control);
        if at > 0.0 {
            crate::ui::fill(canvas, Rect { x: 0.0, y: 0.0, w: (w * at).max(h), h }, corner, pen.theme.soft);
        }
    })
    .progress(Some(at))
    .width(Length::Fill)
}

pub fn scale<M: 'static>(value: f32, min: f32, max: f32, step: f32, marks: &[(f32, &str)], tip: impl Into<String>, change: impl Fn(f32) -> M + 'static) -> Node<M> {
    let k = theme::sheet().knob;
    let under = marks.iter().map(|&(at, words)| {
        let part = share(at, min, max);
        placed(note(words.to_string()), move |(iw, _), (w, _)| ((part * (w - k) + k / 2.0 - iw / 2.0).clamp(0.0, (w - iw).max(0.0)), 0.0))
    }).collect();
    column(vec![slider(value, min, max, step, tip, change).width(Length::Fill), layers(under).width(Length::Fill)]).gap(crate::theme::gaps().small)
}

pub fn volume_icon(level: f32, muted: bool) -> &'static str {
    match level {
        _ if muted || level <= 0.0 => "audio-volume-muted-symbolic",
        l if l < 0.34 => "audio-volume-low-symbolic",
        l if l < 0.67 => "audio-volume-medium-symbolic",
        _ => "audio-volume-high-symbolic",
    }
}

pub fn volume<M: 'static>(level: f32, muted: bool, mute: M, change: impl Fn(f32) -> M + 'static) -> Node<M> {
    let tip = if muted { t!("kit-volume-unmute") } else { t!("kit-volume-mute") };
    row(vec![
        button(volume_icon(level, muted), tip).on_press(mute).on(muted),
        slider(if muted { 0.0 } else { level }, 0.0, 1.0, 0.05, t!("kit-volume"), change).width(Length::Fill),
    ]).gap(crate::theme::gaps().medium)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::input::Mods;

    fn key(sym: Keysym) -> Input {
        Input::Key { sym, text: None, mods: Mods::default(), repeat: false }
    }

    #[test]
    fn a_point_round_the_dial_is_its_value_held_to_the_track() {
        let c = DIAL_SIZE / 2.0;
        assert_eq!(value_at(c, 0.0, DIAL_SIZE, 0.0, 100.0, 1.0), 50.0, "straight up is the middle");
        assert_eq!(value_at(DIAL_SIZE, c, DIAL_SIZE, 0.0, 100.0, 1.0), 83.0);
        assert_eq!(value_at(c - 1.0, DIAL_SIZE, DIAL_SIZE, 0.0, 100.0, 1.0), 0.0, "below the gap holds to the near end");
        assert_eq!(value_at(c + 1.0, DIAL_SIZE, DIAL_SIZE, 0.0, 100.0, 1.0), 100.0);
        assert_eq!(snap(47.0, 0.0, 100.0, 5.0), 45.0);
        assert_eq!(snap(130.0, 0.0, 100.0, 5.0), 100.0);
    }

    #[test]
    fn keys_move_a_step_ten_steps_or_to_an_end() {
        let at = |sym| value_after_key(&key(sym), 50.0, 0.0, 100.0, 2.0);
        assert_eq!((at(Keysym::Up), at(Keysym::Left)), (Some(52.0), Some(48.0)));
        assert_eq!((at(Keysym::Page_Up), at(Keysym::Page_Down)), (Some(70.0), Some(30.0)));
        assert_eq!((at(Keysym::Home), at(Keysym::End), at(Keysym::a)), (Some(0.0), Some(100.0), None));
        assert_eq!(value_after_key(&key(Keysym::Up), 100.0, 0.0, 100.0, 2.0), Some(100.0));
    }

    #[test]
    fn the_volume_icon_follows_the_level() {
        assert_eq!(volume_icon(0.8, true), "audio-volume-muted-symbolic");
        assert_eq!(volume_icon(0.0, false), "audio-volume-muted-symbolic");
        assert_eq!(volume_icon(0.2, false), "audio-volume-low-symbolic");
        assert_eq!(volume_icon(0.5, false), "audio-volume-medium-symbolic");
        assert_eq!(volume_icon(1.0, false), "audio-volume-high-symbolic");
    }
}
