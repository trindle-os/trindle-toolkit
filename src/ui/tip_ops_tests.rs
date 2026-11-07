//! A label on pointing, a refusal and the bell are recorded as drawing ops, so the GPU draws them and no window-sized picture is rasterised, hashed or sent for them.

use super::*;
use super::super::paint::{changes, Op, Seen};
use super::super::theme::Theme;
use super::super::widget::{button, column};

const W: u32 = 600;
const H: u32 = 400;

fn ui() -> Ui<u8> {
    let view = column(vec![button("document-open-symbolic", "Save John Smith's work").on_press(1u8).key("save")]).padding(10.0);
    let mut ui = Ui::new(view, Theme::new(true, "#fbc711"));
    ui.lay_at(W, H);
    ui.ready((W, H), 1.0);
    ui.dev.gpu.set(true);
    ui
}

fn recorded(ui: &Ui<u8>) -> Vec<Op<'_>> {
    let mut p = Painter::default();
    ui.record(&mut p, None);
    p.ops
}

fn counts(ops: &[Op]) -> (usize, usize, usize) {
    let whole = ops.iter().filter(|o| matches!(o, Op::Pixels { at, .. } if at.width() >= W && at.height() >= H)).count();
    let shapes = ops.iter().filter(|o| matches!(o, Op::Round { .. })).count();
    let letters = ops.iter().filter(|o| matches!(o, Op::Glyphs { .. })).count();
    (whole, shapes, letters)
}

fn laid_area(ops: &[Op]) -> u64 {
    ops.iter().filter_map(|o| if let Op::Laid { pixmap, .. } = o { Some(pixmap.width() as u64 * pixmap.height() as u64) } else { None }).sum()
}

fn show_tip(ui: &mut Ui<u8>) {
    ui.tip = Some(Tip { id: "save".into(), state: TipState::Shown(Instant::now()) });
}

#[test]
fn a_hovered_tip_records_ops_and_no_pixel_region() {
    let mut ui = ui();
    let (whole, shapes, letters) = counts(&recorded(&ui));
    show_tip(&mut ui);
    let (whole_tip, shapes_tip, letters_tip) = counts(&recorded(&ui));
    assert_eq!(whole_tip, whole, "the tip added a window-sized pixel region");
    assert!(shapes_tip > shapes, "the tip's pill and edge are shapes");
    assert!(letters_tip > letters, "the tip's words are letters");
}

#[test]
fn a_refusal_records_ops_and_no_pixel_region() {
    let mut ui = ui();
    let (whole, shapes, letters) = counts(&recorded(&ui));
    ui.say_refused(Instant::now());
    let (whole_said, shapes_said, letters_said) = counts(&recorded(&ui));
    assert_eq!(whole_said, whole);
    assert!(shapes_said > shapes && letters_said > letters);
}

#[test]
fn the_gpu_path_lays_no_tip_pixels_down() {
    let mut ui = ui();
    show_tip(&mut ui);
    let mut p = Painter::default();
    ui.record(&mut p, None);
    let (ops, _) = changes(p, (W, H), &mut Seen::new(), &mut None);
    let (whole, ..) = counts(&ops);
    assert_eq!(whole, 0, "a window-sized pixel region is left");
    assert!(laid_area(&ops) < (W as u64) * (H as u64) / 4, "{} pixels laid for a tip", laid_area(&ops));
}

#[test]
fn the_tip_is_the_same_on_the_processor_whether_painted_or_recorded() {
    let mut ui = ui();
    show_tip(&mut ui);
    let mut direct = tiny_skia::Pixmap::new(W, H).unwrap();
    ui.paint(&mut direct.as_mut(), 1.0);
    let mut p = Painter::default();
    ui.record(&mut p, None);
    let mut recorded = tiny_skia::Pixmap::new(W, H).unwrap();
    crate::paint::cpu::play(p.ops, &mut recorded.as_mut(), &ui.shadows);
    let off = direct.data().iter().zip(recorded.data()).filter(|(a, b)| a != b).count();
    assert_eq!(off, 0, "{off} bytes differ");
}
