//! The parts a program paints with the toolkit's help (a table, a tab bar, and the like) drawn in a real window, not just reasoned about.

use super::*;
use super::super::table::{paint_table, Column, Table};
use super::super::tabs::{paint_tabs, Tab, Tabs};
use super::super::theme::Theme;
use super::super::widget::{self as ww, Length};

#[derive(Clone, Debug, PartialEq)]
enum Msg {
    Other,
    Ticked(bool),
    Colour(super::super::picker::Edit),
    Day(super::super::when::Pick),
    Time(super::super::when::Typed),
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

fn colour_at(pix: &tiny_skia::Pixmap, x: u32, y: u32) -> [u8; 4] {
    let p = pix.pixel(x, y).unwrap();
    [p.red(), p.green(), p.blue(), p.alpha()]
}

fn table_of(chosen: Option<usize>) -> Table {
    let mut t = Table::new(vec![Column::new("Name", 200.0).sortable(), Column::new("Size", 100.0).end()]);
    t.set_rows(40);
    t.chosen = chosen;
    t.sort = Some((0, true));
    t
}

fn table_ui(table: Table) -> Ui<Msg> {
    let drawn = table.clone();
    ui_of(ww::drawing((380.0, 280.0), "Files", move |canvas, pen| {
        paint_table(canvas, pen, &drawn, None, |r, c| format!("row {r} col {c}"));
    }).width(Length::Fill).height(Length::Fill))
}

#[test]
fn a_table_paints_its_chosen_row_in_the_picked_colour_and_only_that_row() {
    let mut plain = table_ui(table_of(None));
    let mut chosen = table_ui(table_of(Some(2)));
    let (a, b) = (shot(&mut plain), shot(&mut chosen));
    assert_ne!(a.data(), b.data());
    let (inside, outside) = (colour_at(&b, 390, 32 + 72 + 18), colour_at(&b, 390, 32 + 36 + 18));
    assert_eq!(inside, colour_at(&b, 395, 32 + 72 + 5));
    assert_ne!(inside, outside);
    assert_eq!(outside, colour_at(&a, 390, 32 + 36 + 18));
}

#[test]
fn a_table_scrolled_down_paints_later_rows_and_the_heading_stays() {
    let mut top = table_ui(table_of(None));
    let mut scrolled = {
        let mut t = table_of(None);
        t.scroll(10.0);
        table_ui(t)
    };
    let (a, b) = (shot(&mut top), shot(&mut scrolled));
    assert_ne!(a.data(), b.data());
    let band = |p: &tiny_skia::Pixmap| (0..400).map(|x| colour_at(p, x, 16)).collect::<Vec<_>>();
    assert_eq!(band(&a), band(&b));
}

#[test]
fn a_tab_bar_paints_the_chosen_tab_apart_from_the_others() {
    let bar = |chosen: usize| {
        let mut tabs = Tabs::new(vec![Tab::new("One"), Tab::new("Two"), Tab::new("Three")]);
        tabs.chosen = chosen;
        ui_of(ww::drawing((380.0, 36.0), "Tabs", move |canvas, pen| paint_tabs(canvas, pen, &tabs, None, None)))
    };
    let (a, b) = (shot(&mut bar(0)), shot(&mut bar(2)));
    assert_ne!(a.data(), b.data());
    assert_ne!(colour_at(&a, 20, 28), colour_at(&b, 20, 28));
}

#[test]
fn a_text_area_paints_its_words_and_tells_a_reader_it_is_a_text_box() {
    use super::super::editing::{area::TextArea, editor::Look};
    let look = Look { font: None, size: 14.0, wrap: true, numbers: false };
    let shown = |text: &str| {
        let area = TextArea::new(text, look.clone());
        let mut ui = ui_of(area.node("Notes", 200.0, |_| Msg::Other));
        (shot(&mut ui), ui.access("Test"))
    };
    let (empty, _) = shown("");
    let (full, told) = shown("Some words\nand a second line");
    assert_ne!(empty.data(), full.data());
    let role = told.nodes.iter().find(|(_, n)| n.role() == accesskit::Role::MultilineTextInput);
    assert!(role.is_some());
}

#[test]
fn a_cell_grid_paints_fills_the_cursor_and_the_selection() {
    use super::super::cellgrid::{cell_metrics, paint_cells, Caret, Cell, Screen, Selection, Shape};
    let screen = {
        let mut s = Screen::new(10, 3);
        s.put(0, 0, "hello 日本", &Cell { bg: Some(crate::colour::Colour::rgb(0.0, 0.0, 1.0)), ..Cell::default() });
        s
    };
    let ui_with = |caret: Option<Caret>, sel: Option<Selection>| {
        let s = screen.clone();
        shot(&mut ui_of(ww::drawing((380.0, 280.0), "Grid", move |canvas, pen| paint_cells(canvas, pen, &s, 14.0, caret, sel))))
    };
    let plain = ui_with(None, None);
    let (w, h) = {
        let fonts = crate::text::Fonts::new();
        let pen = ww::Pen { theme: &Theme::new(true, "#fbc711"), fonts: &fonts, icons: &crate::icon::Icons::new(), scale: 1.0 };
        cell_metrics(&pen, 14.0).1
    };
    assert_eq!(colour_at(&plain, (w * 0.5) as u32, (h * 0.1) as u32)[2], 255);
    assert_ne!(colour_at(&plain, (w * 0.5) as u32, (h * 1.5) as u32), colour_at(&plain, (w * 0.5) as u32, (h * 0.1) as u32));
    let with_cursor = ui_with(Some(Caret { col: 4, row: 2, shape: Shape::Block, focused: true }), None);
    assert_ne!(plain.data(), with_cursor.data());
    let picked = ui_with(None, Some(Selection { from: (0, 1), to: (3, 1), block: false }));
    assert_ne!(plain.data(), picked.data());
}

#[test]
fn an_avatar_is_round_an_icon_has_ink_and_an_image_keeps_its_proportions() {
    use std::rc::Rc;
    let red = Rc::new(image::DynamicImage::ImageRgba8(image::RgbaImage::from_pixel(40, 20, image::Rgba([255, 0, 0, 255]))));
    let mut ui = ui_of(ww::column(vec![
        ww::avatar(Some(red.clone()), "Ann Lee", 60.0).key("pic"),
        ww::avatar(None, "Ann Lee", 60.0).key("none"),
        ww::image_view(red, "A red bar", (100.0, 100.0)).key("bar"),
        ww::icon_image("edit-copy-symbolic", 24.0, ww::Tone::Plain).key("icon"),
    ]).gap(10.0));
    let pix = shot(&mut ui);
    let at = |id: &str| ui.spot(id).map(|s| s.rect).unwrap();
    let (pic, none, bar) = (at("pic"), at("none"), at("bar"));
    let mid = |r: ww::Rect, dx: f32, dy: f32| colour_at(&pix, (r.x + r.w / 2.0 + dx) as u32, (r.y + r.h / 2.0 + dy) as u32);
    assert_eq!(mid(pic, 0.0, 0.0)[0], 255);
    assert_eq!(colour_at(&pix, pic.x as u32 + 1, pic.y as u32 + 1), colour_at(&pix, 390, 2));
    let ground = colour_at(&pix, none.x as u32 + 1, none.y as u32 + 1);
    let inked = (0..60).flat_map(|dy| (0..60).map(move |dx| (dx, dy))).filter(|&(dx, dy)| colour_at(&pix, none.x as u32 + dx, none.y as u32 + dy) != ground).count();
    assert!(inked > 20, "the initials show");
    assert_eq!(mid(bar, 0.0, 0.0)[0], 255);
    assert_ne!(mid(bar, 0.0, 40.0)[0], 255);
    let names: Vec<_> = ui.access("T").nodes.iter().filter_map(|(_, n)| n.label().map(String::from)).collect();
    assert!(names.contains(&"Ann Lee".to_string()) && names.contains(&"A red bar".to_string()));
}

#[test]
fn a_colour_square_paints_the_hue_and_a_press_on_it_picks_saturation_and_brightness() {
    use super::super::colour::Colour;
    use super::super::picker::{ColourBox, Edit};
    let view = |hex: &str| ColourBox::new(Colour::parse(hex).unwrap()).view(Msg::Colour);
    let (mut red, mut blue) = (ui_of(view("#ff0000")), ui_of(view("#0000ff")));
    let (a, b) = (shot(&mut red), shot(&mut blue));
    let square = ww::Rect { x: 0.0, y: 0.0, w: 220.0, h: 150.0 };
    let at = |p: &tiny_skia::Pixmap, dx: f32, dy: f32| colour_at(p, (square.x + dx) as u32, (square.y + dy) as u32);
    let (top_right, top_left, bottom) = (at(&a, 205.0, 10.0), at(&a, 10.0, 10.0), at(&a, 100.0, 140.0));
    assert!(top_right[0] > 200 && top_right[1] < 40 && top_right[2] < 40, "{top_right:?}");
    assert!(at(&b, 205.0, 10.0)[2] > 200 && at(&b, 205.0, 10.0)[0] < 40);
    assert!(top_left.iter().take(3).all(|&c| c > 200));
    assert!(bottom.iter().take(3).all(|&c| c < 30));
    let sent = red.input(&Input::Press { button: Button::Left, x: square.x + 110.0, y: square.y + 75.0 }, Instant::now()).messages;
    assert!(matches!(sent.as_slice(), [Msg::Colour(Edit::Square(s, v))] if (s - 0.5).abs() < 0.02 && (v - 0.5).abs() < 0.02), "{sent:?}");
}

#[test]
fn a_date_grid_starts_on_monday_chooses_a_day_and_names_it_in_full() {
    use super::super::when::{Date, DateBox};
    let chosen = Date::new(2026, 10, 25).unwrap();
    let mut ui = ui_of(DateBox::new(Some(chosen), chosen).view(Msg::Day));
    let told = ui.access("Test");
    let names: Vec<String> = told.nodes.iter().filter_map(|(_, n)| n.label().map(String::from)).collect();
    assert!(names.contains(&"Sunday 25 October 2026".to_string()), "{names:?}");
    assert!(names.contains(&"Thursday 1 October 2026".to_string()));
    let texts: Vec<String> = told.nodes.iter().filter_map(|(_, n)| n.value().map(String::from).or_else(|| n.label().map(String::from))).collect();
    assert!(texts.contains(&"October 2026".to_string()) && texts.contains(&"Mon".to_string()), "{texts:?}");
    assert!(names.contains(&"Saturday 31 October 2026".to_string()), "{names:?}");
    shot(&mut ui);
}

#[test]
fn a_time_box_and_a_settings_page_build_and_tell_a_reader_their_names() {
    use super::super::settings::{setting, setting_page};
    use super::super::when::{Time, TimeBox};
    let page = setting_page("page", "Date and time", "Set the clock.", vec![
        ("Clock".into(), vec![setting("Time", "The time of day.", TimeBox::new(Time::new(7, 5).unwrap()).view(Msg::Time)),
                              setting("Quiet", "", ww::switch(true, "Quiet", |_| Msg::Other))]),
        ("Empty".into(), vec![]),
    ]);
    let mut ui = ui_of(page);
    let _ = shot(&mut ui);
    let names: Vec<String> = ui.access("Test").nodes.iter().filter_map(|(_, n)| n.label().map(String::from)).collect();
    for want in ["Hours", "Minutes", "Quiet"] {
        assert!(names.contains(&want.to_string()), "{want} in {names:?}");
    }
}

fn first_spot(ui: &Ui<Msg>) -> Rect {
    ui.spots.iter().find(|s| matches!(ui.tree.at(&s.path).map(|n| &n.what), Some(What::Button { .. } | What::Card(..)))).unwrap().rect
}

fn dark(tree: Node<Msg>) -> Ui<Msg> {
    let mut ui = Ui::new(tree, Theme::chosen());
    ui.size = (400, 300);
    ui.settle();
    ui
}

#[test]
fn an_icon_button_has_no_disc_and_a_card_no_edge() {
    let mut ui = dark(ww::column(vec![ww::button("list-add-symbolic", "Add").on_press(Msg::Other)]).padding(24.0));
    let r = first_spot(&ui);
    let pix = shot(&mut ui);
    assert_eq!(colour_at(&pix, (r.x + r.w / 2.0 - 13.0) as u32, (r.y + r.h / 2.0) as u32), colour_at(&pix, 4, 4));
    let mut ui = dark(ww::column(vec![ww::card(ww::label("Words".to_string()))]).padding(24.0));
    let r = first_spot(&ui);
    let pix = shot(&mut ui);
    let x = (r.x + r.w / 2.0) as u32;
    assert_eq!(colour_at(&pix, x, r.y as u32), colour_at(&pix, x, r.y as u32 + 4));
}

#[test]
fn a_quiet_action_is_a_box_not_the_highlight_pill() {
    let mut ui = ui_of(ww::column(vec![ww::quiet_action("list-add-symbolic", "New").on_press(Msg::Other)]).padding(24.0));
    let r = first_spot(&ui);
    let pix = shot(&mut ui);
    let accent = Theme::new(true, "#fbc711").accent.to_skia().to_color_u8();
    assert_ne!(colour_at(&pix, (r.x + 4.0) as u32, (r.y + r.h / 2.0) as u32)[..3], [accent.red(), accent.green(), accent.blue()]);
    assert!(r.w > 0.0 && r.h > 0.0);
}

#[test]
fn a_tab_bar_has_no_accent_underline() {
    let mut tabs = Tabs::new(vec![Tab::new("One"), Tab::new("Two")]);
    tabs.chosen = 0;
    tabs.height = 36.0;
    let pix = shot(&mut dark(ww::drawing((380.0, 36.0), "Tabs", move |canvas, pen| paint_tabs(canvas, pen, &tabs, None, None))));
    assert!(!pix.data().chunks_exact(4).any(|c| c[..3] == [251, 199, 17]));
}

#[test]
fn a_tick_box_is_pressed_or_spaced_and_read_as_a_check_box() {
    let tree = |on| ww::column(vec![ww::tick(on, "Remember me", Msg::Ticked).key("tick")]).padding(10.0);
    let mut ui = ui_of(tree(false));
    let now = Instant::now();
    let r = ui.spot("tick").unwrap().rect;
    assert_eq!((r.w, r.h), (18.0, 18.0));
    let (x, y) = (r.x + r.w / 2.0, r.y + r.h / 2.0);
    ui.input(&Input::PointerMove { x, y }, now);
    ui.input(&Input::Press { button: Button::Left, x, y }, now);
    assert_eq!(ui.input(&Input::Release { button: Button::Left, x, y }, now).messages, vec![Msg::Ticked(true)]);
    let mut on = ui_of(tree(true));
    on.input(&Input::Key { sym: Keysym::Tab, text: None, mods: Mods::default(), repeat: false }, now);
    assert_eq!(on.input(&Input::Key { sym: Keysym::space, text: None, mods: Mods::default(), repeat: false }, now).messages, vec![Msg::Ticked(false)]);
    let (a, b) = (shot(&mut ui_of(tree(false))), shot(&mut ui_of(tree(true))));
    assert_ne!(a.data(), b.data());
    let (cx, cy) = (x as u32, y as u32);
    assert_ne!(colour_at(&b, cx, cy), colour_at(&b, r.x as u32 + 3, r.y as u32 + 3));
    let told = on.access("Test");
    let part = told.nodes.iter().find(|(_, n)| n.role() == accesskit::Role::CheckBox).map(|(_, n)| n).unwrap();
    assert_eq!((part.label(), part.toggled()), (Some("Remember me"), Some(accesskit::Toggled::True)));
}

#[test]
#[ignore]
fn measure_long_lists() {
    for n in [1_000usize, 10_000, 100_000] {
        let began = std::time::Instant::now();
        let mut ui = ui_of(ww::scroll("rows", ww::list((0..n).map(|i| ww::item(format!("row {i}"))).collect(), None, |_| Msg::Other)));
        shot(&mut ui);
        let list = began.elapsed();
        let mut t = table_of(None);
        t.set_rows(n * 10);
        let began = std::time::Instant::now();
        let mut ui = table_ui(t);
        shot(&mut ui);
        println!("{n} rows: list node {list:?}; {} rows in a painted table {:?}", n * 10, began.elapsed());
    }
}
