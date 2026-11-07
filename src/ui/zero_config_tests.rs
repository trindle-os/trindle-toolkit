//! Accessibility with nothing set up: a sample program written with the plain widget calls alone, no keys, names or order given, has every control named and given a role, and the keyboard reaches each in reading order.

use super::*;
use super::super::widget::{button, card, choice, column, dropdown, entry, label, main_word, row, slider, switch, tick};

#[derive(Clone, Debug, PartialEq)]
enum Msg {
    Name(String),
    Wifi(bool),
    Volume(f32),
    Size(usize),
    City(usize),
    Agree(bool),
    Save,
    Help,
}

fn sample() -> Node<Msg> {
    column(vec![
        label("Settings"),
        card(column(vec![
            entry("", "Your name", Msg::Name),
            switch(false, "Wi-Fi", Msg::Wifi),
            slider(0.5, 0.0, 1.0, 0.1, "Volume", Msg::Volume),
        ])),
        choice(vec!["Small".into(), "Large".into()], 0, "Text size", Msg::Size),
        dropdown(vec!["London".into(), "Leeds".into()], None, "City", Msg::City),
        tick(false, "I agree", Msg::Agree),
        row(vec![button("help-about-symbolic", "Help").on_press(Msg::Help), main_word("Save").on_press(Msg::Save)]),
    ]).padding(16.0)
}

fn ui() -> Ui<Msg> {
    let mut ui = Ui::new(sample(), Theme::new(false, "#fbc711"));
    ui.lay_at(480, 640);
    ui
}

fn tab(back: bool) -> Input {
    Input::Key { sym: if back { Keysym::ISO_Left_Tab } else { Keysym::Tab }, text: None, mods: Mods { shift: back, ..Mods::default() }, repeat: false }
}

#[test]
fn every_control_of_a_plain_program_is_named_and_has_a_role() {
    let mut ui = ui();
    assert_eq!(ui.unnamed_controls(), Vec::<String>::new());
    let told = ui.access("Sample");
    let roles: Vec<accesskit::Role> = told.nodes.iter().map(|(_, p)| p.role()).collect();
    for role in [accesskit::Role::TextInput, accesskit::Role::Switch, accesskit::Role::Slider, accesskit::Role::RadioGroup,
                 accesskit::Role::ComboBox, accesskit::Role::CheckBox, accesskit::Role::Button] {
        assert!(roles.contains(&role), "no {role:?} in {roles:?}");
    }
}

#[test]
fn tab_reaches_every_control_in_reading_order_and_back() {
    let mut ui = ui();
    let now = Instant::now();
    let order: Vec<String> = (0..9).map(|_| {
        ui.input(&tab(false), now);
        ui.keyboard_on().unwrap()
    }).collect();
    let mut seen = order[..8].to_vec();
    seen.sort();
    seen.dedup();
    assert_eq!(seen.len(), 8, "each control once: {order:?}");
    assert_eq!(order[8], order[0], "round to the first again: {order:?}");
    let place = |id: &str| ui.boxes().into_iter().find(|b| b.0 == id).unwrap().1;
    for pair in order[..8].windows(2) {
        let (a, b) = (place(&pair[0]), place(&pair[1]));
        assert!(b.y > a.y + 0.5 || (b.y - a.y).abs() <= 0.5 && b.x > a.x, "{} comes after {} out of reading order", pair[1], pair[0]);
    }
    let back: Vec<String> = (0..8).map(|_| {
        ui.input(&tab(true), now);
        ui.keyboard_on().unwrap()
    }).collect();
    let expect: Vec<String> = order[..8].iter().rev().cloned().collect();
    assert_eq!(back, expect);
}

#[test]
fn the_keyboard_s_place_is_always_drawn() {
    let mut ui = ui();
    let now = Instant::now();
    for _ in 0..8 {
        ui.input(&tab(false), now);
        let id = ui.keyboard_on().unwrap();
        let mut plain = tiny_skia::Pixmap::new(480, 640).unwrap();
        let held = ui.focus.take();
        ui.paint(&mut plain.as_mut(), 1.0);
        ui.focus = held;
        let mut shown = tiny_skia::Pixmap::new(480, 640).unwrap();
        ui.paint(&mut shown.as_mut(), 1.0);
        assert_ne!(plain.data(), shown.data(), "nothing shows the keyboard is on {id}");
    }
}
