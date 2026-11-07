//! The controls, actions and data helpers driven in a real window: keys, pointer, paint and what the screen reader hears.

use super::*;
use super::super::action::{Action, Actions};
use super::super::palette::{self, Palette};
use super::super::validate::{self, CheckedBox, Mask};
use super::super::{dial, info, rows, spin};
use super::super::theme::Theme;
use super::super::widget as ww;

#[derive(Clone, Debug, PartialEq)]
enum Msg {
    Palette(palette::Edit),
    Value(f32),
    Check(validate::Edit),
    On(bool),
    Mute,
}

fn ui_of(tree: Node<Msg>) -> Ui<Msg> {
    let mut ui = Ui::new(tree, Theme::new(true, "#fbc711"));
    ui.size = (400, 300);
    ui.settle();
    ui
}

fn shot(ui: &mut Ui<Msg>) -> tiny_skia::Pixmap {
    let mut pix = tiny_skia::Pixmap::new(400, 300).unwrap();
    ui.paint(&mut pix.as_mut(), 1.0);
    pix
}

fn key(sym: Keysym) -> Input {
    Input::Key { sym, text: None, mods: Mods::default(), repeat: false }
}

fn typed(t: &str) -> Input {
    Input::Key { sym: Keysym::NoSymbol, text: Some(t.into()), mods: Mods::default(), repeat: false }
}

fn actions() -> Actions<&'static str> {
    let mut all = Actions::new();
    all.add(Action::new("save", "Save").icon("document-save-symbolic").keys("Ctrl+S")).unwrap();
    all.add(Action::new("close", "Close").keys("Ctrl+W")).unwrap();
    all.add(Action::new("reopen", "Reopen closed tab")).unwrap();
    all
}

#[test]
fn the_palette_runs_what_is_typed_with_enter_and_lists_the_rest() {
    let all = actions();
    let mut pal = Palette::default();
    let mut ui = ui_of(pal.view(&all, Msg::Palette));
    let mut ran = Vec::new();
    for input in [typed("c"), typed("l"), typed("o"), key(Keysym::Return)] {
        for m in ui.input(&input, Instant::now()).messages {
            if let Msg::Palette(e) = m {
                ran.extend(pal.update(e, &all));
            }
        }
        ui.set_view(pal.view(&all, Msg::Palette));
    }
    assert_eq!(ran, vec!["close"]);
    assert_eq!(pal, Palette::default(), "running an action empties the palette");
    pal.query = "zzz".into();
    let mut ui = ui_of(pal.view(&all, Msg::Palette));
    assert!(ui.tree.said().iter().any(|w| w == "No action matches."));
    let told = ui.access("Test");
    assert!(told.nodes.iter().any(|(_, n)| n.role() == accesskit::Role::TextInput && n.label() == Some("Type to find an action")));
}

#[test]
fn the_palette_runs_a_row_opened_in_its_list() {
    let all = actions();
    let mut pal = Palette { query: "o".into(), chosen: 0 };
    assert_eq!(pal.update(palette::Edit::Open(1), &all), Some("reopen"));
}

fn role_of(told: &accesskit::TreeUpdate, role: accesskit::Role) -> Option<&accesskit::Node> {
    told.nodes.iter().find(|(_, n)| n.role() == role).map(|(_, n)| n)
}

#[test]
fn a_dial_turns_by_pointer_keys_and_wheel_and_is_heard_as_a_slider() {
    let tree = |v: f32| ww::column(vec![dial::dial(v, 0.0, 100.0, 5.0, "Treble", Msg::Value).key("dial")]).padding(10.0);
    let mut ui = ui_of(tree(50.0));
    let r = ui.spot("dial").unwrap().rect;
    assert_eq!((r.w, r.h), (dial::DIAL_SIZE, dial::DIAL_SIZE));
    let now = Instant::now();
    ui.input(&Input::PointerMove { x: r.x + r.w - 1.0, y: r.y + r.h / 2.0 }, now);
    let sent = ui.input(&Input::Press { button: Button::Left, x: r.x + r.w - 1.0, y: r.y + r.h / 2.0 }, now).messages;
    assert_eq!(sent, vec![Msg::Value(85.0)]);
    ui.input(&Input::Release { button: Button::Left, x: r.x + r.w - 1.0, y: r.y + r.h / 2.0 }, now);
    let mut ui = ui_of(tree(50.0));
    ui.input(&key(Keysym::Tab), now);
    assert_eq!(ui.input(&key(Keysym::Up), now).messages, vec![Msg::Value(55.0)]);
    assert_eq!(ui.input(&key(Keysym::End), now).messages, vec![Msg::Value(100.0)]);
    let told = ui.access("Test");
    let part = role_of(&told, accesskit::Role::Slider).unwrap();
    assert_eq!((part.label(), part.numeric_value(), part.max_numeric_value()), (Some("Treble"), Some(50.0), Some(100.0)));
    assert_ne!(shot(&mut ui_of(tree(0.0))).data(), shot(&mut ui_of(tree(100.0))).data());
}

#[test]
fn a_level_bar_is_heard_as_progress_and_fills_with_its_value() {
    let tree = |v: f32| ww::column(vec![dial::level_bar(v, 0.0, 10.0, "Disk used")]).padding(10.0);
    let mut ui = ui_of(tree(2.5));
    let told = ui.access("Test");
    let part = role_of(&told, accesskit::Role::ProgressIndicator).unwrap();
    assert_eq!((part.label(), part.numeric_value()), (Some("Disk used"), Some(0.25)));
    assert_ne!(shot(&mut ui_of(tree(1.0))).data(), shot(&mut ui_of(tree(9.0))).data());
}

#[test]
fn a_scale_shows_its_marks_and_a_volume_mutes() {
    let mut ui = ui_of(ww::column(vec![dial::scale(1.0, 0.0, 2.0, 1.0, &[(0.0, "Low"), (1.0, "Medium"), (2.0, "High")], "Quality", Msg::Value)]).padding(10.0));
    let said = ui.tree.said();
    assert!(["Low", "Medium", "High"].iter().all(|m| said.iter().any(|w| w == m)), "{said:?}");
    assert!(role_of(&ui.access("Test"), accesskit::Role::Slider).is_some());
    let v = dial::volume(0.5, false, Msg::Mute, Msg::Value);
    assert_eq!(v.presses().into_iter().map(|(t, _, m)| (t, m.clone())).collect::<Vec<_>>(), vec![("Mute".to_string(), Msg::Mute)]);
    assert_eq!(dial::volume(0.5, true, Msg::Mute, Msg::Value).buttons()[0].0, "Unmute");
}

#[test]
fn a_checked_box_shows_red_words_only_once_left_and_nothing_moves() {
    let mut b = CheckedBox::new(Mask::Hex, "");
    let tree = |b: &CheckedBox| ww::column(vec![b.view("Colour", Msg::Check), ww::label("Under").key("under")]).padding(10.0);
    let mut ui = ui_of(tree(&b));
    let below = ui.spot("under").unwrap().rect.y;
    let now = Instant::now();
    ui.input(&key(Keysym::Tab), now);
    for t in ["f", "b", "c", "7"] {
        for m in ui.input(&typed(t), now).messages {
            if let Msg::Check(e) = m {
                b.update(e);
            }
        }
        ui.set_view(tree(&b));
    }
    assert_eq!(b.text, "fbc7");
    assert!(!ui.tree.said().iter().any(|w| w.starts_with("Write")), "not called wrong while typing");
    b.update(validate::Edit::Left);
    let mut ui = ui_of(tree(&b));
    assert!(ui.tree.said().iter().any(|w| w == "Write a colour code such as #fbc711."));
    assert_eq!(ui.spot("under").unwrap().rect.y, below, "the words under it did not move");
    let told = ui.access("Test");
    let entry = role_of(&told, accesskit::Role::TextInput).unwrap();
    assert_eq!(entry.invalid(), Some(accesskit::Invalid::True));
    assert!(role_of(&told, accesskit::Role::Alert).is_some());
}

#[test]
fn a_spin_box_steps_by_its_buttons_and_keys() {
    let tree = |v: f32| ww::column(vec![spin::spin(v, 0.0, 10.0, 1.0, "Copies", Msg::Value)]).padding(10.0);
    let mut ui = ui_of(tree(3.0));
    let now = Instant::now();
    ui.input(&key(Keysym::Tab), now);
    assert_eq!(ui.input(&key(Keysym::space), now).messages, vec![Msg::Value(2.0)]);
    ui.input(&key(Keysym::Tab), now);
    assert_eq!(ui.input(&key(Keysym::Page_Up), now).messages, vec![Msg::Value(10.0)]);
    let told = ui.access("Test");
    assert_eq!(role_of(&told, accesskit::Role::Slider).and_then(|n| n.numeric_value()), Some(3.0));
    assert_ne!(shot(&mut ui_of(tree(3.0))).data(), shot(&mut ui_of(tree(7.0))).data());
}

#[test]
fn settings_rows_name_their_controls_by_the_row() {
    let page = ww::column(vec![
        rows::toggle_row("Wi-Fi", "Join networks near you", true, Msg::On),
        rows::spin_row("Copies", "", 1.0, (1.0, 9.0, 1.0), Msg::Value),
        rows::entry_row("Name", "", "John Smith", |_| Msg::Mute),
        rows::combo_row("Size", "", vec!["Small".into(), "Large".into()], Some(0), |_| Msg::Mute),
    ]).padding(10.0);
    let mut ui = ui_of(page);
    let told = ui.access("Test");
    let named = |role| role_of(&told, role).and_then(|n| n.label().map(str::to_string));
    assert_eq!(named(accesskit::Role::Switch).as_deref(), Some("Wi-Fi"));
    assert_eq!(named(accesskit::Role::Slider).as_deref(), Some("Copies"));
    assert_eq!(named(accesskit::Role::TextInput).as_deref(), Some("Name"));
    assert!(ui.unnamed_controls().is_empty(), "{:?}", ui.unnamed_controls());
}

#[test]
fn an_info_panel_speaks_and_only_a_failure_is_red() {
    let mut note = ui_of(info::info_panel(info::Kind::Note, "Saved to Documents.", vec![]));
    let told = note.access("Test");
    assert!(told.nodes.iter().any(|(_, n)| n.live() == Some(accesskit::Live::Polite)));
    assert!(note.tree.said().iter().any(|w| w == "Saved to Documents."));
    let mut failed = ui_of(info::info_panel(info::Kind::Failed, "The file could not be saved.", vec![ww::button("view-refresh-symbolic", "Try again").on_press(Msg::Mute)]));
    assert!(role_of(&failed.access("Test"), accesskit::Role::Alert).is_some());
    let red = |pix: &tiny_skia::Pixmap| pix.pixels().iter().any(|p| p.red() > 180 && p.green() < 90 && p.blue() < 90);
    assert!(red(&shot(&mut failed)) && !red(&shot(&mut note)));
}

#[test]
fn the_palettes_list_of_matches_has_a_name_a_reader_can_say() {
    let all = actions();
    let mut ui = ui_of(Palette::default().view(&all, Msg::Palette));
    assert!(ui.unnamed_lists().is_empty(), "{:?}", ui.unnamed_lists());
    let told = ui.access("Test");
    assert!(told.nodes.iter().any(|(_, p)| p.role() == accesskit::Role::ListBox && p.label() == Some("Matching actions")));
}
