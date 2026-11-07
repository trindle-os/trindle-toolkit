//! A list's rows are recorded as drawing ops, not drawn on pixels, so the GPU draws them and the processor never rasterises them for it.

use super::*;
use super::super::paint::{changes, Op, Seen};
use super::super::theme::Theme;
use super::super::widget::{item, list, scroll, Length};

const W: u32 = 600;
const H: u32 = 400;

fn rows(n: usize) -> Vec<crate::widget::Item> {
    (0..n).map(|i| item(format!("John Smith {i}")).icon("folder-symbolic").folder().detail(format!("{i} KB"))).collect()
}

fn ui(n: usize) -> Ui<usize> {
    let view = scroll("area", list(rows(n), Some(3), |i| i).key("rows")).height(Length::Fill);
    let mut ui = Ui::new(view, Theme::new(true, "#fbc711"));
    ui.lay_at(W, H);
    ui.ready((W, H), 1.0);
    ui
}

fn recorded(ui: &Ui<usize>) -> Vec<Op<'_>> {
    let mut p = Painter::default();
    ui.record(&mut p, None);
    p.ops
}

fn tallest(ops: &[Op]) -> u32 {
    ops.iter().filter_map(|o| match o {
        Op::Pixels { at, .. } => Some(at.height()),
        Op::Laid { pixmap, .. } => Some(pixmap.height()),
        _ => None,
    }).max().unwrap_or(0)
}

#[test]
fn a_list_frame_records_ops_and_no_pixel_region_of_rows() {
    let mut ui = ui(10_000);
    let point = ui.spot("rows").unwrap().rect;
    ui.input(&Input::PointerIn { x: point.x + 20.0, y: point.y + 60.0 }, Instant::now());
    ui.tip = None;
    let row = ui.px(theme::sheet().row) as u32;
    let ops = recorded(&ui);
    assert!(ops.iter().any(|o| matches!(o, Op::Glyphs { .. })), "words are recorded as letters");
    assert!(ops.iter().any(|o| matches!(o, Op::Image { .. })), "icons are recorded as pictures");
    assert!(ops.iter().any(|o| matches!(o, Op::Round { .. })), "bands and the chosen row are recorded as shapes");
    assert!(tallest(&ops) < 2 * row, "a pixel region {} tall holds more than one row ({row})", tallest(&ops));
}

#[test]
fn only_the_rows_in_sight_are_recorded() {
    let few = recorded(&ui(10_000)).len();
    let many = recorded(&ui(100)).len();
    assert!(few < 1000, "{few} ops for a list of 10,000");
    assert!((few as i64 - many as i64).abs() < 50, "{few} against {many}");
}

#[test]
fn the_gpu_path_lays_no_list_pixels_down() {
    let ui = ui(10_000);
    let row = ui.px(theme::sheet().row) as u32;
    let mut p = Painter::default();
    ui.record(&mut p, None);
    let (ops, _) = changes(p, (W, H), &mut Seen::new(), &mut None);
    let area: u64 = ops.iter().filter_map(|o| if let Op::Laid { pixmap, .. } = o { Some(pixmap.width() as u64 * pixmap.height() as u64) } else { None }).sum();
    assert!(tallest(&ops) < 2 * row);
    assert!(area < (W as u64) * (row as u64) * 2, "{area} pixels laid, the list is {} ", W * H);
}
