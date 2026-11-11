//! Tests of what the widgets paint: moving things, glows, rings, cards and rows.

use super::*;

#[test]
fn what_moves_by_itself_is_painted_alone_and_matches_the_whole() {
    use crate::widget::{spinner, Length};
    let red = |c: &mut PixmapMut, _: &w::Pen| c.fill(tiny_skia::Color::from_rgba8(200, 40, 40, 255));
    let rows = (0..30).map(|i| row(vec![spinner(), label(format!("Row {i}")), drawing((16.0, 16.0), "", red).alive()])).collect();
    let view: Node<Msg> = column(vec![
        label("Above"),
        card(row(vec![spinner(), label("Looking"), drawing((16.0, 16.0), "", red).alive(), drawing((16.0, 16.0), "", red)])),
        scroll("area", column(rows)).height(Length::Fixed(90.0)),
    ]);
    let mut ui = Ui::new(view, Theme::new(false, "#fbc711"));
    crate::text::paint_until_settled(|| ui.paint(&mut tiny_skia::Pixmap::new(400, 300).unwrap().as_mut(), 1.0));
    ui.moving = false;
    for scale in [1.0, 2.0] {
        let (w, h) = ((400.0 * scale) as u32, (300.0 * scale) as u32);
        let mut whole = tiny_skia::Pixmap::new(w, h).unwrap();
        ui.paint(&mut whole.as_mut(), scale);
        let mut held = whole.clone();
        held.data_mut().chunks_exact_mut(4).for_each(|p| p.swap(0, 2));
        let expected = held.clone();
        let parts = ui.paint_live(&mut held.as_mut(), scale).expect("painted in parts");
        assert!(parts.len() >= 4, "the card's two and what shows of the area's: {parts:?}");
        assert!(parts.iter().all(|r| r.w * r.h < (w * h) as f32 / 50.0), "each part is small: {parts:?}");
        let first = held.data().chunks_exact(4).zip(expected.data().chunks_exact(4)).position(|(a, b)| a != b)
            .map(|i| (i as u32 % w, i as u32 / w, held.data()[i * 4..i * 4 + 4].to_vec(), expected.data()[i * 4..i * 4 + 4].to_vec()));
        assert_eq!(first, None, "at scale {scale}, a part painted alone differs from the whole, first at this pixel; parts {parts:?}");
        for r in &parts {
            for y in r.y as u32..(r.y + r.h) as u32 {
                for x in r.x as u32..(r.x + r.w) as u32 {
                    let at = ((y * w + x) * 4) as usize;
                    held.data_mut()[at..at + 4].copy_from_slice(&[255, 0, 255, 255]);
                }
            }
        }
        assert_ne!(held.data(), expected.data());
        ui.paint_live(&mut held.as_mut(), scale).expect("painted in parts");
        assert_eq!(held.data(), expected.data(), "at scale {scale}, the parts were not all put back");
    }
}

#[test]
fn steps_come_out_as_a_person_would_write_them() {
    assert_eq!(snap(0.30000004, 0.0, 1.0, 0.1), 0.3);
    assert_eq!(snap(1.26, 0.0, 1.0, 0.1), 1.0);
    assert_eq!(snap(47.0, 0.0, 100.0, 5.0), 45.0);
}

#[test]
fn every_widget_paints_without_trouble() {
    let mut ui = ui(true, 0.3);
    let now = Instant::now();
    ui.input(&key(Keysym::Tab, false), now);
    ui.tip = None;
    let mut pixmap = tiny_skia::Pixmap::new(800, 600).unwrap();
    ui.paint(&mut pixmap.as_mut(), 2.0);
    let s = ui.spot("wifi").unwrap().rect;
    let px = pixmap.pixel((s.x + 8.0) as u32, (s.y + s.h / 2.0) as u32).unwrap();
    assert_eq!((px.red(), px.green(), px.blue()), u8s(ui.theme.soft));
    let b = ui.spot("0.0").unwrap().rect;
    let ring = pixmap.pixel((b.x + b.w / 2.0) as u32, (b.y - 7.0) as u32).unwrap();
    assert_eq!((ring.red(), ring.green()), (0xfb, 0xc7));
}

#[test]
fn a_kept_box_glows_in_the_accent_for_a_moment() {
    let view = |turn| column(vec![entry("John Smith", "Name", Msg::Name).key("name").glow(turn)]).padding(10.0);
    let mut ui = Ui::new(view(0), Theme::new(false, "#fbc711"));
    let now = Instant::now();
    assert_eq!(ui.glow_at("name", now), 0.0);
    assert_eq!(ui.next_wake(now), None);
    ui.set_view(view(1));
    let began = ui.glows.get("name").and_then(|g| g.1).unwrap();
    assert_eq!(ui.glow_at("name", began + theme::sheet().glow / 2), 1.0);
    assert_eq!(ui.glow_at("name", began + theme::sheet().glow), 0.0);
    assert_eq!(ui.next_wake(began), Some(FRAME));
    ui.set_view(view(1));
    assert_eq!(ui.glows.get("name").and_then(|g| g.1), Some(began));
    assert!(ui.tick(began + theme::sheet().glow));
    assert_eq!(ui.next_wake(began + theme::sheet().glow), None);
    ui.set_view(view(2));
    assert!(ui.glows.get("name").and_then(|g| g.1).is_some());
    ui.glows.insert("name".into(), (2, Some(ago(theme::sheet().glow / 2))));
    ui.size = (300, 80);
    let mut pixmap = tiny_skia::Pixmap::new(300, 80).unwrap();
    ui.paint(&mut pixmap.as_mut(), 1.0);
    let r = ui.spot("name").unwrap().rect;
    let p = pixmap.pixel((r.x + r.w / 2.0) as u32, r.y as u32).unwrap();
    assert!(p.red() > 150 && p.blue() < 120, "{p:?}");
}

#[test]
fn a_round_button_whose_press_was_kept_is_ringed_in_the_accent_for_a_moment() {
    let view = |turn| column(vec![button("document-open-symbolic", "Keep").key("keep").on_press(Msg::Go).glow(turn)]).padding(20.0);
    let ringed = |ui: &mut Ui<Msg>| {
        ui.size = (120, 90);
        let mut pixmap = tiny_skia::Pixmap::new(120, 90).unwrap();
        ui.paint(&mut pixmap.as_mut(), 1.0);
        let r = ui.spot("keep").unwrap().rect;
        let b = ui.disc(r, theme::icon_button(), false);
        (1..=3).any(|up| {
            let p = pixmap.pixel((b.x + b.w / 2.0) as u32, (b.y as u32).saturating_sub(up)).unwrap();
            p.red() > 150 && p.blue() < 120
        })
    };
    let mut ui = Ui::new(view(0), Theme::new(false, "#fbc711"));
    assert!(!ringed(&mut ui));
    ui.set_view(view(1));
    let began = ui.glows.get("keep").and_then(|g| g.1).unwrap();
    assert_eq!(ui.next_wake(began), Some(FRAME));
    ui.glows.insert("keep".into(), (1, Some(ago(theme::sheet().glow / 2))));
    assert!(ringed(&mut ui));
    ui.glows.insert("keep".into(), (1, Some(ago(theme::sheet().glow * 2))));
    assert!(!ringed(&mut ui));
}

#[test]
fn a_list_shows_a_row_the_program_chose() {
    let rows = || (0..40).map(|i| item(format!("Row {i}"))).collect::<Vec<_>>();
    let view = |at| column(vec![scroll("area", list(rows(), Some(at), Msg::Pick).key("rows"))
        .height(crate::widget::Length::Fixed(120.0))]);
    let mut ui = Ui::new(view(0), Theme::new(false, "#fbc711"));
    ui.size = (400, 300);
    ui.settle();
    assert_eq!(ui.offset(ui.spot("area").unwrap()), 0.0);
    ui.set_view(view(30));
    let (s, l) = (ui.spot("area").unwrap(), ui.spot("rows").unwrap());
    let top = l.rect.y + 30.0 * ui.px(theme::sheet().row);
    assert!(top >= s.rect.y && top + ui.px(theme::sheet().row) <= s.rect.y + s.rect.h, "{top}");
}

#[test]
fn a_drawing_kept_to_sight_is_painted_only_while_it_shows() {
    use std::cell::RefCell;
    use std::rc::Rc;
    let painted = Rc::new(RefCell::new(Vec::new()));
    let strips = |seen_only: bool| {
        let rows = (0..30).map(|i| {
            let painted = painted.clone();
            let d = drawing((10.0, 40.0), "", move |_, _| painted.borrow_mut().push(i))
                .width(crate::widget::Length::Fill).height(crate::widget::Length::Fixed(40.0));
            if seen_only { d.only_when_seen() } else { d }
        }).collect();
        column(vec![scroll("area", column(rows).gap(0.0)).height(crate::widget::Length::Fixed(100.0))])
    };
    let mut ui: Ui<Msg> = Ui::new(strips(true), Theme::new(false, "#fbc711"));
    let mut pixmap = tiny_skia::Pixmap::new(200, 300).unwrap();
    ui.paint(&mut pixmap.as_mut(), 1.0);
    assert_eq!(*painted.borrow(), vec![0, 1, 2]);
    painted.borrow_mut().clear();
    assert!(ui.scroll_area("area", 400.0));
    ui.paint(&mut pixmap.as_mut(), 1.0);
    assert_eq!(*painted.borrow(), vec![10, 11, 12]);
    painted.borrow_mut().clear();
    let mut ui: Ui<Msg> = Ui::new(strips(false), Theme::new(false, "#fbc711"));
    ui.paint(&mut pixmap.as_mut(), 1.0);
    assert_eq!(painted.borrow().len(), 30);
}

#[test]
fn a_toned_ring_keeps_its_colour_when_full() {
    let mut ui: Ui<Msg> = Ui::new(column(vec![ring(1.0, 60.0).tone(Tone::Problem).key("full"), label("Nearly full").tone(Tone::Warn)]),
                         Theme::new(false, "#fbc711"));
    ui.size = (200, 200);
    ui.settle();
    let r = ui.spot("full").unwrap().rect;
    let mut pixmap = tiny_skia::Pixmap::new(200, 200).unwrap();
    ui.paint(&mut pixmap.as_mut(), 1.0);
    let px = pixmap.pixel((r.x + r.w / 2.0) as u32, (r.y + 4.0) as u32).unwrap();
    let danger = ui.theme.danger;
    let near = |got: u8, want: f32| (got as i32 - (want * 255.0).round() as i32).abs() <= 2;
    assert!(near(px.red(), danger.r) && near(px.green(), danger.g) && near(px.blue(), danger.b), "the full ring is not in the danger colour");
    let mid = pixmap.pixel((r.x + r.w / 2.0) as u32, (r.y + r.h / 2.0) as u32).unwrap();
    assert_eq!(mid, pixmap.pixel(199, 199).unwrap());
}

#[test]
fn a_working_button_has_its_line_round_its_own_edge() {
    let mut ui: Ui<Msg> = Ui::new(column(vec![button("view-refresh-symbolic", "Look again").working(0.5, Tone::Calm).key("look")]),
                         Theme::new(true, "#fbc711"));
    ui.size = (100, 100);
    ui.settle();
    let r = ui.spot("look").unwrap().rect;
    let mut pixmap = tiny_skia::Pixmap::new(100, 100).unwrap();
    ui.paint(&mut pixmap.as_mut(), 1.0);
    let calm = |x: f32, y: f32| {
        let px = pixmap.pixel(x as u32, y as u32).unwrap();
        px.blue() as i32 - px.red() as i32 > 100
    };
    let (cx, cy) = (r.x + r.w / 2.0, r.y + r.h / 2.0);
    assert!(calm(r.x + r.w - 1.5, cy), "no line at the right edge");
    assert!(!calm(r.x + 1.5, cy), "the line went past half way");
    assert!(!calm(r.x + r.w + 2.0, cy));
    assert!(!calm(cx, cy));
}

fn painted_card(card: Node<Msg>) -> tiny_skia::Pixmap {
    let mut ui = Ui::new(column(vec![card.key("c").width(crate::widget::Length::Fixed(200.0)).height(crate::widget::Length::Fixed(100.0))]), Theme::new(false, "#fbc711"));
    ui.size = (300, 200);
    ui.settle();
    let mut pixmap = tiny_skia::Pixmap::new(300, 200).unwrap();
    ui.paint(&mut pixmap.as_mut(), 1.0);
    pixmap
}

#[test]
fn a_faded_widget_shows_only_its_share() {
    let shade = |p: &tiny_skia::Pixmap, x: u32, y: u32| { let c = p.pixel(x, y).unwrap(); c.red() as i32 + c.green() as i32 + c.blue() as i32 };
    let full = painted_card(card(label("Words")));
    let none = painted_card(card(label("Words")).opacity(0.0));
    let half = painted_card(card(label("Words")).opacity(0.5));
    let (x, y) = (150, 90);
    let (f, n, h) = (shade(&full, x, y), shade(&none, x, y), shade(&half, x, y));
    assert_ne!(f, n, "the card shows on the window");
    assert!((h - (f + n) / 2).abs() <= 6, "half way: {f} {h} {n}");
    assert_eq!(shade(&full, 290, 190), shade(&none, 290, 190));
    assert_eq!(shade(&full, 290, 190), shade(&half, 290, 190));
}

#[test]
fn a_lit_card_has_its_edge_drawn_in_the_highlight() {
    let plain = painted_card(card(label("Words")));
    let lit = painted_card(card(label("Words")).lit(1.0));
    let differs = |a: &tiny_skia::Pixmap, b: &tiny_skia::Pixmap| a.pixels().iter().zip(b.pixels()).filter(|(p, q)| p != q).count();
    assert!(differs(&plain, &lit) > 200);
    assert_eq!(differs(&plain, &painted_card(card(label("Words")).lit(0.0))), 0);
}

#[test]
fn a_rows_detail_sits_at_its_far_end_and_cuts_its_words_short() {
    let long = "A very long name that will not fit beside its size at all, not even close";
    let mut ui = Ui::new(list(vec![item(long).detail("12 MB")], None, Msg::Pick).key("list"), Theme::new(false, "#fbc711"));
    ui.size = (300, 100);
    ui.settle();
    let mut pixmap = tiny_skia::Pixmap::new(300, 100).unwrap();
    ui.paint(&mut pixmap.as_mut(), 1.0);
    let r = ui.spot("list").unwrap().rect;
    let h = ui.px(theme::sheet().row);
    let at = |x: f32, y: f32| pixmap.pixel(x as u32, y as u32).unwrap().red() as i32;
    let plain = at(r.x + r.w / 2.0, r.y + 1.0);
    let ink = |x0: f32, x1: f32| (x0 as i32..x1 as i32).any(|x| (2..h as i32 - 2).any(|y| (at(x as f32, r.y + y as f32) - plain).abs() > 60));
    assert!(ink(r.x + r.w - 30.0, r.x + r.w - 4.0));
    assert!(ink(r.x + 4.0, r.x + 60.0));
}

#[test]
fn marked_letters_in_rows_and_labels_get_a_wash_and_keep_their_words() {
    let paint = |node: Node<Msg>| {
        let mut ui = Ui::new(node, Theme::new(false, "#fbc711"));
        ui.size = (300, 100);
        ui.settle();
        let mut pixmap = tiny_skia::Pixmap::new(300, 100).unwrap();
        ui.paint(&mut pixmap.as_mut(), 1.0);
        pixmap
    };
    let differs = |a: &tiny_skia::Pixmap, b: &tiny_skia::Pixmap| a.pixels().iter().zip(b.pixels()).filter(|(p, q)| p != q).count();
    let rows = |marks: &[std::ops::Range<usize>]| list(vec![item("Café au lait").marked(Field::Text, marks)], None, Msg::Pick).key("list");
    let plain = paint(rows(&[]));
    let marked = paint(rows(&[4..5, 3..6, 90..99]));
    assert!(differs(&plain, &marked) > 40);
    assert_eq!(differs(&marked, &paint(rows(&[3..6]))), 0);
    assert_eq!(differs(&plain, &paint(rows(&[90..99, 2..2]))), 0);
    let label_marked = paint(label("Café au lait").marked(&[0..3]));
    assert!(differs(&paint(label("Café au lait")), &label_marked) > 40);
    let mut ui = Ui::new(column::<Msg>(vec![label("Café au lait").marked(&[0..3]).key("l")]), Theme::new(false, "#fbc711"));
    ui.size = (300, 100);
    ui.settle();
    ui.paint(&mut tiny_skia::Pixmap::new(300, 100).unwrap().as_mut(), 1.0);
    let Some(What::Label { text, marks, .. }) = ui.node("l").map(|n| &n.what) else { panic!("no label") };
    assert_eq!((text.as_str(), marks.clone()), ("Café au lait", vec![0..3]));
}

#[test]
fn a_rich_row_has_its_name_over_a_quieter_line() {
    let row = vec![item("Holiday photos").under("Folder").icon("folder-symbolic").folder().detail("3 Oct")];
    let mut ui = Ui::new(list(row, None, Msg::Pick).row_height(theme::sheet().rich_row).key("list"), Theme::new(false, "#fbc711"));
    ui.size = (300, 100);
    ui.settle();
    let mut pixmap = tiny_skia::Pixmap::new(300, 100).unwrap();
    ui.paint(&mut pixmap.as_mut(), 1.0);
    let r = ui.spot("list").unwrap().rect;
    let words = r.x + ui.px(theme::sheet().entry_padding + theme::sheet().rich_icon + theme::sheet().gap);
    let at = |x: f32, y: f32| pixmap.pixel(x as u32, y as u32).unwrap().red() as i32;
    let plain = at(r.x + r.w / 2.0, r.y + 1.0);
    let inked = |y0: f32, y1: f32| (words as i32..words as i32 + 40).any(|x| (y0 as i32..y1 as i32).any(|y| (at(x as f32, y as f32) - plain).abs() > 60));
    let mid = r.y + ui.px(theme::sheet().rich_row) / 2.0;
    assert!(inked(r.y + 2.0, mid - 1.0));
    assert!(inked(mid + 1.0, r.y + ui.px(theme::sheet().rich_row) - 2.0));
}

#[test]
fn a_scrolled_list_paints_only_inside_its_area() {
    let mut ui = Ui::new(column(vec![
        button("x", "Top").on_press(Msg::Save).key("top"),
        scroll("area", list(rows(40), Some(3), Msg::Pick).key("list")).height(crate::widget::Length::Fixed(200.0)),
    ]), Theme::new(false, "#fbc711"));
    let now = Instant::now();
    let mut pixmap = tiny_skia::Pixmap::new(400, 300).unwrap();
    ui.paint(&mut pixmap.as_mut(), 1.0);
    let a = ui.spot("area").unwrap().rect;
    let row = ui.spot("list").unwrap().rect.y + 3.5 * ui.px(theme::sheet().row);
    let px = pixmap.pixel((a.x + 3.0) as u32, row as u32).unwrap();
    let bg = ui.theme.bg.to_skia().to_color_u8();
    assert!(px.red() > bg.red());
    ui.input(&Input::PointerMove { x: a.x + 50.0, y: a.y + 50.0 }, now);
    ui.input(&wheel(3.0), now);
    let mut pixmap = tiny_skia::Pixmap::new(400, 300).unwrap();
    ui.paint(&mut pixmap.as_mut(), 1.0);
    let above = pixmap.pixel((a.x + 3.0) as u32, (a.y - 4.0).max(0.0) as u32).unwrap();
    assert_eq!((above.red(), above.green(), above.blue()), (bg.red(), bg.green(), bg.blue()));
    let mut ui = Ui::new(list(vec![item("A very long name ".repeat(20))], None, Msg::Pick), Theme::new(false, "#fbc711"));
    let mut pixmap = tiny_skia::Pixmap::new(300, 100).unwrap();
    ui.paint(&mut pixmap.as_mut(), 1.0);
    let pad = ui.px(theme::sheet().entry_padding) as u32;
    for x in 300 - pad + 1..300 {
        for y in 0..40 {
            let p = pixmap.pixel(x, y).unwrap();
            assert_eq!((p.red(), p.green(), p.blue()), (bg.red(), bg.green(), bg.blue()));
        }
    }
}

fn u8s(c: Colour) -> (u8, u8, u8) {
    let c = c.to_skia().to_color_u8();
    (c.red(), c.green(), c.blue())
}

fn rgb(pixmap: &tiny_skia::Pixmap, x: u32, y: u32) -> (u8, u8, u8) {
    let p = pixmap.pixel(x, y).unwrap();
    (p.red(), p.green(), p.blue())
}

fn ring_ui(share: f32) -> (Ui<()>, tiny_skia::Pixmap) {
    let mut ui = Ui::new(column(vec![ring(share, 44.0)]), Theme::chosen());
    let mut pixmap = tiny_skia::Pixmap::new(60, 60).unwrap();
    ui.paint(&mut pixmap.as_mut(), 1.0);
    (ui, pixmap)
}

#[test]
fn a_ring_fills_clockwise_from_the_top_and_closes_in_the_words_colour_when_done() {
    let calm = u8s(theme::calm());
    let (ui, quarter) = ring_ui(0.25);
    let bg = u8s(ui.theme.bg);
    assert_eq!(rgb(&quarter, 35, 9), calm);
    let track = rgb(&quarter, 4, 22);
    assert!(track != bg && track != calm);
    assert_eq!(rgb(&quarter, 22, 30), bg);
    let (ui, done) = ring_ui(1.0);
    let ok = u8s(ui.theme.fg);
    assert_eq!(rgb(&done, 4, 22), ok);
    assert!(!(14..30).any(|x| rgb(&done, x, 22) == u8s(ui.theme.icon)));
    let (_, empty) = ring_ui(0.0);
    assert_eq!(rgb(&empty, 35, 9), rgb(&empty, 9, 35));
    let theme = Theme::new(false, "#fbc711");
    let accent = u8s(theme.soft);
    let mut ui = Ui::new(column::<()>(vec![ring(1.0, 44.0).tone(Tone::Accent)]), theme);
    let mut full = tiny_skia::Pixmap::new(60, 60).unwrap();
    ui.paint(&mut full.as_mut(), 1.0);
    assert_eq!(rgb(&full, 4, 22), accent);
    assert_eq!(rgb(&full, 35, 9), accent);
    assert!(!(14..30).any(|x| rgb(&full, x, 22) == u8s(ui.theme.fg)));
}

#[test]
fn a_see_through_window_fades_as_a_whole() {
    let mut ui = Ui::new(column::<()>(vec![card(label("Hello"))]).padding(10.0), Theme::chosen());
    ui.see_through = true;
    let mut full = tiny_skia::Pixmap::new(120, 80).unwrap();
    ui.paint(&mut full.as_mut(), 1.0);
    assert!(full.pixel(2, 2).unwrap().alpha() < 40);
    assert_eq!(full.pixel(60, 40).unwrap().alpha(), 255);
    ui.shown = 0.5;
    let mut half = tiny_skia::Pixmap::new(120, 80).unwrap();
    ui.paint(&mut half.as_mut(), 1.0);
    let (a, b) = (full.pixel(60, 40).unwrap(), half.pixel(60, 40).unwrap());
    assert!((b.alpha() as i32 - 128).abs() <= 1 && (b.red() as i32 - a.red() as i32 / 2).abs() <= 1);
}

#[test]
fn natural_height_is_what_the_widgets_ask_for() {
    let ui = Ui::new(column::<()>(vec![ring(0.0, 44.0)]).padding(10.0), Theme::chosen());
    assert_eq!(ui.natural_height(200.0), 64.0);
}

#[test]
fn a_whole_window_change_fades_the_old_view_out_over_the_new() {
    use crate::widget::{drawing, Length};
    let white = || column::<()>(vec![drawing((10.0, 10.0), "", |c: &mut PixmapMut, _: &_| c.fill(tiny_skia::Color::WHITE))
        .width(Length::Fill).height(Length::Fill)]);
    let mut ui = Ui::new(white(), Theme::new(false, "#fbc711"));
    ui.size = (10, 10);
    let red = |ui: &mut Ui<()>| {
        let mut pixmap = Pixmap::new(10, 10).unwrap();
        ui.paint(&mut pixmap.as_mut(), 1.0);
        pixmap.pixel(5, 5).unwrap().red()
    };
    ui.blend_to(Some((0, 1.0)));
    assert_eq!(red(&mut ui), 255);
    let bg = (ui.theme.bg.r * 255.0).round() as i32;
    ui.blend_to(Some((1, 0.0)));
    ui.set_view(column(vec![]));
    assert_eq!(red(&mut ui), 255);
    ui.blend_to(Some((1, 0.5)));
    assert!((i32::from(red(&mut ui)) - (255 + bg) / 2).abs() <= 2);
    ui.blend_to(Some((1, 1.0)));
    assert_eq!(i32::from(red(&mut ui)), bg);
    assert!(ui.before.is_none());
}

#[test]
fn every_kind_of_button_is_one_height() {
    for scale in [1.0, 1.5, 1.75, 2.0] {
        let mut ui = Ui::new(row(vec![
            button("window-close-symbolic", "Cancel").key("icon"),
            crate::widget::main_action("folder-download-symbolic", "Install").key("main"),
            crate::widget::main_word("Save").key("word_main"),
            crate::widget::quiet_action("folder-new-symbolic", "New").key("quiet"),
            crate::widget::word_button("Dark").key("pill"),
            crate::widget::choice(vec!["One".into(), "Two".into()], 0, "Pick", |_| Msg::Go).key("choice"),
        ]).centred(), Theme::new(false, "#fbc711"));
        ui.scale = scale;
        ui.size = ((900.0 * scale) as u32, (100.0 * scale) as u32);
        ui.settle();
        let want = ui.px(theme::sheet().button);
        for id in ["icon", "main", "word_main", "quiet", "pill", "choice"] {
            let h = ui.spot(id).unwrap().rect.h;
            assert_eq!(h, want, "{id} at scale {scale}");
        }
    }
}

#[test]
fn a_ring_button_is_a_soft_square_not_a_disc() {
    let mut ui = Ui::new(row(vec![button::<Msg>("window-close-symbolic", "Close").tinted(0.5).size(40.0).key("b")]).centred(), Theme::new(false, "#fbc711"));
    ui.size = (200, 100);
    ui.settle();
    let r = ui.spot("b").unwrap().rect;
    let mut pm = tiny_skia::Pixmap::new(200, 100).unwrap();
    ui.paint(&mut pm.as_mut(), 1.0);
    let at = |x: f32, y: f32| pm.pixel(x as u32, y as u32).unwrap();
    let back = at(2.0, 2.0);
    assert_ne!(at(r.x + r.w / 2.0, r.y + 2.0), back, "edge middle is filled");
    assert_ne!(at(r.x + 3.0, r.y + r.h / 2.0), back, "side middle is filled");
    assert_eq!(at(r.x, r.y), back, "the corner point is cut");
    assert_ne!(at(r.x + 4.0, r.y + 4.0), back, "a disc would have cut this point");
}

#[test]
fn square_things_are_soft_squares_not_discs() {
    use crate::widget::{avatar, icon_circle};
    let mut ui = Ui::new(column(vec![card(row(vec![
        icon_circle::<Msg>("window-close-symbolic").key("badge"),
        avatar::<Msg>(None, "John Smith", 40.0).key("avatar"),
        button::<Msg>("window-close-symbolic", "Close").tinted(0.5).size(40.0).key("ring"),
    ]).gap(20.0))]), Theme::new(false, "#fbc711"));
    ui.size = (300, 100);
    ui.settle();
    let mut pm = tiny_skia::Pixmap::new(300, 100).unwrap();
    ui.paint(&mut pm.as_mut(), 1.0);
    for id in ["badge", "avatar", "ring"] {
        let r = ui.spot(id).unwrap().rect;
        let back = pm.pixel((r.x - 4.0) as u32, (r.y + r.h / 2.0) as u32).unwrap();
        let p = pm.pixel((r.x + 3.0) as u32, (r.y + 3.0) as u32).unwrap();
        assert_ne!(p, back, "{id}: its corner point is empty, so it is a disc");
    }
}
