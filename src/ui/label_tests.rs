//! Labels on pointing, remove asking twice, and things that float.

use super::*;
use super::super::widget::{button, card, choice, column, drawing, label, layers, note, placed, remove, row, scroll, slider, switch, Length};

#[derive(Clone, Debug, PartialEq)]
enum Msg {
    Go,
    Gone,
    On(bool),
    Set(f32),
    Pick(usize),
}

const LONG: &str = "Close all of the windows this program has open, on every workspace, with nothing kept back at all";

fn every_kind() -> Node<Msg> {
    let page = column(vec![
        row(vec![
            button("document-open-symbolic", "Save").on_press(Msg::Go).key("button"),
            button("x", "Bigger").size(40.0).on_press(Msg::Go).key("sized"),
            remove("user-trash-symbolic", "Remove").on_press(Msg::Gone).key("remove"),
            button("x", LONG).on_press(Msg::Go).key("long"),
        ]),
        switch(true, "Wi-Fi", Msg::On).key("switch"),
        slider(0.5, 0.0, 1.0, 0.1, "Volume", Msg::Set).key("slider"),
        choice(vec!["One".into(), "Two".into()], 0, "Which", Msg::Pick).key("choice"),
        drawing((60.0, 40.0), "All apps", |_, _| {}).pressable(Msg::Go).key("drawing"),
        label("A name far too long for the little room it has been given here").cut().width(Length::Fixed(80.0)).key("cut"),
        scroll("area", column(vec![button("x", "In a scroll").on_press(Msg::Go).key("scrolled")])).height(Length::Fixed(60.0)),
    ]).padding(10.0);
    layers(vec![
        page,
        placed(card(row(vec![note("Floating"), button("x", "On a card").on_press(Msg::Go).key("floating")])),
               |(w, _), (aw, _)| ((aw - w) / 2.0, 600.0)),
    ])
}

fn laid(scale: f32) -> Ui<Msg> {
    let mut ui = Ui::new(every_kind(), Theme::new(false, "#fbc711"));
    ui.size = ((1280.0 * scale) as u32, (720.0 * scale) as u32);
    ui.scale = scale;
    ui.settle();
    ui
}

#[test]
fn every_label_is_as_wide_as_its_words_and_no_wider() {
    for scale in [1.0, 1.75] {
        let ui = laid(scale);
        let mut pixmap = tiny_skia::Pixmap::new(ui.size.0, ui.size.1).unwrap();
        let cw = ui.size.0 as f32;
        let k = theme::sheet();
        let mut seen = 0;
        for spot in ui.spots.clone() {
            let Some(text) = ui.label_of(&spot.id) else { continue };
            let mut ui = laid(scale);
            ui.tip = Some(Tip { id: spot.id.clone(), state: TipState::Shown(Instant::now()) });
            let drawn = ui.tip_drawn(&mut pixmap.as_mut()).expect("a label is drawn");
            let layout = label_layout(&ui.fonts, &text, scale, cw);
            let words = layout.widest().ceil();
            let pad = (k.label_padding.0 * scale).round();
            assert!((drawn.w - (words + 2.0 * pad)).abs() < 1.0, "{} at {scale}: {} wide for {words}", spot.id, drawn.w);
            assert!(drawn.w < cw * 0.6, "{} at {scale} runs across the screen: {}", spot.id, drawn.w);
            assert!(drawn.x >= 0.0 && drawn.x + drawn.w <= cw, "{} at {scale} runs off the screen", spot.id);
            assert!(drawn.y >= spot.rect.y + spot.rect.h, "{} at {scale} is not below what it names", spot.id);
            seen += 1;
        }
        assert!(seen >= 9, "only {seen} labels were found at {scale}");
    }
}

#[test]
fn a_card_floating_over_the_rest_is_as_wide_as_what_it_holds() {
    for scale in [1.0, 1.75] {
        let ui = laid(scale);
        let card = ui.spots.iter().find(|s| matches!(ui.tree.at(&s.path).map(|n| &n.what), Some(What::Card(..)))).unwrap();
        assert!(card.rect.w < 300.0 * scale, "a floating card is {} wide at {scale}", card.rect.w);
    }
    let mut ui: Ui<Msg> = Ui::new(layers(vec![placed(card(note("Drag across the part to keep")), |(w, _), (aw, _)| ((aw - w) / 2.0, 40.0))]),
                                  Theme::new(false, "#fbc711"));
    ui.size = (1280, 720);
    ui.settle();
    let hint = ui.spots.iter().find(|s| matches!(ui.tree.at(&s.path).map(|n| &n.what), Some(What::Card(..)))).unwrap();
    assert!(hint.rect.w < 400.0 && hint.rect.x > 400.0, "the hint is at {:?}", hint.rect);
}

#[test]
fn a_label_goes_after_a_moment_unless_it_is_kept() {
    for (labels, stays) in [(Labels::Brief, false), (Labels::WhilePointing, true)] {
        let mut ui = laid(1.0);
        ui.set_labels(labels);
        ui.settle();
        let r = ui.spot("button").unwrap().rect;
        let start = Instant::now();
        ui.input(&Input::PointerIn { x: r.x + 4.0, y: r.y + 4.0 }, start);
        assert!(!ui.tick(start + tip_rest() - Duration::from_millis(1)), "not before the rest is over");
        assert!(ui.tick(start + tip_rest()));
        ui.tick(start + tip_rest() + tip_stays());
        let shown = matches!(ui.tip.as_ref().map(|t| t.state), Some(TipState::Shown(_)));
        assert_eq!(shown, stays, "{labels:?}");
    }
}

#[test]
fn with_the_screen_reader_every_round_button_has_its_name_under_it() {
    let mut ui = laid(1.0);
    let before = ui.spot("button").unwrap().rect;
    ui.set_labels(Labels::Always);
    ui.settle();
    let after = ui.spot("button").unwrap().rect;
    assert!(after.h > before.h + 10.0, "room is made for the name: {before:?} then {after:?}");
    assert_eq!(ui.label_of("button"), None);
    assert_eq!(ui.label_of("switch").as_deref(), Some("Wi-Fi"));
}

#[test]
fn a_choice_follows_its_remove_button_while_armed() {
    let tree = row(vec![
        remove("user-trash-symbolic", "Remove").on_press(Msg::Gone).key("rm"),
        switch(false, "Delete its data too", Msg::On).key("wipe").when_armed("rm"),
    ]);
    let mut ui = Ui::new(tree, Theme::new(false, "#fbc711"));
    ui.size = (1280, 720);
    ui.scale = 1.0;
    ui.settle();
    let shown = |ui: &Ui<Msg>| ui.node("wipe").map(|n| (!n.off, n.opacity > 0.0));
    let at = ui.spot("wipe").unwrap().rect;
    assert_eq!(shown(&ui), Some((false, false)), "not shown before the first press, but its room is kept");
    let r = ui.spot("rm").unwrap().rect;
    let (x, y) = (r.x + r.w / 2.0, r.y + r.h / 2.0);
    let now = Instant::now();
    let click = |ui: &mut Ui<Msg>, x: f32, y: f32| {
        ui.input(&Input::Press { button: Button::Left, x, y }, now);
        ui.input(&Input::Release { button: Button::Left, x, y }, now).messages
    };
    assert!(click(&mut ui, x, y).is_empty());
    assert_eq!(shown(&ui), Some((true, true)), "shown once Remove is armed");
    assert_eq!(ui.spot("wipe").unwrap().rect, at, "and nothing moved");
    let w = ui.spot("wipe").unwrap().rect;
    assert_eq!(click(&mut ui, w.x + w.w / 2.0, w.y + w.h / 2.0), vec![Msg::On(true)]);
    assert!(ui.is_armed("rm"), "choosing leaves Remove armed");
    assert_eq!(click(&mut ui, x, y), vec![Msg::Gone]);
    assert_eq!(shown(&ui), Some((false, false)), "gone again after the removal");
    assert!(click(&mut ui, x, y).is_empty());
    assert_eq!(shown(&ui), Some((true, true)));
    assert!(ui.tick(Instant::now() + Duration::from_secs(4)));
    assert_eq!(shown(&ui), Some((false, false)), "gone again when the arm lapses");
    assert!(click(&mut ui, w.x + w.w / 2.0, w.y + w.h / 2.0).is_empty());
}

#[test]
fn a_greyed_part_stays_grey_when_its_remove_button_is_armed() {
    let tree = row(vec![
        remove("user-trash-symbolic", "Remove").on_press(Msg::Gone).key("rm"),
        row(vec![
            switch(false, "Delete its data too", Msg::On).key("wipe").off(true),
            switch(false, "Keep a copy", Msg::On).key("copy"),
        ]).when_armed("rm"),
    ]);
    let mut ui = Ui::new(tree, Theme::new(false, "#fbc711"));
    ui.size = (1280, 720);
    ui.scale = 1.0;
    ui.settle();
    let off = |ui: &Ui<Msg>, k| ui.node(k).map(|n| n.off);
    assert_eq!((off(&ui, "wipe"), off(&ui, "copy")), (Some(true), Some(true)));
    let r = ui.spot("rm").unwrap().rect;
    let (x, y) = (r.x + r.w / 2.0, r.y + r.h / 2.0);
    let now = Instant::now();
    let arm = |ui: &mut Ui<Msg>| {
        ui.input(&Input::Press { button: Button::Left, x, y }, now);
        ui.input(&Input::Release { button: Button::Left, x, y }, now);
    };
    arm(&mut ui);
    assert!(ui.tick(now + Duration::from_secs(4)), "armed, then lapsed");
    arm(&mut ui);
    assert!(ui.is_armed("rm"));
    assert_eq!((off(&ui, "wipe"), off(&ui, "copy")), (Some(true), Some(false)), "the greyed one stays grey");
    let What::Row { items, .. } = &ui.tree.what else { unreachable!() };
    assert_eq!(items[1].opacity, 1.0);
}

#[test]
fn remove_asks_twice() {
    let mut ui = laid(1.0);
    let r = ui.spot("remove").unwrap().rect;
    let (x, y) = (r.x + r.w / 2.0, r.y + r.h / 2.0);
    let now = Instant::now();
    let click = |ui: &mut Ui<Msg>| {
        ui.input(&Input::Press { button: Button::Left, x, y }, now);
        ui.input(&Input::Release { button: Button::Left, x, y }, now).messages
    };
    assert!(click(&mut ui).is_empty(), "the first press only arms it");
    assert!(ui.is_armed("remove"));
    assert_eq!(ui.label_of("remove").as_deref(), Some("Click again to remove"));
    assert_eq!(click(&mut ui), vec![Msg::Gone]);
    assert!(!ui.is_armed("remove"));
    assert!(click(&mut ui).is_empty());
    assert!(ui.next_wake(Instant::now()).is_some());
    assert!(ui.tick(Instant::now() + Duration::from_secs(3)));
    assert!(!ui.is_armed("remove"));
    let b = ui.spot("button").unwrap().rect;
    ui.input(&Input::Press { button: Button::Left, x: b.x + 4.0, y: b.y + 4.0 }, now);
    assert_eq!(ui.input(&Input::Release { button: Button::Left, x: b.x + 4.0, y: b.y + 4.0 }, now).messages, vec![Msg::Go]);
}

#[test]
fn a_made_up_press_cannot_arm_a_button_of_an_act_only_the_person_may_do() {
    use super::super::widget::remove_by_hand;
    use crate::window::{refused_act, with_named, with_press};
    struct Guarded;
    const SETTINGS_POWER: &str = "made-up-power";
    impl crate::host::Host for Guarded {
        fn act(&self, act: &str) -> crate::host::Act {
            if act == SETTINGS_POWER { crate::host::Act::PersonsOwn } else { crate::host::Act::Unknown }
        }
    }
    let mut ui = Ui::new(column(vec![remove_by_hand("x", "Shut down", SETTINGS_POWER).on_press(Msg::Gone).key("off")]).padding(10.0),
                         Theme::new(false, "#fbc711"));
    ui.size = (400, 300);
    ui.scale = 1.0;
    ui.settle();
    let r = ui.spot("off").unwrap().rect;
    let (x, y) = (r.x + r.w / 2.0, r.y + r.h / 2.0);
    let now = Instant::now();
    let click = |ui: &mut Ui<Msg>, serial: u32| {
        crate::host::with(&Guarded, || with_named(true, Some(7), || with_press(Some(serial), || {
            ui.input(&Input::Press { button: Button::Left, x, y }, now);
            ui.input(&Input::Release { button: Button::Left, x, y }, now).messages
        })))
    };
    let _ = refused_act();
    assert!(click(&mut ui, 8).is_empty() && !ui.is_armed("off"), "a made-up press armed it");
    assert!(refused_act(), "a refused arming asked for no words");
    assert!(click(&mut ui, 7).is_empty() && ui.is_armed("off"), "the person's own press did not arm it");
    assert!(!refused_act());
    assert!(click(&mut ui, 8).is_empty() && !ui.is_armed("off"));
    assert!(refused_act());
    assert!(click(&mut ui, 7).is_empty());
    assert_eq!(click(&mut ui, 7), vec![Msg::Gone]);
    assert!(!ui.is_armed("off"));
}
