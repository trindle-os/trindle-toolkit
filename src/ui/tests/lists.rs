//! Tests of lists and rows of choices: choosing, picking, menus and moving rows.

use super::*;

fn path_names() -> Vec<String> {
    vec!["Home".to_string(), "Papers".to_string(), "Albums".to_string()]
}

fn sub_rows(names: &[&str]) -> Rc<Vec<String>> {
    Rc::new(names.iter().map(|s| s.to_string()).collect::<Vec<_>>())
}

fn sub_reading() -> crate::widget::Sub {
    crate::widget::Sub::Words("Reading\u{2026}".into())
}

fn path_view(second: crate::widget::Sub) -> Node<Msg> {
    use crate::widget::Sub;
    column(vec![
        choice(path_names(), 2, "Where this pane is", Msg::Pick)
            .arrows(vec![Sub::Rows(sub_rows(&["Music", "Papers", "Pictures"]), Some(1)), second, Sub::None],
                    |i| Msg::Many(vec![10 + i]), |i, row| Msg::Many(vec![i, row])).key("path"),
        button("a", "Under it").on_press(Msg::Go).key("under"),
    ]).gap(8.0)
}

fn click_at(ui: &mut Ui<Msg>, x: f32, y: f32, now: Instant) -> Vec<Msg> {
    ui.input(&Input::PointerMove { x, y }, now);
    let mut all = ui.input(&Input::Press { button: Button::Left, x, y }, now).messages;
    all.extend(ui.input(&Input::Release { button: Button::Left, x, y }, now).messages);
    all
}

fn the_first_arrow_opens_its_list(ui: &mut Ui<Msg>, pills: &[Rect], y: f32, now: Instant) {
    let before = ui.boxes();
    assert_eq!(click_at(ui, pills[0].x + pills[0].w - 6.0, y, now), vec![Msg::Many(vec![10])]);
    assert!(ui.dropdown_open());
    assert_eq!(ui.open_rows(), ["Music", "Papers", "Pictures"]);
    assert_eq!(ui.drop.as_ref().and_then(|d| d.lit), Some(1), "the one marked");
    let (list, own) = ui.open_list().unwrap();
    assert_eq!(own, pills[0]);
    assert!(list.y >= own.y + own.h && (list.x - own.x).abs() <= 4.0, "{list:?} {own:?}");
    assert_eq!(ui.boxes(), before);
    let mut pix = tiny_skia::Pixmap::new(500, 400).unwrap();
    ui.paint(&mut pix.as_mut(), 1.0);
    ui.input(&key(Keysym::Down, false), now);
    assert_eq!(ui.input(&key(Keysym::Return, false), now).messages, vec![Msg::Many(vec![0, 2])]);
    assert!(!ui.dropdown_open());
}

fn the_second_arrow_opens_words_then_rows(ui: &mut Ui<Msg>, pills: &[Rect], y: f32, now: Instant) {
    use crate::widget::Sub;
    assert_eq!(click_at(ui, pills[1].x + pills[1].w - 6.0, y, now), vec![Msg::Many(vec![11])]);
    assert_eq!(ui.open_rows(), ["Reading\u{2026}"]);
    assert_eq!(ui.drop.as_ref().and_then(|d| d.lit), None);
    ui.set_view(path_view(Sub::Rows(sub_rows(&["2025", "2026"]), None)));
    ui.settle();
    assert!(ui.dropdown_open());
    assert_eq!(ui.open_rows(), ["2025", "2026"]);
    let laid = ui.drop_laid().unwrap();
    assert_eq!(click_at(ui, laid.rows.x + 10.0, laid.rows.y + laid.row * 1.5, now), vec![Msg::Many(vec![1, 1])]);
    ui.set_view(path_view(sub_reading()));
    ui.settle();
    click_at(ui, pills[1].x + pills[1].w - 6.0, y, now);
    ui.input(&key(Keysym::Down, false), now);
    assert!(ui.input(&key(Keysym::Return, false), now).messages.is_empty());
    assert!(!ui.dropdown_open());
}

fn the_keyboard_goes_along_the_choices(ui: &mut Ui<Msg>, bare: &mut Ui<Msg>, now: Instant) {
    ui.focus = Some("path".into());
    ui.along = None;
    assert!(ui.input(&key(Keysym::Left, false), now).messages.is_empty());
    assert_eq!(ui.along, Some(("path".to_string(), 1)));
    assert!(ui.input(&key(Keysym::Left, false), now).messages.is_empty());
    assert!(ui.input(&key(Keysym::Left, false), now).messages.is_empty());
    assert_eq!(ui.along, Some(("path".to_string(), 0)));
    assert_eq!(ui.input(&key(Keysym::Down, false), now).messages, vec![Msg::Many(vec![10])]);
    assert!(ui.dropdown_open());
    ui.input(&key(Keysym::Escape, false), now);
    assert!(!ui.dropdown_open());
    assert_eq!(ui.input(&key(Keysym::Return, false), now).messages, vec![Msg::Many(vec![10])]);
    ui.input(&key(Keysym::Escape, false), now);
    assert_eq!(ui.input(&key(Keysym::space, false), now).messages, vec![Msg::Pick(0)]);
    ui.input(&key(Keysym::Right, false), now);
    ui.input(&key(Keysym::End, false), now);
    assert_eq!(ui.along, Some(("path".to_string(), 2)));
    assert!(ui.input(&key(Keysym::Return, false), now).messages.is_empty());
    assert!(!ui.dropdown_open());
    bare.focus = Some("path".into());
    assert_eq!(bare.input(&key(Keysym::Left, false), now).messages, vec![Msg::Pick(1)]);
}

#[test]
fn each_of_a_row_of_choices_with_arrows_opens_a_list_of_its_own() {
    let mut ui = Ui::new(path_view(sub_reading()), Theme::new(false, "#fbc711"));
    ui.size = (500, 400);
    ui.settle();
    let now = Instant::now();
    let pills = ui.pill_rects("path");
    let mut bare = Ui::new(column(vec![choice(path_names(), 2, "Where this pane is", Msg::Pick).key("path")]), Theme::new(false, "#fbc711"));
    bare.size = (500, 400);
    bare.settle();
    let plain = bare.pill_rects("path");
    assert!(pills[0].w > plain[0].w + 10.0 && pills[1].w > plain[1].w + 10.0 && (pills[2].w - plain[2].w).abs() < 0.5, "{pills:?} {plain:?}");
    let y = pills[0].y + pills[0].h / 2.0;
    assert_eq!(click_at(&mut ui, pills[0].x + 6.0, y, now), vec![Msg::Pick(0)]);
    assert!(!ui.dropdown_open());
    the_first_arrow_opens_its_list(&mut ui, &pills, y, now);
    the_second_arrow_opens_words_then_rows(&mut ui, &pills, y, now);
    the_keyboard_goes_along_the_choices(&mut ui, &mut bare, now);
    ui.input(&key(Keysym::Home, false), now);
    ui.input(&key(Keysym::Down, false), now);
    assert!(ui.dropdown_open());
    ui.set_view(column(vec![choice(path_names(), 2, "Where this pane is", Msg::Pick).key("path")]));
    ui.settle();
    assert!(!ui.dropdown_open());
}

#[test]
fn a_lists_rows_are_dragged_to_a_new_place_and_things_held_land_between_them() {
    use crate::widget::Pointer;
    let build = || column(vec![
        drawing((300.0, 60.0), "from", |_, _| {}).on_pointer(|p| match p {
            Pointer::Up(..) => Msg::Go,
            _ => Msg::Pick(99),
        }),
        list(vec![item("Home"), item("Documents"), item("Music"), item("Trash")], None, Msg::Pick)
            .on_reorder(|from, gap| Msg::Many(vec![from, gap]))
            .on_drop_between(|gap| Msg::Many(vec![100 + gap]))
            .on_drop(|row| Msg::Many(vec![200 + row])).key("places"),
    ]).gap(0.0);
    let mut ui = Ui::new(build(), Theme::new(false, "#fbc711"));
    ui.size = (300, 400);
    ui.settle();
    let now = Instant::now();
    let r = ui.spot("places").unwrap().rect;
    let h = ui.px(theme::sheet().row);
    let row_y = |i: f32| r.y + h * i;
    let left = |down: bool, y: f32| if down { Input::Press { button: Button::Left, x: 40.0, y } } else { Input::Release { button: Button::Left, x: 40.0, y } };
    ui.input(&left(true, row_y(1.5)), now);
    assert_eq!(ui.input(&left(false, row_y(1.5)), now).messages, vec![Msg::Pick(1)]);
    ui.input(&left(true, row_y(3.5)), now);
    ui.input(&Input::PointerMove { x: 40.0, y: row_y(3.4) }, now);
    assert!(ui.gap.is_none(), "a small move is still a click");
    ui.input(&Input::PointerMove { x: 40.0, y: row_y(1.1) }, now);
    assert_eq!(ui.gap, Some(("places".to_string(), 1)));
    assert_eq!(ui.input(&left(false, row_y(1.1)), now).messages, vec![Msg::Many(vec![3, 1])]);
    assert!(ui.gap.is_none() && ui.row_drag.is_none());
    ui.input(&left(true, row_y(0.5)), now);
    ui.input(&Input::PointerMove { x: 40.0, y: row_y(9.0) }, now);
    assert_eq!(ui.input(&left(false, row_y(9.0)), now).messages, vec![Msg::Many(vec![0, 4])]);
    ui.input(&left(true, row_y(1.5)), now);
    ui.input(&Input::PointerMove { x: 40.0, y: row_y(2.2) }, now);
    assert!(ui.input(&left(false, row_y(2.2)), now).messages.is_empty());
    ui.input(&Input::Press { button: Button::Left, x: 40.0, y: 20.0 }, now);
    ui.input(&Input::PointerMove { x: 40.0, y: row_y(2.1) }, now);
    assert_eq!(ui.gap, Some(("places".to_string(), 2)), "a line shows where it would land");
    assert_eq!(ui.input(&left(false, row_y(2.1)), now).messages, vec![Msg::Many(vec![102]), Msg::Go]);
    ui.input(&Input::Press { button: Button::Left, x: 40.0, y: 20.0 }, now);
    ui.input(&Input::PointerMove { x: 40.0, y: row_y(2.5) }, now);
    assert!(ui.gap.is_none());
    assert_eq!(ui.input(&left(false, row_y(2.5)), now).messages, vec![Msg::Many(vec![202]), Msg::Go]);
    ui.input(&Input::Press { button: Button::Left, x: 40.0, y: 20.0 }, now);
    ui.input(&Input::PointerMove { x: 40.0, y: row_y(3.9) }, now);
    assert_eq!(ui.gap, Some(("places".to_string(), 4)));
    let mut pix = tiny_skia::Pixmap::new(300, 400).unwrap();
    ui.paint(&mut pix.as_mut(), 1.0);
    let accent = ui.theme.accent.to_skia().to_color_u8();
    let lined = (0..400).any(|y| pix.pixel(150, y).is_some_and(|c| {
        let c = c.demultiply();
        (c.red(), c.green(), c.blue()) == (accent.red(), accent.green(), accent.blue())
    }));
    assert!(lined, "the line in the highlight colour");
    assert_eq!(ui.input(&left(false, row_y(3.9)), now).messages, vec![Msg::Many(vec![104]), Msg::Go]);
}

#[test]
fn a_middle_click_is_told_to_a_list_or_a_row_of_choices_that_asks() {
    let build = || column(vec![
        list(vec![item("Home"), item("Documents"), item("Trash")], None, Msg::Pick).on_middle(|row| Msg::Many(vec![row])).key("places"),
        choice(vec!["One".into(), "Two".into()], 0, "Tabs", Msg::Pick).on_middle(|i| Msg::Many(vec![100 + i])).key("tabs"),
        list(vec![item("Plain")], None, Msg::Pick).key("plain"),
    ]).gap(0.0);
    let mut ui = Ui::new(build(), Theme::new(false, "#fbc711"));
    ui.size = (300, 400);
    ui.settle();
    let now = Instant::now();
    let middle = |x: f32, y: f32| Input::Press { button: Button::Middle, x, y };
    let r = ui.spot("places").unwrap().rect;
    let got = ui.input(&middle(r.x + 20.0, r.y + ui.px(theme::sheet().row) * 2.5), now).messages;
    assert_eq!(got, vec![Msg::Many(vec![2])], "the row, and nothing is chosen");
    let t = ui.spot("tabs").unwrap().rect;
    let got = ui.input(&middle(t.x + t.w - 12.0, t.y + t.h / 2.0), now).messages;
    assert_eq!(got, vec![Msg::Many(vec![101])]);
    let p = ui.spot("plain").unwrap().rect;
    assert!(ui.input(&middle(p.x + 20.0, p.y + 5.0), now).messages.is_empty());
}

fn list_ui(chosen: Option<usize>) -> Ui<Msg> {
    let mut ui = Ui::new(scroll("area", list(rows(40), chosen, Msg::Pick).on_open(Msg::Open).key("list")),
                         Theme::new(false, "#fbc711"));
    ui.size = (400, 300);
    ui.settle();
    ui
}

#[test]
fn a_list_is_chosen_by_keys_and_opened_by_a_double_click() {
    let mut ui = list_ui(None);
    let now = Instant::now();
    ui.input(&key(Keysym::Tab, false), now);
    assert_eq!(ui.focus.as_deref(), Some("list"));
    assert_eq!(ui.input(&key(Keysym::Down, false), now).messages, vec![Msg::Pick(0)]);
    let mut ui = list_ui(Some(0));
    ui.focus = Some("list".into());
    assert_eq!(ui.input(&key(Keysym::Down, false), now).messages, vec![Msg::Pick(1)]);
    assert_eq!(ui.input(&key(Keysym::Return, false), now).messages, vec![Msg::Open(0)]);
    assert_eq!(ui.input(&key(Keysym::End, false), now).messages, vec![Msg::Pick(39)]);
    let s = ui.spot("area").unwrap();
    assert_eq!(ui.offset(s), (s.full - s.rect.h).round());
    let mut ui = list_ui(Some(39));
    ui.focus = Some("list".into());
    ui.scrolls.insert("area".into(), 10_000.0);
    ui.laid = None;
    ui.settle();
    let per = (ui.spot("area").unwrap().rect.h / ui.px(theme::sheet().row)).floor() as usize;
    assert_eq!(ui.input(&key(Keysym::Page_Up, false), now).messages, vec![Msg::Pick(39 - per)]);
    let mut ui = list_ui(None);
    let r = ui.spot("list").unwrap().rect;
    let (x, y) = (r.x + 20.0, r.y + ui.px(theme::sheet().row) * 2.5);
    ui.input(&Input::Press { button: Button::Left, x, y }, now);
    assert_eq!(ui.input(&Input::Release { button: Button::Left, x, y }, now).messages, vec![Msg::Pick(2)]);
    ui.input(&Input::Press { button: Button::Left, x, y }, now);
    let later = now + Duration::from_millis(200);
    assert_eq!(ui.input(&Input::Release { button: Button::Left, x, y }, later).messages, vec![Msg::Pick(2), Msg::Open(2)]);
    let mut ui = list_ui(Some(2));
    ui.input(&Input::Press { button: Button::Left, x, y }, now);
    ui.input(&Input::Release { button: Button::Left, x, y }, now);
    ui.input(&Input::Press { button: Button::Left, x, y }, now);
    assert!(ui.input(&Input::Release { button: Button::Left, x, y }, now + Duration::from_secs(1)).messages.is_empty());
}

fn menu_ui(chosen: Option<usize>, picked: Vec<usize>) -> Ui<Msg> {
    let names = ["Apples", "Bread", "Butter", "Cheese", "beans", "Carrots"];
    let rows = names.iter().map(|n| item(*n)).collect();
    let mut ui = Ui::new(scroll("area", list(rows, chosen, Msg::Pick).many(picked, Msg::Many)
                                .on_menu(|r, x, y| Msg::Menu(r, x as i32, y as i32)).type_ahead().key("list")),
                         Theme::new(false, "#fbc711"));
    ui.size = (400, 300);
    ui.settle();
    ui
}

#[test]
fn a_right_click_on_a_row_chooses_it_and_asks_for_its_menu() {
    let now = Instant::now();
    let mut ui = menu_ui(Some(0), vec![0]);
    let r = ui.spot("list").unwrap().rect;
    let (x, y) = (r.x + 20.0, r.y + ui.px(theme::sheet().row) * 2.5);
    let m = ui.input(&Input::Press { button: Button::Right, x, y }, now).messages;
    assert_eq!(m, vec![Msg::Pick(2), Msg::Many(vec![2]), Msg::Menu(2, x as i32, y as i32)]);
    assert_eq!(ui.focus.as_deref(), Some("list"));
    let mut ui = menu_ui(Some(1), vec![1, 2, 3]);
    let m = ui.input(&Input::Press { button: Button::Right, x, y }, now).messages;
    assert_eq!(m, vec![Msg::Menu(2, x as i32, y as i32)]);
    let below = r.y + ui.px(theme::sheet().row) * 20.0;
    assert!(ui.input(&Input::Press { button: Button::Right, x, y: below }, now).messages.is_empty());
    let mut ui = menu_ui(Some(1), vec![1]);
    ui.focus = Some("list".into());
    let m = ui.input(&key(Keysym::Menu, false), now).messages;
    assert!(matches!(m.as_slice(), [Msg::Menu(1, _, _)]), "{m:?}");
    let mut ui = list_ui(None);
    assert!(ui.input(&Input::Press { button: Button::Right, x, y }, now).messages.is_empty());
}

#[test]
fn a_long_hold_on_a_row_asks_for_its_menu() {
    let now = Instant::now();
    let mut ui = menu_ui(None, vec![]);
    let r = ui.spot("list").unwrap().rect;
    let (x, y) = (r.x + 20.0, r.y + ui.px(theme::sheet().row) * 1.5);
    let reply = ui.input(&Input::TouchDown { id: 1, x, y }, now);
    let n = reply.hold.expect("a hold is timed");
    let m = ui.held(n).messages;
    assert_eq!(m.last(), Some(&Msg::Menu(1, x as i32, y as i32)));
}

#[test]
fn typing_over_a_list_jumps_to_the_row() {
    let now = Instant::now();
    let mut ui = menu_ui(None, vec![]);
    ui.focus = Some("list".into());
    assert_eq!(ui.input(&typed("b"), now).messages, vec![Msg::Pick(1), Msg::Many(vec![1])]);
    let mut ui = menu_ui(Some(1), vec![1]);
    ui.focus = Some("list".into());
    ui.ahead = Some(("list".into(), "b".into(), Instant::now()));
    assert_eq!(ui.input(&typed("u"), now).messages, vec![Msg::Pick(2), Msg::Many(vec![2])]);
    let mut ui = menu_ui(Some(1), vec![1]);
    ui.focus = Some("list".into());
    ui.ahead = Some(("list".into(), "b".into(), Instant::now()));
    assert_eq!(ui.input(&typed("b"), now).messages, vec![Msg::Pick(2), Msg::Many(vec![2])]);
    let mut ui = menu_ui(Some(2), vec![2]);
    ui.focus = Some("list".into());
    ui.ahead = Some(("list".into(), "bu".into(), ago(Duration::from_secs(5))));
    assert_eq!(ui.input(&typed("b"), now).messages, vec![Msg::Pick(4), Msg::Many(vec![4])]);
    let mut ui = list_ui(None);
    ui.focus = Some("list".into());
    assert!(ui.input(&typed("r"), now).messages.is_empty());
}

#[test]
fn the_program_is_told_what_has_the_keyboard() {
    let now = Instant::now();
    let mut ui = Ui::new(column(vec![
        entry("", "Name", Msg::Name).key("name"),
        list(rows(3), None, Msg::Pick).key("list"),
        button("b", "B").on_press(Msg::Go).key("go"),
    ]), Theme::new(false, "#fbc711"));
    ui.size = (400, 300);
    ui.settle();
    assert_eq!(ui.keyboard(), Keyboard::default());
    ui.input(&key(Keysym::Tab, false), now);
    assert_eq!(ui.keyboard(), Keyboard { keys: true, text: true, arrows: true });
    ui.input(&key(Keysym::Tab, false), now);
    assert_eq!(ui.keyboard(), Keyboard { keys: true, text: false, arrows: true });
    ui.input(&key(Keysym::Tab, false), now);
    assert_eq!(ui.keyboard(), Keyboard { keys: true, text: false, arrows: false });
}

#[test]
fn where_you_are_is_a_lighter_box_with_no_dot() {
    let mut ui = Ui::new(list(vec![item("One"), item("Two")], Some(1), Msg::Pick).here().key("list"), Theme::new(false, "#fbc711"));
    ui.size = (300, 100);
    ui.settle();
    let mut pix = tiny_skia::Pixmap::new(300, 100).unwrap();
    let spot = ui.spot("list").unwrap().clone();
    let What::List { rows, chosen, .. } = &ui.node("list").unwrap().what else { panic!() };
    let mut p = crate::paint::Painter::default();
    ui.paint_places(&mut p, (300, 100), &spot, rows, *chosen);
    p.play(&mut pix.as_mut());
    let near = |p: &tiny_skia::PremultipliedColorU8, want: Colour| {
        let c = p.demultiply();
        (c.red() as i32 - (want.r * 255.0) as i32).abs() < 12 && (c.green() as i32 - (want.g * 255.0) as i32).abs() < 12
            && (c.blue() as i32 - (want.b * 255.0) as i32).abs() < 12 && c.alpha() > 200
    };
    let row = ui.px(theme::sheet().row) as u32;
    let (first, second): (Vec<_>, Vec<_>) = pix.pixels().iter().enumerate().partition(|(i, _)| (*i as u32 / 300) < row);
    assert!(second.iter().filter(|(_, p)| near(p, ui.theme.raised)).count() > 2000, "a box");
    assert!(!pix.pixels().iter().any(|p| near(p, ui.theme.accent)), "no dot");
    assert!(first.iter().filter(|(_, p)| near(p, ui.theme.raised)).count() < 10);
    assert!(!pix.pixels().iter().any(|p| near(p, ui.theme.picked)));
}

#[test]
fn the_main_action_is_a_pill_with_its_words() {
    let mut ui = Ui::new(row(vec![crate::widget::main_action("folder-download-symbolic", "Install").on_press(Msg::Go).key("go"),
                                  button("window-close-symbolic", "Cancel").key("no")]),
                         Theme::new(false, "#fbc711"));
    ui.size = (400, 100);
    ui.settle();
    let (pill, round) = (ui.spot("go").unwrap().rect, ui.spot("no").unwrap().rect);
    assert!(pill.w > 2.0 * pill.h && pill.h == ui.px(theme::sheet().action_height));
    assert_eq!(round.w, ui.px(theme::icon_button()));
    assert!(ui.node("go").unwrap().tip().is_none());
    assert!(ui.node("no").unwrap().tip().is_some());
    let mut pix = tiny_skia::Pixmap::new(400, 100).unwrap();
    ui.paint(&mut pix.as_mut(), 1.0);
    let accent = ui.theme.accent;
    let lit = pix.pixels().iter().filter(|p| {
        let c = p.demultiply();
        (c.red() as i32 - (accent.r * 255.0) as i32).abs() < 6 && (c.green() as i32 - (accent.g * 255.0) as i32).abs() < 6
    }).count();
    assert!(lit as f32 > pill.w * pill.h * 0.4, "filled in the accent");
}

fn many_ui(chosen: Option<usize>, picked: Vec<usize>) -> Ui<Msg> {
    let mut ui = Ui::new(scroll("area", list(rows(40), chosen, Msg::Pick).many(picked, Msg::Many).key("list")),
                         Theme::new(false, "#fbc711"));
    ui.size = (400, 300);
    ui.settle();
    ui.focus = Some("list".into());
    ui
}

#[test]
fn several_rows_are_picked_with_ctrl_shift_and_the_keys() {
    let now = Instant::now();
    let ctrl_down = Input::Key { sym: Keysym::Control_L, text: None, mods: Mods::default(), repeat: false };
    let ctrl_up = Input::KeyUp { sym: Keysym::Control_L, mods: Mods { ctrl: true, ..Mods::default() } };
    let shift_down = Input::Key { sym: Keysym::Shift_L, text: None, mods: Mods::default(), repeat: false };
    let at = |ui: &Ui<Msg>, i: f32| {
        let r = ui.spot("list").unwrap().rect;
        (r.x + 20.0, r.y + ui.px(theme::sheet().row) * (i + 0.5))
    };
    let click = |ui: &mut Ui<Msg>, (x, y): (f32, f32)| {
        ui.input(&Input::Press { button: Button::Left, x, y }, now);
        ui.input(&Input::Release { button: Button::Left, x, y }, now).messages
    };
    let mut ui = many_ui(Some(1), vec![1, 4]);
    let p = at(&ui, 2.0);
    assert_eq!(click(&mut ui, p), vec![Msg::Pick(2), Msg::Many(vec![2])]);
    let mut ui = many_ui(Some(1), vec![1]);
    ui.input(&ctrl_down, now);
    let p = at(&ui, 3.0);
    assert_eq!(click(&mut ui, p), vec![Msg::Pick(3), Msg::Many(vec![1, 3])]);
    let mut ui = many_ui(Some(3), vec![1, 3]);
    ui.input(&ctrl_down, now);
    let p = at(&ui, 1.0);
    assert_eq!(click(&mut ui, p), vec![Msg::Pick(1), Msg::Many(vec![3])]);
    ui.input(&ctrl_up, now);
    let p = at(&ui, 5.0);
    assert_eq!(click(&mut ui, p), vec![Msg::Pick(5), Msg::Many(vec![5])]);
    let mut ui = many_ui(Some(1), vec![1]);
    ui.input(&ctrl_down, now);
    ui.input(&Input::Mods(Mods::default()), now);
    let p = at(&ui, 4.0);
    assert_eq!(click(&mut ui, p), vec![Msg::Pick(4), Msg::Many(vec![4])]);
    let mut ui = many_ui(Some(2), vec![2]);
    ui.input(&shift_down, now);
    let p = at(&ui, 5.0);
    assert_eq!(click(&mut ui, p), vec![Msg::Pick(5), Msg::Many(vec![2, 3, 4, 5])]);
    let mut ui = many_ui(Some(2), vec![2]);
    assert_eq!(ui.input(&key(Keysym::Down, true), now).messages, vec![Msg::Pick(3), Msg::Many(vec![2, 3])]);
    let mut ui = many_ui(Some(3), vec![2, 3]);
    assert_eq!(ui.input(&key(Keysym::Down, false), now).messages, vec![Msg::Pick(4), Msg::Many(vec![4])]);
    let mut ui = many_ui(Some(0), vec![0]);
    assert_eq!(ui.input(&ctrl(Keysym::a), now).messages, vec![Msg::Many((0..40).collect())]);
    let mut ui = many_ui(Some(6), vec![0]);
    assert_eq!(ui.input(&key(Keysym::space, false), now).messages, vec![Msg::Many(vec![0, 6])]);
}
