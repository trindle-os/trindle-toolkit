//! Tests of drags over drawings and of what is held over rows and choices.

use super::*;

#[test]
fn a_drawing_hears_a_drag_pass_over_it() {
    use crate::widget::Pointer;
    let told = |n: usize| move |p: Pointer| match p {
        Pointer::Down(..) => Msg::Pick(n * 10 + 1),
        Pointer::Move(..) => Msg::Pick(n * 10 + 2),
        Pointer::Up(..) => Msg::Pick(n * 10 + 3),
        Pointer::Hover(..) => Msg::Pick(n * 10 + 4),
        Pointer::Leave => Msg::Pick(n * 10 + 5),
        _ => Msg::Go,
    };
    let two = row(vec![
        drawing((100.0, 100.0), "from", |_, _| {}).on_pointer(told(1)).on_hover(),
        drawing((100.0, 100.0), "onto", |_, _| {}).on_pointer(told(2)).on_hover(),
    ]).gap(0.0);
    let mut ui = Ui::new(two, Theme::new(false, "#fbc711"));
    ui.size = (200, 100);
    ui.settle();
    let now = Instant::now();
    ui.input(&Input::PointerMove { x: 50.0, y: 50.0 }, now);
    ui.input(&Input::Press { button: Button::Left, x: 50.0, y: 50.0 }, now);
    let got = ui.input(&Input::PointerMove { x: 150.0, y: 50.0 }, now).messages;
    assert!(got.contains(&Msg::Pick(12)) && got.contains(&Msg::Pick(24)), "{got:?}");
    let got = ui.input(&Input::PointerMove { x: 60.0, y: 50.0 }, now).messages;
    assert!(got.contains(&Msg::Pick(25)), "{got:?}");
}

#[test]
fn the_right_button_drags_from_a_drawing_that_asks_and_a_click_is_still_a_menu() {
    use crate::widget::Pointer;
    let told = |n: usize| move |p: Pointer| match p {
        Pointer::RightDown(x, y) => Msg::Menu(n * 10 + 1, x as i32, y as i32),
        Pointer::RightMove(x, y) => Msg::Menu(n * 10 + 2, x as i32, y as i32),
        Pointer::RightUp(x, y) => Msg::Menu(n * 10 + 3, x as i32, y as i32),
        Pointer::Menu(x, y) => Msg::Menu(n * 10 + 4, x as i32, y as i32),
        Pointer::Hover(..) => Msg::Pick(n * 10 + 5),
        Pointer::Leave => Msg::Pick(n * 10 + 6),
        _ => Msg::Go,
    };
    let build = move || column(vec![
        row(vec![
            drawing((100.0, 100.0), "from", |_, _| {}).on_pointer(told(1)).on_hover().on_right_drag(),
            drawing((100.0, 100.0), "onto", |_, _| {}).on_pointer(told(2)).on_hover(),
        ]).gap(0.0),
        list(vec![item("Home"), item("Documents"), item("Trash")], None, Msg::Pick).on_drop(|row| Msg::Many(vec![row])).key("places"),
        choice(vec!["One".into(), "Two".into()], 0, "Tabs", Msg::Pick).on_drop(|i| Msg::Many(vec![100 + i])).key("tabs"),
    ]).gap(0.0);
    let mut ui = Ui::new(build(), Theme::new(false, "#fbc711"));
    ui.size = (300, 400);
    ui.settle();
    let now = Instant::now();
    let right = |down: bool, x: f32, y: f32| if down { Input::Press { button: Button::Right, x, y } } else { Input::Release { button: Button::Right, x, y } };
    ui.input(&Input::PointerMove { x: 50.0, y: 50.0 }, now);
    assert!(ui.input(&right(true, 50.0, 50.0), now).messages.is_empty());
    assert!(ui.input(&Input::PointerMove { x: 52.0, y: 51.0 }, now).messages.is_empty(), "a small move is still a click");
    assert_eq!(ui.input(&right(false, 52.0, 51.0), now).messages, vec![Msg::Menu(14, 50, 50)]);
    ui.input(&right(true, 50.0, 50.0), now);
    let got = ui.input(&Input::PointerMove { x: 150.0, y: 60.0 }, now).messages;
    assert_eq!(got[..2], [Msg::Menu(11, 50, 50), Msg::Menu(12, 150, 60)], "{got:?}");
    assert!(got.contains(&Msg::Pick(25)), "{got:?}");
    let r = ui.spot("places").unwrap().rect;
    let (x, y) = (r.x + 20.0, r.y + ui.px(theme::sheet().row) * 1.5);
    ui.input(&Input::PointerMove { x, y }, now);
    assert_eq!(ui.hover.as_deref(), Some("places"), "the row under what is held is lit");
    let got = ui.input(&right(false, x, y), now).messages;
    assert_eq!(got, vec![Msg::Many(vec![1]), Msg::Menu(13, x as i32, y as i32)]);
    assert!(ui.hover.is_none() && ui.right.is_none());
    let t = ui.spot("tabs").unwrap().rect;
    ui.input(&Input::PointerMove { x: 50.0, y: 50.0 }, now);
    ui.input(&Input::Press { button: Button::Left, x: 50.0, y: 50.0 }, now);
    ui.input(&Input::PointerMove { x: t.x + 12.0, y: t.y + t.h / 2.0 }, now);
    let got = ui.input(&Input::Release { button: Button::Left, x: t.x + 12.0, y: t.y + t.h / 2.0 }, now).messages;
    assert_eq!(got[0], Msg::Many(vec![100]), "{got:?}");
    let got = ui.input(&right(true, 150.0, 50.0), now).messages;
    assert_eq!(got, vec![Msg::Menu(24, 50, 50)]);
    assert!(ui.input(&right(false, 150.0, 50.0), now).messages.is_empty());
}

#[test]
fn only_the_rows_and_choices_that_take_what_is_held_are_lit_and_hear_of_it() {
    let build = || column(vec![
        drawing((100.0, 100.0), "from", |_, _| {}).on_pointer(|_| Msg::Go),
        list(vec![item("Home"), item("Read only")], None, Msg::Pick).on_drop_where(|row| row == 0, |row| Msg::Many(vec![row])).key("places"),
        choice(vec!["One".into(), "Two".into()], 0, "Tabs", Msg::Pick).on_drop_where(|i| i == 1, |i| Msg::Many(vec![100 + i])).key("tabs"),
    ]).gap(0.0);
    let mut ui = Ui::new(build(), Theme::new(false, "#fbc711"));
    ui.size = (300, 400);
    ui.settle();
    let now = Instant::now();
    let r = ui.spot("places").unwrap().rect;
    let t = ui.spot("tabs").unwrap().rect;
    let tab_y = t.y + t.h / 2.0;
    let on_tab = |ui: &Ui<Msg>, i: usize| (0..t.w as usize).map(|at| t.x + at as f32).find(|x| ui.pill("tabs", *x) == Some(i)).unwrap();
    let (first_tab, second_tab) = (on_tab(&ui, 0), on_tab(&ui, 1));
    let row_h = ui.px(theme::sheet().row);
    let row_y = |i: f32| r.y + row_h * (i + 0.5);
    let dropped = |msgs: &[Msg]| msgs.iter().filter(|m| matches!(m, Msg::Many(_))).cloned().collect::<Vec<_>>();
    let hold = |ui: &mut Ui<Msg>| {
        ui.input(&Input::PointerMove { x: 50.0, y: 50.0 }, now);
        ui.input(&Input::Press { button: Button::Left, x: 50.0, y: 50.0 }, now);
    };
    for (x, y, lit, got) in [(r.x + 20.0, row_y(0.0), Some("places"), vec![Msg::Many(vec![0])]), (r.x + 20.0, row_y(1.0), None, vec![]),
                             (second_tab, tab_y, Some("tabs"), vec![Msg::Many(vec![101])]), (first_tab, tab_y, None, vec![])] {
        hold(&mut ui);
        ui.input(&Input::PointerMove { x, y }, now);
        assert_eq!(ui.hover.as_deref(), lit, "at {x}, {y}");
        let told = ui.input(&Input::Release { button: Button::Left, x, y }, now).messages;
        assert_eq!(dropped(&told), got, "at {x}, {y}");
    }
    hold(&mut ui);
    ui.input(&Input::PointerMove { x: r.x + 20.0, y: row_y(0.0) }, now);
    assert_eq!(ui.hover.as_deref(), Some("places"));
    ui.input(&Input::PointerMove { x: r.x + 20.0, y: row_y(1.0) }, now);
    assert!(ui.hover.is_none());
}
