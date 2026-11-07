//! Tests of the inspector and the speed overlay.

use super::*;
use super::super::widget::{button, column, entry, spinner};

#[derive(Clone, Debug, PartialEq)]
enum Msg {
    Go,
    Typed(String),
}

fn ui() -> Ui<Msg> {
    let view = column(vec![button("document-open-symbolic", "Open").on_press(Msg::Go).key("open"), entry("", "Name", Msg::Typed).key("name"), spinner()]).padding(10.0);
    let mut ui = Ui::new(view, Theme::new(false, "#fbc711"));
    ui.lay_at(400, 300);
    ui
}

fn chord(sym: Keysym) -> Input {
    Input::Key { sym, text: None, mods: Mods { ctrl: true, shift: true, ..Mods::default() }, repeat: false }
}

fn centre(ui: &Ui<Msg>, key: &str) -> (f32, f32) {
    let r = ui.boxes().into_iter().find(|b| b.0 == key).unwrap().1;
    (r.x + r.w / 2.0, r.y + r.h / 2.0)
}

fn paint(ui: &mut Ui<Msg>) -> tiny_skia::Pixmap {
    let mut pix = tiny_skia::Pixmap::new(400, 300).unwrap();
    ui.paint(&mut pix.as_mut(), 1.0);
    pix
}

#[test]
fn the_inspector_key_names_the_widget_under_the_pointer() {
    let mut ui = ui();
    let now = Instant::now();
    let (x, y) = centre(&ui, "open");
    ui.input(&Input::PointerMove { x, y }, now);
    assert_eq!(ui.inspected(), None, "off until the key");
    let reply = ui.input(&chord(Keysym::F12), now);
    assert!(reply.redraw && reply.messages.is_empty());
    let (r, lines) = ui.inspected().expect("the button is inspected");
    assert_eq!(r, ui.boxes().into_iter().find(|b| b.0 == "open").unwrap().1);
    assert_eq!(lines[0], "button open");
    assert_eq!(lines[1], format!("{} by {} pixels at {}, {}", r.w, r.h, r.x, r.y));
    assert_eq!(lines[2], "State: pointer on it, takes the keyboard");
    assert_eq!(lines[3], "Screen reader: Button \"Open\"");
    assert!(lines[4].starts_with("--trindle-icon-button-corner: "), "{lines:?}");
    ui.input(&chord(Keysym::F12), now);
    assert_eq!(ui.inspected(), None);
}

#[test]
fn the_inspector_follows_the_pointer_and_says_the_keyboard() {
    let mut ui = ui();
    let now = Instant::now();
    ui.inspect(true);
    let (x, y) = centre(&ui, "name");
    assert!(ui.input(&Input::PointerMove { x, y }, now).redraw, "each move paints the inspector afresh");
    ui.keyboard_to("name");
    let (_, lines) = ui.inspected().unwrap();
    assert_eq!(lines[0], "entry name");
    assert!(lines[2].contains("keyboard on it"), "{lines:?}");
    assert!(lines[3].starts_with("Screen reader: TextInput"), "{lines:?}");
    assert!(lines.iter().any(|l| l.starts_with("--trindle-box-corner: ")), "{lines:?}");
}

#[test]
fn the_inspector_draws_an_edge_round_the_widget_and_its_words_under_it() {
    let mut ui = ui();
    let (x, y) = centre(&ui, "open");
    ui.input(&Input::PointerMove { x, y }, Instant::now());
    let plain = paint(&mut ui);
    ui.inspect(true);
    let shown = paint(&mut ui);
    let r = ui.inspected().unwrap().0;
    let at = |p: &tiny_skia::Pixmap, x: f32, y: f32| p.pixel(x as u32, y as u32).unwrap();
    let accent = ui.theme.accent.to_skia().to_color_u8();
    let edge = at(&shown, r.x, r.y + r.h / 2.0);
    assert_eq!((edge.red(), edge.green(), edge.blue()), (accent.red(), accent.green(), accent.blue()));
    let below = (r.y + r.h + 8.0) as u32..(r.y + r.h + 60.0) as u32;
    assert!(below.into_iter().any(|y| at(&plain, r.x + r.w / 2.0, y as f32) != at(&shown, r.x + r.w / 2.0, y as f32)));
}

#[test]
fn the_speed_overlay_says_the_paint_time_the_memory_and_what_was_painted_again() {
    let mut ui = ui();
    assert_eq!(ui.speed(), None);
    assert!(ui.input(&chord(Keysym::F11), Instant::now()).redraw);
    let plain = {
        ui.show_speed(false);
        paint(&mut ui)
    };
    ui.show_speed(true);
    let shown = paint(&mut ui);
    let lines = ui.speed().unwrap();
    assert!(lines[0].starts_with("Paint ") && lines[0].ends_with(" ms"), "{lines:?}");
    let mb: f32 = lines[1].trim_start_matches("Memory ").trim_end_matches(" MB").parse().unwrap();
    assert!(mb > 1.0, "{lines:?}");
    assert_eq!(lines[2], "Painted again: 100 percent of the window");
    let differs = |x: u32, y: u32| plain.pixel(x, y) != shown.pixel(x, y);
    assert!((300..400).any(|x| (0..60).any(|y| differs(x, y))));
    assert!(!(0..100).any(|x| (200..300).any(|y| differs(x, y))));
    let mut held = shown.clone();
    let parts = ui.paint_live(&mut held.as_mut(), 1.0).expect("painted in parts");
    let share: f32 = ui.speed().unwrap()[2].trim_start_matches("Painted again: ").trim_end_matches(" percent of the window").parse().unwrap();
    assert!(share > 0.0 && share < 50.0, "{share}");
    assert!(parts.iter().any(|p| p.x + p.w >= 399.0 && p.y <= 1.0), "{parts:?}");
}
