//! The bottom notice over a window: the page never moves and the pointer reaches what is under the clear band.

use super::*;
use super::super::notices::Notices;
use super::super::theme::Theme;
use super::super::widget as ww;

#[derive(Clone, Debug, PartialEq)]
enum Msg {
    Pressed,
}

fn ui_with(notices: &Notices) -> Ui<Msg> {
    let page = ww::card(ww::label("Under")).on_press(Msg::Pressed).key("under").height(ww::Length::Fill);
    let mut ui = Ui::new(notices.over(page), Theme::new(true, "#fbc711"));
    ui.size = (600, 400);
    ui.settle();
    ui
}

#[test]
fn a_notice_shows_over_the_foot_and_the_page_does_not_move() {
    let now = Instant::now();
    let mut notices = Notices::new();
    let before = ui_with(&notices).spot("under").unwrap().rect;
    notices.push("Copied to the clipboard", now);
    let mut ui = ui_with(&notices);
    assert_eq!(ui.spot("under").unwrap().rect, before, "the page stays where it was");
    let band = ui.spot("notice").unwrap().rect;
    assert_eq!((band.w, band.y + band.h), (600.0, 400.0 - 8.0));
    let mut a = tiny_skia::Pixmap::new(600, 400).unwrap();
    ui.paint(&mut a.as_mut(), 1.0);
    let mut b = tiny_skia::Pixmap::new(600, 400).unwrap();
    ui_with(&Notices::new()).paint(&mut b.as_mut(), 1.0);
    assert_ne!(a.pixel(300, 400 - 48 - 17), b.pixel(300, 400 - 48 - 17), "the pill is painted at its place");
    let told = ui.access("Test");
    assert!(told.nodes.iter().any(|(_, n)| n.live() == Some(accesskit::Live::Polite)), "the reader is told it");
}

#[test]
fn a_press_through_the_clear_band_reaches_the_page() {
    let mut notices = Notices::new();
    notices.push("Saved", Instant::now());
    let mut ui = ui_with(&notices);
    let r = ui.spot("under").unwrap().rect;
    let band = ui.spot("notice").unwrap().rect;
    assert!(r.y + r.h > band.y, "the button lies under the band");
    let (x, y) = (r.x + 8.0, band.y + band.h / 2.0);
    let now = Instant::now();
    ui.input(&Input::Press { button: Button::Left, x, y }, now);
    assert_eq!(ui.input(&Input::Release { button: Button::Left, x, y }, now).messages, vec![Msg::Pressed]);
}
