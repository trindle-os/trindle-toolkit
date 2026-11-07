//! Tests of scrolling areas, down and sideways, by wheel, thumb, finger and program.

use super::*;

fn long_ui() -> Ui<Msg> {
    let buttons = (0..30).map(|i| button("x", format!("Button {i}")).on_press(Msg::Save).key(format!("b{i}"))).collect();
    let mut ui = Ui::new(scroll("area", column(buttons)), Theme::new(false, "#fbc711"));
    ui.size = (400, 300);
    ui.settle();
    ui
}

#[test]
fn a_card_of_set_width_is_as_tall_as_its_wrapped_words() {
    let words = "Words long enough to wrap onto several lines in a narrow card, but onto one in a wide window.";
    let card_of = || card(column(vec![label(words), button("x", "Go").on_press(Msg::Go)])).width(crate::widget::Length::Fixed(200.0));
    let ui = Ui::new(card_of(), Theme::new(false, "#fbc711"));
    let narrow = ui.natural_height(200.0);
    let wide = Ui::new(row(vec![card_of()]), Theme::new(false, "#fbc711")).natural_height(1200.0);
    assert_eq!(wide, narrow);
    assert!(narrow > Ui::<Msg>::new(column(vec![label(words)]), Theme::new(false, "#fbc711")).natural_height(1200.0) * 2.0);
}

#[test]
fn a_part_the_program_seeks_is_scrolled_into_sight_once_per_turn() {
    let tall = |turn: u32, row: f32| scroll("area", column(vec![drawing((0.0, 600.0), "rows", |_, _| {}).seek(turn, row * 20.0, 20.0)]));
    let mut ui = Ui::new(tall(0, 0.0), Theme::new(false, "#fbc711"));
    ui.size = (400, 300);
    ui.settle();
    let at = |ui: &mut Ui<Msg>| {
        ui.settle();
        let s = ui.spot("area").unwrap().clone();
        ui.offset(&s)
    };
    assert_eq!(at(&mut ui), 0.0);
    ui.set_view(tall(1, 20.0));
    assert_eq!(at(&mut ui), 420.0 - 300.0);
    ui.scroll_area("area", 0.0);
    ui.set_view(tall(1, 20.0));
    assert_eq!(at(&mut ui), 0.0);
    ui.set_view(tall(2, 3.0));
    assert_eq!(at(&mut ui), 0.0);
    ui.set_view(tall(3, 29.0));
    assert_eq!(at(&mut ui), 300.0);
}

#[test]
fn a_part_sought_in_a_sideways_area_scrolls_along_not_down() {
    let wide = |turn: u32, col: f32| crate::widget::scroll_x("wide", row(vec![drawing((1200.0, 0.0), "cols", |_, _| {}).seek(turn, col * 100.0, 100.0)]));
    let mut ui = Ui::new(wide(0, 0.0), Theme::new(false, "#fbc711"));
    ui.size = (400, 300);
    ui.settle();
    let at = |ui: &mut Ui<Msg>| {
        ui.settle();
        let s = ui.spot("wide").unwrap().clone();
        ui.offset(&s)
    };
    assert_eq!(at(&mut ui), 0.0);
    ui.set_view(wide(1, 8.0));
    assert_eq!(at(&mut ui), 900.0 - 400.0);
    ui.set_view(wide(2, 6.0));
    assert_eq!(at(&mut ui), 500.0);
    ui.set_view(wide(3, 0.0));
    assert_eq!(at(&mut ui), 0.0);
}

#[test]
fn the_tab_key_brings_a_hidden_widget_into_sight_along_a_sideways_area() {
    let cols = (0..12).map(|i| button("x", format!("Column {i}")).on_press(Msg::Save).key(format!("c{i}")).width(crate::widget::Length::Fixed(120.0))).collect();
    let mut ui = Ui::new(crate::widget::scroll_x("wide", row(cols)), Theme::new(false, "#fbc711"));
    ui.size = (400, 300);
    ui.settle();
    let now = Instant::now();
    for _ in 0..12 {
        ui.input(&key(Keysym::Tab, false), now);
    }
    assert_eq!(ui.focus.as_deref(), Some("c11"));
    let (a, b) = (ui.spot("wide").unwrap().rect, ui.spot("c11").unwrap().rect);
    assert!(b.x >= a.x && b.x + b.w <= a.x + a.w, "{b:?} in {a:?}");
    assert!(ui.offset(ui.spot("wide").unwrap()) > 0.0);
}

#[test]
fn a_nudge_moves_the_area_on_by_so_much_once_per_turn() {
    let tall = |turn: u32, by: f32| scroll("area", column(vec![drawing((0.0, 600.0), "rows", |_, _| {}).nudge(turn, by)]));
    let mut ui = Ui::new(tall(0, 0.0), Theme::new(false, "#fbc711"));
    ui.size = (400, 300);
    ui.settle();
    let at = |ui: &mut Ui<Msg>| {
        ui.settle();
        let s = ui.spot("area").unwrap().clone();
        ui.offset(&s)
    };
    ui.scroll_area("area", 100.0);
    ui.set_view(tall(1, 40.0));
    assert_eq!(at(&mut ui), 140.0);
    ui.set_view(tall(1, 40.0));
    assert_eq!(at(&mut ui), 140.0);
    ui.set_view(tall(2, -60.0));
    assert_eq!(at(&mut ui), 80.0);
    ui.set_view(tall(3, -500.0));
    assert_eq!(at(&mut ui), 0.0);
    ui.set_view(tall(4, 5000.0));
    assert_eq!(at(&mut ui), 300.0);
    let two = |turn: u32| row(vec![
        scroll("left", column(vec![drawing((0.0, 600.0), "rows", |_, _| {}).nudge(turn, 30.0)])),
        scroll("right", column(vec![drawing((0.0, 600.0), "rows", |_, _| {}).nudge(turn, 50.0)])),
    ]);
    let mut ui: Ui<Msg> = Ui::new(two(0), Theme::new(false, "#fbc711"));
    ui.size = (400, 300);
    ui.settle();
    ui.set_view(two(7));
    ui.settle();
    let offs: Vec<f32> = ["left", "right"].iter().map(|id| ui.offset(&ui.spot(id).unwrap().clone())).collect();
    assert_eq!(offs, [30.0, 50.0]);
}

#[test]
fn a_widget_is_aimed_at_the_top_unless_it_shows_whole() {
    let mut ui = long_ui();
    let (area, now, to) = ui.aim_at("b1", 8.0).unwrap();
    assert_eq!((area.as_str(), now, to), ("area", 0.0, 0.0));
    let (_, _, to) = ui.aim_at("b20", 8.0).unwrap();
    assert!(to > 0.0);
    assert!(ui.scroll_area("area", to));
    let (a, b) = (ui.spot("area").unwrap().rect, ui.spot("b20").unwrap().rect);
    assert!((b.y - (a.y + ui.px(8.0))).abs() <= 1.0, "{} {}", b.y, a.y);
    let (_, _, to) = ui.aim_at("b29", 8.0).unwrap();
    let s = ui.spot("area").unwrap();
    assert!(to <= s.full - s.rect.h);
    assert!(ui.aim_at("nothing", 8.0).is_none());
}

#[test]
fn the_wheel_scrolls_and_stops_at_the_ends() {
    let mut ui = long_ui();
    let now = Instant::now();
    let s = ui.spot("area").unwrap().clone();
    assert!(s.full > s.rect.h);
    assert!(!ui.input(&wheel(1.0), now).redraw);
    ui.input(&Input::PointerMove { x: 100.0, y: 100.0 }, now);
    assert!(ui.input(&wheel(1.0), now).redraw);
    let off = ui.offset(ui.spot("area").unwrap());
    assert!(off > 0.0);
    let b0 = ui.spot("b0").unwrap().rect;
    assert_eq!(b0.y, s.rect.y - off);
    ui.input(&wheel(-5.0), now);
    assert_eq!(ui.offset(ui.spot("area").unwrap()), 0.0);
    assert!(!ui.input(&wheel(-1.0), now).redraw);
    ui.input(&wheel(1000.0), now);
    let s = ui.spot("area").unwrap();
    assert_eq!(ui.offset(s), (s.full - s.rect.h).round());
    ui.input(&key(Keysym::Home, false), now);
    assert_eq!(ui.offset(ui.spot("area").unwrap()), 0.0);
}

#[test]
fn the_thumb_drags_and_a_click_beside_it_pages() {
    let mut ui = long_ui();
    let now = Instant::now();
    let s = ui.spot("area").unwrap().clone();
    let (track, thumb) = ui.bar(&s, ui.px(theme::sheet().scrollbar_hover)).unwrap();
    let x = track.x + track.w / 2.0;
    ui.input(&Input::PointerMove { x, y: thumb.y + 2.0 }, now);
    assert_eq!(ui.bar_hover.as_deref(), Some("area"));
    assert_eq!(ui.hover, None);
    ui.input(&Input::Press { button: Button::Left, x, y: thumb.y + 2.0 }, now);
    ui.input(&Input::PointerMove { x, y: track.y + track.h }, now);
    let s = ui.spot("area").unwrap().clone();
    assert_eq!(ui.offset(&s), (s.full - s.rect.h).round());
    ui.input(&Input::Release { button: Button::Left, x, y: track.y + track.h }, now);
    assert!(ui.thumb.is_none());
    let before = ui.offset(&s);
    ui.input(&Input::Press { button: Button::Left, x, y: track.y + 1.0 }, now);
    ui.input(&Input::Release { button: Button::Left, x, y: track.y + 1.0 }, now);
    let after = ui.offset(ui.spot("area").unwrap());
    assert_eq!(before - after, ui.page(s.rect.h).round());
}

fn wide_ui() -> Ui<Msg> {
    let cols = (0..12).map(|i| button("x", format!("Column {i}")).on_press(Msg::Save).key(format!("c{i}")).width(crate::widget::Length::Fixed(120.0))).collect();
    let mut ui = Ui::new(crate::widget::scroll_x("wide", row(cols)), Theme::new(false, "#fbc711"));
    ui.size = (400, 300);
    ui.settle();
    ui
}

#[test]
fn a_sideways_area_scrolls_by_shift_wheel_and_sideways_wheel() {
    let mut ui = wide_ui();
    let now = Instant::now();
    let s = ui.spot("wide").unwrap().clone();
    assert!(s.full > s.rect.w + 100.0);
    let c0 = ui.spot("c0").unwrap().rect;
    assert!(c0.h <= s.rect.h - ui.px(theme::sheet().scrollbar_hover), "the bar's strip is kept clear");
    ui.input(&Input::PointerMove { x: 100.0, y: 100.0 }, now);
    assert!(ui.input(&wheel(1.0), now).redraw);
    let a = ui.offset(ui.spot("wide").unwrap());
    assert!(a > 0.0);
    assert_eq!(ui.spot("c0").unwrap().rect.x, s.rect.x - a);
    ui.input(&Input::Mods(Mods { shift: true, ..Mods::default() }), now);
    ui.input(&wheel(1.0), now);
    assert!(ui.offset(ui.spot("wide").unwrap()) > a);
    ui.input(&Input::Mods(Mods::default()), now);
    ui.input(&Input::Scroll { x: -1000.0, y: 0.0, steps: true }, now);
    assert_eq!(ui.offset(ui.spot("wide").unwrap()), 0.0);
    ui.input(&Input::Scroll { x: 1000.0, y: 0.0, steps: true }, now);
    let s = ui.spot("wide").unwrap();
    assert_eq!(ui.offset(s), (s.full - s.rect.w).round());
}

#[test]
fn a_sideways_bar_drags_pages_and_looks_like_the_other() {
    let mut ui = wide_ui();
    let now = Instant::now();
    let s = ui.spot("wide").unwrap().clone();
    let thick = ui.px(theme::sheet().scrollbar_hover);
    let (track, thumb) = ui.bar(&s, thick).unwrap();
    assert_eq!((track.h, track.y + track.h + ui.px(theme::sheet().scrollbar_inset)), (thick, s.rect.y + s.rect.h));
    let y = track.y + track.h / 2.0;
    ui.input(&Input::PointerMove { x: thumb.x + 2.0, y }, now);
    assert_eq!(ui.bar_hover.as_deref(), Some("wide"));
    assert_eq!(ui.hover, None);
    ui.input(&Input::Press { button: Button::Left, x: thumb.x + 2.0, y }, now);
    ui.input(&Input::PointerMove { x: track.x + track.w, y }, now);
    let s = ui.spot("wide").unwrap().clone();
    assert_eq!(ui.offset(&s), (s.full - s.rect.w).round());
    ui.input(&Input::Release { button: Button::Left, x: track.x + track.w, y }, now);
    assert!(ui.thumb.is_none());
    let before = ui.offset(&s);
    ui.input(&Input::Press { button: Button::Left, x: track.x + 1.0, y }, now);
    ui.input(&Input::Release { button: Button::Left, x: track.x + 1.0, y }, now);
    assert_eq!(before - ui.offset(ui.spot("wide").unwrap()), ui.page(s.rect.w).round());
}

#[test]
fn a_sideways_area_holds_half_a_million_points_of_fixed_columns() {
    let cols: Vec<Node<Msg>> = (0..2000).map(|i| drawing((250.0, 20.0), "col", |_, _| {}).key(format!("c{i}"))).collect();
    let mut ui = Ui::new(crate::widget::scroll_x("wide", row(cols).gap(0.0)), Theme::new(false, "#fbc711"));
    ui.size = (400, 300);
    ui.settle();
    let s = ui.spot("wide").unwrap().clone();
    assert!(s.full >= 500_000.0, "{}", s.full);
    ui.scroll_area("wide", s.full);
    let s = ui.spot("wide").unwrap().clone();
    assert_eq!(ui.offset(&s), s.full - s.rect.w);
    let last = ui.spot("c1999").unwrap().rect;
    assert!(last.x + last.w <= s.rect.x + s.rect.w + 0.5 && last.x >= s.rect.x, "{last:?}");
}

#[test]
fn a_sideways_area_that_fits_has_no_bar() {
    let mut ui = Ui::new(crate::widget::scroll_x("wide", row(vec![button("x", "One").on_press(Msg::Save)])), Theme::new(false, "#fbc711"));
    ui.size = (400, 300);
    ui.settle();
    let s = ui.spot("wide").unwrap().clone();
    assert!(ui.bar(&s, 8.0).is_none());
    ui.input(&Input::PointerMove { x: 100.0, y: 100.0 }, Instant::now());
    assert!(!ui.input(&Input::Scroll { x: 5.0, y: 0.0, steps: true }, Instant::now()).redraw);
}

#[test]
fn a_finger_drags_the_area_without_pressing() {
    let mut ui = long_ui();
    let now = Instant::now();
    let b = ui.spot("b2").unwrap().rect;
    let (x, y) = (b.x + b.w / 2.0, b.y + b.h / 2.0);
    ui.input(&Input::TouchDown { id: 1, x, y }, now);
    assert_eq!(ui.pressed.as_deref(), Some("b2"));
    ui.input(&Input::TouchMove { id: 1, x, y: y - 60.0 }, now);
    assert_eq!(ui.offset(ui.spot("area").unwrap()), 60.0);
    assert!(ui.pressed.is_none());
    assert!(ui.input(&Input::TouchUp { id: 1 }, now).messages.is_empty());
    let b = ui.spot("b2").unwrap().rect;
    let (x, y) = (b.x + b.w / 2.0, b.y + b.h / 2.0);
    ui.input(&Input::TouchDown { id: 2, x, y }, now);
    ui.input(&Input::TouchMove { id: 2, x, y: y + 3.0 }, now);
    assert_eq!(ui.input(&Input::TouchUp { id: 2 }, now).messages, vec![Msg::Save]);
}

#[test]
fn a_finger_taken_for_a_drag_out_leaves_the_next_touch_free() {
    let mut ui = long_ui();
    let now = Instant::now();
    let b = ui.spot("b2").unwrap().rect;
    let (x, y) = (b.x + b.w / 2.0, b.y + b.h / 2.0);
    ui.input(&Input::TouchDown { id: 1, x, y }, now);
    assert!(ui.touch.is_some());
    ui.input(&Input::DragEnded { took: None, short: None }, now);
    assert!(ui.touch.is_none());
    ui.input(&Input::TouchDown { id: 2, x, y }, now);
    assert_eq!(ui.input(&Input::TouchUp { id: 2 }, now).messages, vec![Msg::Save]);
}

#[test]
fn tab_brings_a_hidden_widget_into_view() {
    let mut ui = long_ui();
    let now = Instant::now();
    for _ in 0..12 {
        ui.input(&key(Keysym::Tab, false), now);
    }
    assert_eq!(ui.focus.as_deref(), Some("b11"));
    let (a, b) = (ui.spot("area").unwrap().rect, ui.spot("b11").unwrap().rect);
    assert!(b.y >= a.y && b.y + b.h <= a.y + a.h);
    ui.input(&key(Keysym::Tab, false), now);
    for _ in 0..12 {
        ui.input(&key(Keysym::ISO_Left_Tab, true), now);
    }
    assert_eq!(ui.focus.as_deref(), Some("b0"));
    assert_eq!(ui.offset(ui.spot("area").unwrap()), 0.0);
}

#[derive(Default)]
struct Picker {
    chosen: Option<usize>,
    opened: Option<usize>,
}

impl App for Picker {
    type Msg = Msg;
    fn update(&mut self, msg: Msg) {
        match msg {
            Msg::Pick(i) => self.chosen = Some(i),
            Msg::Open(i) => self.opened = Some(i),
            _ => {}
        }
    }
    fn view(&self) -> Node<Msg> {
        let rows = (0..80).map(|i| item(format!("Row {i}, with words long enough to be cut short at the end of the row")))
            .collect();
        scroll("area", list(rows, self.chosen, Msg::Pick).on_open(Msg::Open).key("list"))
    }
}

fn area_place(shown: &Shown<Picker>) -> (f32, f32, f32) {
    let held = shown.held.borrow();
    let s = held.ui.spot("area").unwrap();
    (held.ui.offset(s), s.full, s.rect.h)
}

fn paint_to_a_file(shown: &Shown<Picker>) {
    let (w, h) = (1200, 800);
    let mut pixmap = tiny_skia::Pixmap::new(w, h).unwrap();
    shown.held.borrow_mut().ui.paint(&mut pixmap.as_mut(), 1.0);
    let file = std::env::temp_dir().join(format!("trindle-list-{w}x{h}.rgba"));
    std::fs::write(&file, pixmap.data()).unwrap();
    eprintln!("painted to {}", file.display());
}

#[test]
#[ignore]
fn a_long_list_scrolls_on_a_real_compositor() {
    use crate::window::tests::Hands;
    use wayland_client::protocol::wl_pointer;
    let events = Loop::new();
    let display = Display::connect(&events, || panic!("compositor gone")).expect("connected");
    let shown = Shown::open(&events, &display, Kind::Window { title: "List".into(), app_id: "org.trindle.Test".into() },
                            (1200, 800), Picker::default()).expect("a window");
    let Hands { conn, keys, mouse, queue: _queue } = Hands::lend();
    let keys = keys.expect("a stand-in keyboard");
    let has_mouse = mouse.is_some();
    let wheeled = Rc::new(Cell::new(0.0f32));
    let w = wheeled.clone();
    let held = shown.held.clone();
    let at = Cell::new(0u32);
    let steps = events.clone();
    events.every(Duration::from_millis(250), move || {
        let i = at.get();
        at.set(i + 1);
        let ms = i * 250;
        if let Some(m) = &mouse {
            match i {
                3 => { m.motion_absolute(ms, 1, 1, 2, 2); m.frame(); }
                4 => {
                    m.axis_source(wl_pointer::AxisSource::Wheel);
                    m.axis_discrete(ms, wl_pointer::Axis::VerticalScroll, 45.0, 3);
                    m.frame();
                }
                6 => {
                    let h = held.borrow();
                    w.set(h.ui.spot("area").map(|s| h.ui.offset(s)).unwrap_or(0.0));
                }
                _ => {}
            }
        }
        if let Some(&key) = [15, 107, 28].get((i as usize).wrapping_sub(7)) {
            keys.key(ms, key, 1);
            keys.key(ms + 10, key, 0);
        }
        let _ = conn.flush();
        if i == 12 {
            steps.quit();
            return crate::event::Flow::Stop;
        }
        crate::event::Flow::Continue
    });
    events.run();
    let (chosen, opened) = shown.with(|p| (p.chosen, p.opened));
    let (off, full, h) = area_place(&shown);
    eprintln!("wheeled to {}, chosen {chosen:?}, opened {opened:?}, offset {off} of {full} in {h}", wheeled.get());
    if has_mouse {
        assert!(wheeled.get() > 0.0, "the wheel scrolled the list");
    }
    assert_eq!((chosen, opened), (Some(79), Some(79)));
    assert_eq!(off, (full - h).round());
    paint_to_a_file(&shown);
}
