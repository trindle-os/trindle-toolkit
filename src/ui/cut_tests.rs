//! Words cut short with dots show whole on pointing, and only when cut.

use super::*;
use super::super::theme::Theme;
use super::super::widget::{self as ww, Length};

const SPEAKERS: &str = "Built-in Audio Analogue Stereo";
const HEADPHONES: &str = "John Smith's Headphones Hands-Free";
const LONG_ROW: &str = "A folder of holiday pictures from the summer of last year";
const LONG_UNDER: &str = "Shared with John Smith and three other people since March";

fn page() -> Node<()> {
    ww::column(vec![
        ww::choice(vec![SPEAKERS.into(), HEADPHONES.into()], 0, "Where sound plays", |_| ()).key("devices").width(Length::Fixed(220.0)),
        ww::choice(vec!["One".into(), "Two".into()], 0, "Which", |_| ()).key("short"),
        ww::list(vec![ww::item(LONG_ROW), ww::item("Short"), ww::item(LONG_ROW).under(LONG_UNDER)], None, |_| ()).key("rows").width(Length::Fixed(160.0)),
        ww::dropdown(vec![SPEAKERS.into(), "Short".into()], Some(0), "Device", |_| ()).key("closed").width(Length::Fixed(140.0)),
        ww::dropdown(vec!["Short".into(), SPEAKERS.into()], Some(0), "Device", |_| ()).key("fits").width(Length::Fixed(140.0)),
        ww::word_button("A word button's long words").on_press(()).key("word").width(Length::Fixed(70.0)),
        ww::main_action("document-open-symbolic", "Save this document and every other document that is open as well").on_press(()).key("action")
            .width(Length::Fixed(120.0)),
        ww::main_action("document-open-symbolic", "Save").on_press(()).key("saves"),
        ww::label(LONG_ROW).cut().key("words").width(Length::Fixed(120.0)),
        ww::button("document-open-symbolic", "Open").on_press(()).key("round"),
    ]).padding(10.0).gap(10.0)
}

fn painted() -> Ui<()> {
    let mut ui = Ui::new(page(), Theme::new(false, "#fbc711"));
    let mut pix = tiny_skia::Pixmap::new(600, 700).unwrap();
    ui.paint(&mut pix.as_mut(), 1.0);
    ui
}

fn pointed(ui: &mut Ui<()>, key: &str, share: (f32, f32)) -> Option<String> {
    let r = ui.spot(key).unwrap_or_else(|| panic!("{key}")).rect;
    ui.pointer = Some((r.x + r.w * share.0, r.y + r.h * share.1));
    ui.label_of(key)
}

#[test]
fn words_cut_short_show_whole_on_pointing_and_only_then() {
    let mut ui = painted();
    assert_eq!(pointed(&mut ui, "devices", (0.2, 0.5)).as_deref(), Some("Built-in Audio Analogue Stereo - Where sound plays"));
    assert_eq!(pointed(&mut ui, "devices", (0.8, 0.5)).as_deref(), Some("John Smith's Headphones Hands-Free - Where sound plays"));
    assert_eq!(pointed(&mut ui, "short", (0.2, 0.5)).as_deref(), Some("Which"));
    assert_eq!(pointed(&mut ui, "short", (0.8, 0.5)).as_deref(), Some("Which"));
    assert_eq!(pointed(&mut ui, "rows", (0.5, 0.5 / 3.0)).as_deref(), Some(LONG_ROW));
    assert_eq!(pointed(&mut ui, "rows", (0.5, 1.5 / 3.0)), None);
    assert_eq!(pointed(&mut ui, "rows", (0.5, 2.5 / 3.0)), Some(format!("{LONG_ROW}\n{LONG_UNDER}")));
    assert_eq!(pointed(&mut ui, "closed", (0.5, 0.5)).as_deref(), Some("Built-in Audio Analogue Stereo - Device"));
    assert_eq!(pointed(&mut ui, "fits", (0.5, 0.5)).as_deref(), Some("Device"));
    assert_eq!(pointed(&mut ui, "word", (0.5, 0.5)).as_deref(), Some("A word button's long words"));
    assert_eq!(pointed(&mut ui, "action", (0.5, 0.5)).as_deref(), Some("Save this document and every other document that is open as well"));
    assert_eq!(pointed(&mut ui, "saves", (0.5, 0.5)), None);
    assert_eq!(pointed(&mut ui, "words", (0.5, 0.5)).as_deref(), Some(LONG_ROW));
    assert_eq!(pointed(&mut ui, "round", (0.5, 0.5)).as_deref(), Some("Open"));
}

#[test]
fn the_label_is_for_the_part_under_the_pointer() {
    let mut ui = painted();
    let rows = ui.spot("rows").unwrap().rect;
    let now = Instant::now();
    let mut reply = Reply::new();
    ui.pointer_to(rows.x + 20.0, rows.y + rows.h * 2.5 / 3.0, now, &mut reply);
    assert!(matches!(ui.tip.as_ref().map(|t| t.state), Some(TipState::Waiting(_))));
    ui.tick(now + tip_rest() + Duration::from_millis(10));
    assert!(matches!(ui.tip.as_ref().map(|t| t.state), Some(TipState::Shown(_))));
    let mut pix = tiny_skia::Pixmap::new(600, 700).unwrap();
    let drawn = ui.tip_drawn(&mut pix.as_mut()).expect("the label is drawn");
    let third = rows.y + rows.h * 2.0 / 3.0;
    assert!(drawn.y >= third + rows.h / 3.0 - 1.0 && drawn.y <= rows.y + rows.h + 24.0, "under the third row: {drawn:?} for rows at {rows:?}");
    ui.pointer_to(rows.x + 20.0, rows.y + rows.h * 0.5 / 3.0, now + Duration::from_secs(1), &mut reply);
    assert!(matches!(ui.tip.as_ref().map(|t| t.state), Some(TipState::Waiting(_))), "a new row starts the label again");
    ui.tick(now + Duration::from_secs(1) + tip_rest() + Duration::from_millis(10));
    let drawn = ui.tip_drawn(&mut pix.as_mut()).expect("the label is drawn");
    assert!(drawn.y < third, "under the first row: {drawn:?}");
    let devices = ui.spot("devices").unwrap().rect;
    ui.pointer_to(devices.x + devices.w * 0.8, devices.y + devices.h / 2.0, now + Duration::from_secs(2), &mut reply);
    ui.tick(now + Duration::from_secs(2) + tip_rest() + Duration::from_millis(10));
    let drawn = ui.tip_drawn(&mut pix.as_mut()).expect("the label is drawn");
    let (middle, half) = (drawn.x + drawn.w / 2.0, devices.x + devices.w / 2.0);
    assert!(middle > half || drawn.x + drawn.w >= 599.0, "under the second pill: {drawn:?} for {devices:?}");
}

#[test]
fn the_screen_reader_always_hears_the_whole_words() {
    let mut ui = painted();
    let names: Vec<String> = ui.access("Test").nodes.iter().filter_map(|(_, part)| part.label().map(str::to_string)).collect();
    for whole in [SPEAKERS, HEADPHONES, LONG_ROW, "Save this document and every other document that is open as well"] {
        assert!(names.iter().any(|n| n.contains(whole)), "{whole:?} is not among {names:?}");
    }
}
