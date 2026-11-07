//! Waiting input keeps its order, and what is done for a press knows whose press it was.

use super::*;

#[test]
fn moves_that_are_still_waiting_are_kept_as_the_last_of_them() {
    let mut inbox = Inbox::default();
    for n in 0..1000 {
        inbox.push((1, Input::PointerMove { x: n as f32, y: 2.0 * n as f32 }));
    }
    assert_eq!(inbox.next(), Some((1, Input::PointerMove { x: 999.0, y: 1998.0 }, None)));
    assert_eq!(inbox.next(), None);
    inbox.push((1, Input::PointerMove { x: 1.0, y: 1.0 }));
    inbox.push((1, Input::PointerMove { x: 2.0, y: 2.0 }));
    inbox.push_pressed(1, Input::Press { button: Button::Left, x: 2.0, y: 2.0 }, 7);
    inbox.push((1, Input::PointerMove { x: 3.0, y: 3.0 }));
    inbox.push((1, Input::PointerMove { x: 4.0, y: 4.0 }));
    inbox.push((1, Input::Scroll { x: 0.0, y: 1.0, steps: true }));
    inbox.push((1, Input::Scroll { x: 0.0, y: 1.0, steps: true }));
    inbox.push((1, Input::PointerMove { x: 5.0, y: 5.0 }));
    let mut got = Vec::new();
    while let Some((_, input, serial)) = inbox.next() {
        got.push((input, serial));
    }
    assert_eq!(got, [
        (Input::PointerMove { x: 2.0, y: 2.0 }, None),
        (Input::Press { button: Button::Left, x: 2.0, y: 2.0 }, Some(7)),
        (Input::PointerMove { x: 4.0, y: 4.0 }, None),
        (Input::Scroll { x: 0.0, y: 1.0, steps: true }, None),
        (Input::Scroll { x: 0.0, y: 1.0, steps: true }, None),
        (Input::PointerMove { x: 5.0, y: 5.0 }, None),
    ]);
    inbox.push((1, Input::PointerMove { x: 1.0, y: 1.0 }));
    inbox.push((2, Input::PointerMove { x: 2.0, y: 2.0 }));
    inbox.push((2, Input::PointerOut));
    inbox.extend([(2, Input::TouchMove { id: 1, x: 1.0, y: 1.0 }), (2, Input::TouchMove { id: 2, x: 2.0, y: 2.0 }),
                  (2, Input::TouchMove { id: 2, x: 3.0, y: 3.0 })]);
    let mut got = Vec::new();
    while let Some((win, input, _)) = inbox.next() {
        got.push((win, input));
    }
    assert_eq!(got, [
        (1, Input::PointerMove { x: 1.0, y: 1.0 }),
        (2, Input::PointerMove { x: 2.0, y: 2.0 }),
        (2, Input::PointerOut),
        (2, Input::TouchMove { id: 1, x: 1.0, y: 1.0 }),
        (2, Input::TouchMove { id: 2, x: 3.0, y: 3.0 }),
    ]);
}

#[test]
fn a_window_being_placed_keeps_the_moves_before_and_after_apart() {
    let mut inbox = Inbox::default();
    inbox.push((1, Input::PointerMove { x: 1.0, y: 1.0 }));
    inbox.push((1, Input::Placed));
    inbox.push((1, Input::PointerMove { x: 2.0, y: 2.0 }));
    let mut got = Vec::new();
    while let Some((_, input, _)) = inbox.next() {
        got.push(input);
    }
    assert_eq!(got, [Input::PointerMove { x: 1.0, y: 1.0 }, Input::Placed, Input::PointerMove { x: 2.0, y: 2.0 }]);
}

#[test]
fn what_a_press_did_keeps_its_number_and_nothing_else_has_one() {
    let mut inbox = Inbox::default();
    inbox.push((1, Input::PointerOut));
    inbox.push_pressed(1, Input::TouchUp { id: 2 }, 77);
    inbox.extend([(2, Input::Focus(true))]);
    inbox.push_pressed(2, Input::TouchUp { id: 3 }, 78);
    inbox.forget(2);
    assert_eq!(inbox.next(), Some((1, Input::PointerOut, None)));
    assert_eq!(inbox.next(), Some((1, Input::TouchUp { id: 2 }, Some(77))));
    assert_eq!(inbox.next(), None);
    assert_eq!(press_now(), None);
    assert_eq!(with_press(Some(5), press_now), Some(5));
    assert_eq!(press_now(), None);
}

#[test]
fn the_key_really_pressed_is_the_one_behind_a_changed_press() {
    note_code(&CODES, 900, 1);
    note_code(&WAS, 900, 58);
    note_code(&CODES, 901, 30);
    assert_eq!(with_press(Some(900), code_now), Some(58), "a changed press did not give the key really pressed");
    assert_eq!(with_press(Some(901), code_now), Some(30));
    assert_eq!(with_press(Some(902), code_now), None);
    assert_eq!(code_now(), None);
    for serial in 1000..1000 + CODES_KEPT as u32 {
        note_code(&CODES, serial, 2);
    }
    assert_eq!(with_press(Some(901), code_now), None);
}

#[test]
fn only_the_persons_own_press_does_an_act_the_program_keeps_for_them() {
    use crate::host::{with, Act, Host};
    struct Guarded(bool);
    impl Host for Guarded {
        fn act(&self, act: &str) -> Act {
            match act {
                "made-up-stop" => Act::PersonsOwn,
                "made-up-later" => Act::Waiting,
                _ => Act::Unknown,
            }
        }
        fn asks_only_the_person(&self, name: &str) -> bool {
            name == "made-up-question"
        }
        fn reader_switched_on(&self) -> bool {
            self.0
        }
    }
    static OFF: Guarded = Guarded(false);
    static ON: Guarded = Guarded(true);
    with(&OFF, || {
        let stop = || persons_own("made-up-stop");
        assert!(with_named(true, Some(7), || with_press(Some(7), stop)), "the person's own press was refused");
        assert!(!with_named(true, Some(7), || with_press(Some(8), stop)), "a press the compositor never named did the act");
        assert!(!with_named(true, Some(7), stop));
        assert!(with_named(false, None, || with_press(Some(7), stop)), "under a compositor that names no presses, the person's own press was refused");
        assert!(!with_named(true, None, || by_reader(stop)), "a screen reader not switched on by the person did the act");
        assert!(with_named(true, None, || persons_own("made-up-later")), "an act still to be decided on was refused");
        assert!(!with_named(true, Some(7), || with_press(Some(7), || persons_own("no-such-act"))));
        assert!(!with_named(true, None, || reader_may_ask("made-up-question")));
        assert!(with_named(false, None, || reader_may_ask("made-up-question")));
        assert!(with_named(true, None, || reader_may_ask("org.example.Notes")));
    });
    with(&ON, || {
        assert!(with_named(true, None, || by_reader(|| persons_own("made-up-stop"))), "a screen reader the person switched on was shut out");
        assert!(with_named(true, None, || reader_may_ask("made-up-question")));
    });
    assert!(with_named(true, None, || reader_may_ask("made-up-question")));
    assert!(!with_named(true, Some(7), || with_press(Some(7), || persons_own("made-up-stop"))));
    let ui = crate::common::source::code(include_str!("../ui/shown.rs"));
    assert!(ui.contains("if !super::window::reader_may_ask(&name) {") && ui.matches("super::window::by_reader(").count() == 1 && ui.matches(".ui.act(&req)").count() == 1,
            "what the accessibility bus asks is no longer done in one place, behind both rules");
    let mut seen = crate::common::own_press::Seen::default();
    (0..1000).for_each(|serial| seen.note(serial));
    assert!(seen.has(999) && !seen.has(0));
}
