//! A control shut while a job runs is greyed out and says why, in its label on pointing.

use super::*;
use super::super::widget::{self as ww};

const WHY: &str = "Wait for the job to finish.";

fn view(why: Option<&str>) -> Node<u8> {
    ww::column(vec![
        ww::button("a", "Update").on_press(1).key("round").off_because(why),
        ww::main_action("b", "Install").on_press(2).key("worded").off_because(why),
        ww::row(vec![ww::button("c", "Remove").on_press(3).key("inner")]).off_because(why),
    ])
}

fn ui_of(why: Option<&str>) -> Ui<u8> {
    let mut ui = Ui::new(view(why), super::super::theme::Theme::new(false, "#fbc711"));
    ui.lay_at(400, 300);
    ui
}

#[test]
fn a_control_shut_by_a_job_is_greyed_out_and_says_why() {
    let ui = ui_of(Some(WHY));
    for (key, tip) in [("round", "Update\n"), ("worded", ""), ("inner", "Remove\n")] {
        assert!(ui.node(key).unwrap().off, "{key} is greyed out");
        assert_eq!(ui.label_of(key).as_deref(), Some(format!("{tip}{WHY}").as_str()), "{key} says why");
    }
    let shut = view(Some(WHY)).shut();
    let tips: Vec<&str> = shut.iter().map(|s| s.0.as_str()).collect();
    assert_eq!(tips, ["Update", "Install", "Remove"], "exactly these three, in order");
    assert!(shut.iter().all(|(_, off, why)| *off && why.as_deref() == Some(WHY)), "{shut:?}");
}

#[test]
fn nothing_stays_grey_when_the_job_ends() {
    let ui = ui_of(None);
    assert!(!ui.node("round").unwrap().off && ui.label_of("round").as_deref() == Some("Update"));
    let shut = view(None).shut();
    assert_eq!(shut.len(), 3, "the buttons are still listed");
    assert!(shut.iter().all(|(_, off, why)| !*off && why.is_none()), "{shut:?}");
}

#[test]
fn pressing_a_shut_control_does_nothing_and_a_free_one_acts() {
    let mut shut = ui_of(Some(WHY));
    for key in ["round", "worded", "inner"] {
        let mut reply = Reply::new();
        shut.activate(key, std::time::Instant::now(), &mut reply);
        assert!(reply.messages.is_empty(), "{key} acted while shut");
    }
    let mut free = ui_of(None);
    for (key, msg) in [("round", 1), ("worded", 2), ("inner", 3)] {
        let mut reply = Reply::new();
        free.activate(key, std::time::Instant::now(), &mut reply);
        assert_eq!(reply.messages, vec![msg], "{key} did not act when free");
    }
}

#[test]
fn the_keyboard_passes_over_shut_controls_and_finds_them_again_when_free() {
    let mut shut = ui_of(Some(WHY));
    shut.move_focus(true);
    assert_eq!(shut.focus, None, "the keyboard landed on a greyed-out control");
    let mut free = ui_of(None);
    free.move_focus(true);
    assert_eq!(free.focus.as_deref(), Some("round"));
}

#[test]
fn the_reason_reaches_buttons_inside_cards_scrolls_and_layers() {
    let tree: Node<u8> = ww::layers(vec![ww::scroll("s", ww::card(ww::column(vec![ww::button("a", "Deep").on_press(1)])))]).off_because(Some(WHY));
    let shut = tree.shut();
    assert_eq!(shut.len(), 1);
    assert!(shut[0].1 && shut[0].2.as_deref() == Some(WHY), "{shut:?}");
}
