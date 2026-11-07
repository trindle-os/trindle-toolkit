//! Scale from 100 to 400 percent in quarter steps with no blur: every widget on whole pixels and words on whole pixels; and the same window given twice paints the same picture.

use super::*;
use super::super::text::Style;
use super::super::widget::{button, card, choice, column, entry, label, main_word, row, slider, switch};

#[derive(Clone, Debug, PartialEq)]
enum Msg {
    Any,
}

fn view() -> Node<Msg> {
    column(vec![
        label("Scaling"),
        card(column(vec![entry("John Smith", "Name", |_| Msg::Any), switch(true, "Wi-Fi", |_| Msg::Any), slider(0.3, 0.0, 1.0, 0.1, "Volume", |_| Msg::Any)])),
        choice(vec!["One".into(), "Two".into(), "Three".into()], 1, "Count", |_| Msg::Any),
        row(vec![button("help-about-symbolic", "Help").on_press(Msg::Any), main_word("Save").on_press(Msg::Any)]),
    ]).padding(16.0).gap(12.0)
}

fn scales() -> impl Iterator<Item = f32> {
    (0..=12).map(|i| 1.0 + i as f32 * 0.25)
}

fn painted(scale: f32) -> (Ui<Msg>, tiny_skia::Pixmap) {
    let mut ui = Ui::new(view(), Theme::new(false, "#fbc711"));
    let (w, h) = ((360.0 * scale).round() as u32, (400.0 * scale).round() as u32);
    let mut pix = tiny_skia::Pixmap::new(w, h).unwrap();
    ui.paint(&mut pix.as_mut(), scale);
    (ui, pix)
}

#[test]
fn every_widget_sits_on_whole_pixels_at_every_quarter_scale() {
    for scale in scales() {
        let (ui, _) = painted(scale);
        let off: Vec<String> = ui.boxes().into_iter().filter(|(_, r, _)| [r.x, r.y, r.w, r.h].iter().any(|v| v.fract() != 0.0))
            .map(|(id, r, _)| format!("{id} at {r:?}")).collect();
        assert!(off.is_empty(), "at {scale}: {off:?}");
    }
}

#[test]
fn words_land_on_whole_pixels_wherever_they_are_asked_for_at_every_quarter_scale() {
    let fonts = Fonts::new();
    for scale in scales() {
        let layout = fonts.layout("Sharp words, 0123", &Style::new(14.0 * scale), None);
        let at = |x: f32, y: f32| {
            let mut pix = tiny_skia::Pixmap::new((200.0 * scale) as u32, (40.0 * scale) as u32).unwrap();
            fonts.draw(&mut pix.as_mut(), &layout, x, y, Colour::rgb(1.0, 1.0, 1.0));
            pix
        };
        let whole = at(8.0, 6.0);
        assert!(whole.pixels().iter().any(|p| p.alpha() > 0), "at {scale} nothing was drawn");
        let (w, h) = (whole.width() as usize, whole.height() as usize);
        let (along, down) = (at(9.0, 6.0), at(8.0, 7.0));
        let px = |p: &tiny_skia::Pixmap, x: usize, y: usize| p.pixels()[y * w + x];
        assert!((0..h).all(|y| (0..w - 1).all(|x| px(&whole, x, y) == px(&along, x + 1, y))), "at {scale}, words a pixel along were drawn differently");
        assert!((0..h - 1).all(|y| (0..w).all(|x| px(&whole, x, y) == px(&down, x, y + 1))), "at {scale}, words a pixel down were drawn differently");
        let mut asked: Vec<u8> = at(8.3, 6.4).pixels().iter().map(|p| p.alpha()).filter(|a| *a > 0).collect();
        let mut kept: Vec<u8> = whole.pixels().iter().map(|p| p.alpha()).filter(|a| *a > 0).collect();
        asked.sort_unstable();
        kept.sort_unstable();
        assert_eq!(asked, kept, "at {scale}, words asked for between pixels were drawn blurred");
    }
}

#[test]
fn the_same_window_paints_the_same_picture_every_time() {
    for scale in [1.0, 1.25, 2.0, 3.5] {
        let (a, first) = painted(scale);
        let (b, second) = painted(scale);
        assert_eq!(a.boxes(), b.boxes(), "at {scale} the layout differs");
        assert!(first.data() == second.data(), "at {scale} the picture differs");
    }
}
