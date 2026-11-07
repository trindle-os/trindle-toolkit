//! An open dropdown list, an open copy menu and the cross-fade from a previous view are recorded as drawing ops, so the GPU draws them and no window-sized picture is rasterised, hashed or sent for them.

use super::*;
use super::super::paint::{changes, Op, Seen};
use super::super::theme::Theme;
use super::super::widget::{self as ww};

const W: u32 = 600;
const H: u32 = 400;

#[derive(Clone)]
enum Msg {
    Pick,
}

fn ui() -> Ui<Msg> {
    let names: Vec<String> = (0..30).map(|i| format!("Choice {i}")).collect();
    let view = ww::column(vec![
        ww::dropdown(names, Some(2), "Time zone", |_| Msg::Pick).key("zone"),
        ww::label("John Smith's work").key("words"),
    ]).padding(10.0);
    let mut ui = Ui::new(view, Theme::new(true, "#fbc711"));
    ui.lay_at(W, H);
    ui.ready((W, H), 1.0);
    ui.dev.gpu.set(true);
    ui
}

fn recorded(ui: &Ui<Msg>) -> Vec<Op<'_>> {
    let mut p = Painter::default();
    ui.record(&mut p, None);
    p.ops
}

fn whole(ops: &[Op]) -> usize {
    ops.iter().filter(|o| matches!(o, Op::Pixels { at, .. } if at.width() >= W && at.height() >= H)).count()
}

fn shapes_and_letters(ops: &[Op]) -> (usize, usize) {
    (ops.iter().filter(|o| matches!(o, Op::Round { .. } | Op::Path { .. } | Op::Shadow { .. })).count(),
     ops.iter().filter(|o| matches!(o, Op::Glyphs { .. })).count())
}

fn laid_area(ops: &[Op]) -> u64 {
    ops.iter().filter_map(|o| if let Op::Laid { pixmap, .. } = o { Some(pixmap.width() as u64 * pixmap.height() as u64) } else { None }).sum()
}

fn open_drop(ui: &mut Ui<Msg>) {
    ui.open_drop("zone");
    assert!(ui.drop.is_some());
}

fn open_menu(ui: &mut Ui<Msg>) {
    let now = Instant::now();
    let r = ui.spot("words").unwrap().rect;
    let (x, y) = (r.x + 4.0, r.y + r.h / 2.0);
    ui.input(&Input::PointerIn { x, y }, now);
    ui.input(&Input::Press { button: Button::Right, x, y }, now);
    assert!(ui.copy_menu.is_some());
}

fn fade(ui: &mut Ui<Msg>) {
    ui.blend_to(Some((1, 0.5)));
    ui.ready((W, H), 1.0);
    ui.blend_to(Some((2, 0.5)));
    assert!(ui.before.is_some());
}

fn lays_nothing_big(ui: &Ui<Msg>) {
    let mut p = Painter::default();
    ui.record(&mut p, None);
    let (ops, _) = changes(p, (W, H), &mut Seen::new(), &mut None);
    assert_eq!(whole(&ops), 0, "a window-sized pixel region is left");
    assert!(laid_area(&ops) < (W as u64) * (H as u64) / 4, "{} pixels laid", laid_area(&ops));
}

fn same_on_the_processor(ui: &mut Ui<Msg>) {
    let mut direct = tiny_skia::Pixmap::new(W, H).unwrap();
    ui.paint(&mut direct.as_mut(), 1.0);
    ui.dev.gpu.set(true);
    let mut p = Painter::default();
    ui.record(&mut p, None);
    let mut played = tiny_skia::Pixmap::new(W, H).unwrap();
    crate::paint::cpu::play(p.ops, &mut played.as_mut(), &ui.shadows);
    let off = direct.data().iter().zip(played.data()).filter(|(a, b)| a != b).count();
    assert_eq!(off, 0, "{off} bytes differ");
}

#[test]
fn an_open_dropdown_records_ops_and_no_pixel_region() {
    let mut ui = ui();
    let (shapes, letters) = shapes_and_letters(&recorded(&ui));
    assert_eq!(whole(&recorded(&ui)), 0);
    open_drop(&mut ui);
    let open = recorded(&ui);
    let (s, l) = shapes_and_letters(&open);
    assert_eq!(whole(&open), 0, "the list added a window-sized pixel region");
    assert!(s > shapes && l > letters, "the list's box, rows and words are drawn natively");
}

#[test]
fn an_open_copy_menu_records_ops_and_no_pixel_region() {
    let mut ui = ui();
    let (shapes, letters) = shapes_and_letters(&recorded(&ui));
    open_menu(&mut ui);
    let open = recorded(&ui);
    let (s, l) = shapes_and_letters(&open);
    assert_eq!(whole(&open), 0, "the menu added a window-sized pixel region");
    assert!(s > shapes && l > letters, "the menu's shadow, box, rows and words are drawn natively");
}

#[test]
fn the_cross_fade_records_no_pixel_region() {
    let mut ui = ui();
    fade(&mut ui);
    let ops = recorded(&ui);
    assert_eq!(whole(&ops), 0, "the earlier view is copied as pixels");
    assert!(ops.iter().any(|o| matches!(o, Op::Image { .. })), "the earlier view is a kept picture");
}

#[test]
fn the_gpu_path_lays_no_list_menu_or_fade_pixels_down() {
    let mut ui = ui();
    open_drop(&mut ui);
    lays_nothing_big(&ui);
    ui.close_drop();
    open_menu(&mut ui);
    lays_nothing_big(&ui);
    ui.close_menu();
    fade(&mut ui);
    lays_nothing_big(&ui);
}

#[test]
fn the_list_and_the_menu_are_the_same_on_the_processor_whether_painted_or_recorded() {
    let mut ui = ui();
    open_drop(&mut ui);
    same_on_the_processor(&mut ui);
    ui.close_drop();
    open_menu(&mut ui);
    same_on_the_processor(&mut ui);
}
