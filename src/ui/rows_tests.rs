//! Rows of a long list scrolled out of sight are not painted; the ones in sight, and half in sight, are.

use super::*;
use super::super::widget::{column, label, scroll};

const SIZE: (u32, u32) = (300, 150);

fn list(n: usize, by: f32) -> Ui<()> {
    let rows = (0..n).map(|i| label(format!("Row number {i}")).key(format!("r{i}"))).collect();
    let mut ui = Ui::new(scroll("list", column(rows)), Theme::new(false, "#fbc711"));
    ui.scrolls.insert("list".into(), by);
    ui.lay_at(SIZE.0, SIZE.1);
    ui
}

fn ops(ui: &Ui<()>) -> usize {
    let mut p = Painter::default();
    ui.paint_range(&mut p, SIZE, 0, ui.spots.len(), 0.0, 0.0);
    p.ops.len()
}

fn pixels(ui: &Ui<()>) -> Vec<u8> {
    let mut canvas = Pixmap::new(SIZE.0, SIZE.1).unwrap();
    ui.paint_widgets(&mut canvas.as_mut());
    canvas.data().to_vec()
}

#[test]
fn a_long_list_paints_no_more_than_a_short_one_that_shows_the_same_rows() {
    let (short, long) = (list(12, 4.0), list(2000, 4.0));
    assert!(short.spot("r11").is_some() && long.spot("r1999").is_some());
    assert_eq!(ops(&short), ops(&long), "rows out of sight were painted");
    assert!(ops(&long) > 0);
}

#[test]
fn rows_cut_by_an_edge_are_still_painted_and_the_pixels_do_not_depend_on_the_rows_waiting() {
    let (short, long) = (list(12, 4.0), list(2000, 4.0));
    crate::text::paint_until_settled(|| {
        pixels(&short);
        pixels(&long);
    });
    let (a, b) = (pixels(&short), pixels(&long));
    let bar = (SIZE.0 as usize - 24) * 4;
    let line = SIZE.0 as usize * 4;
    for y in 0..SIZE.1 as usize {
        assert_eq!(a[y * line..y * line + bar], b[y * line..y * line + bar], "pixels differ on line {y}");
    }
    let bg = &b[0..4];
    let inked = |from: usize, to: usize| (from..to).any(|y| b[y * line..y * line + bar].chunks(4).any(|px| px != bg));
    assert!(inked(0, 6), "the row cut by the top edge is not painted");
    assert!(inked(SIZE.1 as usize - 6, SIZE.1 as usize), "the row cut by the bottom edge is not painted");
}

fn many_rows(n: usize, scrolled: bool) -> Ui<()> {
    let rows = (0..n).map(|i| super::super::widget::item(format!("Row number {i}")).detail(format!("row{i}@example.com"))).collect();
    let all = super::super::widget::list(rows, None, |_| ()).called("rows");
    let mut ui = Ui::new(if scrolled { scroll("area", all) } else { column(vec![all]) }, Theme::new(false, "#fbc711"));
    ui.lay_at(SIZE.0, SIZE.1);
    ui.settle();
    ui
}

static FONTS_STILL: std::sync::Mutex<()> = std::sync::Mutex::new(());

fn paint_once(ui: &mut Ui<()>) {
    let mut canvas = Pixmap::new(SIZE.0, SIZE.1).unwrap();
    ui.paint(&mut canvas.as_mut(), 1.0);
}

#[test]
fn a_long_list_in_no_scrolling_area_lays_out_only_the_words_on_the_canvas() {
    let mut ui = many_rows(5000, false);
    paint_once(&mut ui);
    assert!(ui.texts.laid() < 100, "{} pieces of words were laid out", ui.texts.laid());
}

#[test]
fn words_painted_last_frame_are_not_laid_out_again_when_the_window_is_laid_out_twice_between_frames() {
    let _fonts = FONTS_STILL.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
    let mut ui = many_rows(5000, true);
    crate::text::paint_until_settled(|| paint_once(&mut ui));
    let before = ui.texts.laid();
    for by in [10.0, 20.0] {
        ui.laid = None;
        ui.scrolls.insert("area".into(), by);
        ui.settle();
    }
    paint_once(&mut ui);
    assert!(ui.texts.laid() - before <= 2, "{} pieces of words were laid out again", ui.texts.laid() - before);
}

#[test]
fn a_window_is_laid_out_again_when_the_fonts_arrive() {
    let _fonts = FONTS_STILL.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
    let mut ui = many_rows(20, false);
    crate::text::paint_until_settled(|| paint_once(&mut ui));
    ui.spots.clear();
    paint_once(&mut ui);
    assert!(ui.spots.is_empty(), "laid out again with nothing changed");
    crate::sharp::arrived();
    paint_once(&mut ui);
    assert!(!ui.spots.is_empty(), "the layout is kept from before the fonts came");
}
