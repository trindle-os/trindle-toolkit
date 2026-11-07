//! The gallery at every common screen scale, whole and fractional, and moved between screens of different scales.

use super::*;
use crate::widget::Rect;

const SCALES: [f32; 5] = [1.0, 1.25, 1.5, 1.75, 2.0];
const SLACK: f32 = 3.0;
const DRIFT: f32 = 0.03;

fn laid(view: Node<()>, scale: f32) -> (Ui<()>, tiny_skia::Pixmap) {
    let mut ui = Ui::new(view, Theme::new(false, "#fbc711"));
    let mut pix = tiny_skia::Pixmap::new((SIZE.0 as f32 * scale).round() as u32, (SIZE.1 as f32 * scale).round() as u32).unwrap();
    ui.paint(&mut pix.as_mut(), scale);
    (ui, pix)
}

fn in_points(ui: &Ui<()>, scale: f32) -> Vec<(String, Rect)> {
    ui.boxes().into_iter().map(|(id, r, _)| (id, Rect { x: r.x / scale, y: r.y / scale, w: r.w / scale, h: r.h / scale })).collect()
}

fn apart(a: &Rect, b: &Rect) -> bool {
    [(a.x, b.x), (a.y, b.y), (a.w, b.w), (a.h, b.h)].iter().any(|(p, q)| (p - q).abs() > SLACK + DRIFT * q.abs())
}

#[test]
fn every_part_keeps_its_place_in_points_at_every_scale() {
    let (one, _) = laid(gallery(0), 1.0);
    let want = in_points(&one, 1.0);
    let mut faults = Vec::new();
    for scale in SCALES {
        let (ui, _) = laid(gallery(0), scale);
        let got = in_points(&ui, scale);
        assert_eq!(got.len(), want.len(), "the same parts at scale {scale}");
        for ((id, a), (_, b)) in got.iter().zip(&want) {
            if apart(a, b) {
                faults.push(format!("{id} at scale {scale}: {a:?} in points, {b:?} at 1"));
            }
        }
    }
    assert!(faults.is_empty(), "{}", faults.iter().take(12).cloned().collect::<Vec<_>>().join("\n"));
}

#[test]
fn nothing_is_squeezed_touching_or_off_the_window_at_any_scale() {
    for scale in SCALES {
        let (mut ui, pix) = laid(gallery(0), scale);
        assert_eq!(ui.squeezed_or_touching(), Vec::<String>::new(), "scale {scale}");
        for (id, r, _) in ui.boxes() {
            assert!(r.x >= 0.0 && r.x + r.w <= pix.width() as f32 + 0.5, "{id} goes off the side at scale {scale}: {r:?}");
        }
    }
}

#[test]
fn a_press_in_points_lands_on_the_same_switch_at_every_scale() {
    let view = || column(vec![row(vec![label("Wifi"), switch(false, "Wifi", |_| ())]).gap(crate::theme::gaps().roomy), space()]).padding(crate::theme::gaps().huge);
    for scale in SCALES {
        let mut ui = Ui::new(view(), Theme::new(false, "#fbc711"));
        let mut pix = tiny_skia::Pixmap::new((400.0 * scale) as u32, (200.0 * scale) as u32).unwrap();
        ui.paint(&mut pix.as_mut(), scale);
        let (_, r, _) = ui.boxes().into_iter().find(|(id, _, _)| ui_kind(&ui, id) == Some("switch")).expect("a switch");
        let (x, y) = ((r.x + r.w / 2.0) / scale, (r.y + r.h / 2.0) / scale);
        let now = Instant::now();
        ui.input(&Input::Press { button: Button::Left, x, y }, now);
        let reply = ui.input(&Input::Release { button: Button::Left, x, y }, now);
        assert_eq!(reply.messages.len(), 1, "the switch was pressed at scale {scale}");
    }
}

fn ui_kind(ui: &Ui<()>, id: &str) -> Option<&'static str> {
    ui.pressables().into_iter().find(|(i, _)| i == id).map(|(_, k)| k)
}

#[test]
fn what_is_drawn_grows_with_the_scale() {
    let inked = |pix: &tiny_skia::Pixmap| {
        let bg = pix.pixel(0, 0).unwrap();
        pix.pixels().iter().filter(|p| **p != bg).count() as f32
    };
    let (_, one) = laid(gallery(0), 1.0);
    let base = inked(&one);
    assert!(base > 0.0);
    for scale in SCALES {
        let (_, pix) = laid(gallery(0), scale);
        let share = inked(&pix) / (base * scale * scale);
        assert!((0.85..=1.15).contains(&share), "at scale {scale} {share} of the ink one would expect");
    }
}

#[test]
fn a_window_moved_to_a_screen_of_another_scale_is_laid_out_afresh() {
    for (from, to) in [(1.0, 1.5), (2.0, 1.25), (1.75, 1.0)] {
        let (mut moved, _) = laid(gallery(0), from);
        let mut pix = tiny_skia::Pixmap::new((SIZE.0 as f32 * to).round() as u32, (SIZE.1 as f32 * to).round() as u32).unwrap();
        moved.paint(&mut pix.as_mut(), to);
        let (fresh, fresh_pix) = laid(gallery(0), to);
        assert_eq!(moved.boxes(), fresh.boxes(), "from {from} to {to}");
        let (_, turning, _) = fresh.boxes().into_iter().find(|(id, _, _)| id.contains("gallery:spinner")).expect("the spinner");
        let w = pix.width() as usize;
        let off = pix.pixels().iter().zip(fresh_pix.pixels()).enumerate().filter(|(_, (a, b))| a != b).map(|(i, _)| ((i % w) as f32, (i / w) as f32))
            .filter(|(x, y)| !(turning.x <= *x && *x <= turning.x + turning.w && turning.y <= *y && *y <= turning.y + turning.h)).count();
        assert_eq!(off, 0, "from {from} to {to}: pixels not drawn as in a window opened there");
    }
}
