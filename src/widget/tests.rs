//! Tests of describing widgets and laying them out.

use std::collections::HashMap;

use super::*;

#[test]
fn a_frame_partly_off_the_canvas_is_cut_not_squeezed() {
    let mut frame = Pixmap::new(4, 1).unwrap();
    for (i, p) in frame.data_mut().chunks_exact_mut(4).enumerate() {
        p.copy_from_slice(&[i as u8 * 10, 0, 0, 255]);
    }
    let frames: Frames = Arc::new(Mutex::new(Some(frame)));
    let mut canvas = Pixmap::new(3, 1).unwrap();
    put_frame(&mut canvas.as_mut(), Rect { x: -2.0, y: 0.0, w: 4.0, h: 1.0 }, &frames, true);
    let reds: Vec<u8> = canvas.data().chunks_exact(4).map(|p| p[0]).collect();
    assert_eq!(reds, [20, 30, 0], "the frame's right half lands at the left edge");
}

#[test]
fn an_inner_reason_for_being_off_is_kept() {
    let inner: Node<()> = button("a", "A").off_because(Some("Inner"));
    let outer = column(vec![inner, button("b", "B")]).off_because(Some("Outer"));
    let What::Column { items, .. } = &outer.what else { panic!() };
    assert_eq!((items[0].why.as_deref(), items[1].why.as_deref()), (Some("Inner"), Some("Outer")));
    let kept = column(vec![button::<()>("c", "C").off_because(Some("Own"))]).off_because(None);
    let What::Column { items, .. } = &kept.what else { panic!() };
    assert_eq!(items[0].why.as_deref(), Some("Own"));
}

#[test]
fn a_secret_box_carries_no_spelling_marks() {
    let mut n: Node<()> = entry("hunter2", "Password", |_| ());
    if let What::Entry { misspelt, .. } = &mut n.what {
        misspelt.push(0..6);
    }
    let n = n.secret();
    assert!(matches!(&n.what, What::Entry { misspelt, .. } if misspelt.is_empty()));
}

fn lay(n: &Node<()>, w: f32, h: f32) -> Vec<Spot> {
    lay_scrolled(n, w, h, &HashMap::new())
}

fn lay_scrolled(n: &Node<()>, w: f32, h: f32, scrolled: &HashMap<String, f32>) -> Vec<Spot> {
    let (fonts, texts) = (Fonts::new(), Texts::default());
    let lay = Lay::bare(&fonts, &texts, scrolled);
    let mut spots = Vec::new();
    lay.place(n, Rect { x: 0.0, y: 0.0, w, h }, &mut Vec::new(), &mut spots);
    spots
}

#[test]
fn a_row_gives_what_is_left_to_what_fills() {
    let n = row(vec![button("a", "A"), slider(0.5, 0.0, 1.0, 0.1, "B", |_| ()), button("c", "C")]).gap(10.0);
    let s = lay(&n, 400.0, 36.0);
    assert_eq!(s.iter().map(|s| s.rect.w).collect::<Vec<_>>(), vec![34.0, 312.0, 34.0]);
    assert_eq!(s[2].rect.x, 366.0);
    assert_eq!(s[1].rect.y, 6.0);
}

#[test]
fn an_even_grid_fills_the_width_and_lines_up_each_row() {
    let cards = (0..5).map(|i| card(column((0..=i % 2).map(|_| button("x", "X")).collect()))).collect();
    let s = lay(&even_grid(300.0, cards).gap(20.0), 1000.0, 800.0);
    let cards: Vec<_> = s.iter().filter(|s| s.path.len() == 1).collect();
    assert_eq!(cards.iter().map(|c| c.rect.x).collect::<Vec<_>>(), vec![0.0, 340.0, 680.0, 0.0, 340.0]);
    assert_eq!(cards[0].rect.w, 320.0);
    assert_eq!(cards[2].rect.x + cards[2].rect.w, 1000.0);
    assert_eq!(cards[0].rect.h, cards[1].rect.h);
    assert_eq!(cards[3].rect.y, cards[0].rect.y + cards[0].rect.h + 20.0);
}

#[test]
fn a_column_shares_spare_height_and_keys_name_widgets() {
    let n = column(vec![button("a", "A").key("first"), card(label("x")).height(Length::Fill), button("b", "B")])
        .padding(10.0).gap(0.0);
    let s = lay(&n, 200.0, 300.0);
    assert_eq!(s[0].id, "first");
    assert_eq!(s[1].id, "1");
    assert_eq!(s[1].rect.h, 300.0 - 20.0 - 68.0);
    assert_eq!(s.last().unwrap().rect.y, 290.0 - 34.0);
}

#[test]
fn long_words_wrap_inside_their_box() {
    let n = column(vec![label("Words that are far too long to sit on one line in a narrow box")]);
    let s = lay(&n, 120.0, 400.0);
    assert!(s[0].rect.w <= 120.0);
    assert!(s[0].rect.h > 40.0, "{:?}", s[0].rect);
}

#[test]
fn a_scrolling_area_keeps_its_bar_clear_of_what_it_holds() {
    let gutter = theme::sheet().scrollbar_hover + 2.0 * theme::sheet().scrollbar_inset;
    let cards = |n: usize| scroll("area", column((0..n).map(|i| card(button(format!("{i}"), "B"))).collect()));
    let s = lay(&cards(1), 200.0, 300.0);
    assert_eq!(s.iter().find(|s| s.path == [0, 0]).unwrap().rect.w, 200.0);
    let s = lay(&cards(20), 200.0, 300.0);
    assert_eq!(s.iter().find(|s| s.path == [0, 0]).unwrap().rect.w, 200.0 - gutter);
}

#[test]
fn a_scrolling_area_takes_what_is_left_and_moves_what_it_holds() {
    let rows = (0..20).map(|i| button(format!("{i}"), "B")).collect();
    let n = column(vec![button("top", "T"), scroll("area", column(rows).gap(4.0))]).gap(0.0);
    let s = lay(&n, 200.0, 234.0);
    let area = s.iter().find(|s| s.id == "area").unwrap();
    assert_eq!((area.rect.y, area.rect.h), (34.0, 200.0));
    assert_eq!(area.full, 20.0 * 34.0 + 19.0 * 4.0);
    let first = s.iter().find(|s| s.path == [1, 0, 0]).unwrap();
    assert_eq!(first.rect.y, 34.0);
    assert_eq!(first.clip, Some(area.rect));
    let s = lay_scrolled(&n, 200.0, 234.0, &HashMap::from([("area".to_string(), 100.0)]));
    let first = s.iter().find(|s| s.path == [1, 0, 0]).unwrap();
    assert_eq!(first.rect.y, -66.0);
    assert!(!first.shows(10.0, 0.0) && !first.shows(10.0, -30.0));
    let s = lay_scrolled(&n, 200.0, 228.0, &HashMap::from([("area".to_string(), 1e6)]));
    let last = s.iter().find(|s| s.path == [1, 0, 19]).unwrap();
    assert_eq!(last.rect.y + last.rect.h, 228.0);
}

#[test]
fn a_sideways_window_cuts_and_moves_what_it_holds() {
    let icons = (0..6).map(|i| button(format!("{i}"), "B").size(30.0)).collect();
    let n = row(vec![slide(row(icons).gap(2.0), 100.0, 64.0).key("strip")]);
    let s = lay(&n, 400.0, 40.0);
    let strip = s.iter().find(|s| s.id == "strip").unwrap();
    assert_eq!((strip.rect.x, strip.rect.w), (0.0, 100.0));
    let third = s.iter().find(|s| s.path == [0, 0, 2]).unwrap();
    assert_eq!(third.rect.x, 0.0);
    let first = s.iter().find(|s| s.path == [0, 0, 0]).unwrap();
    assert!(!first.shows(first.rect.x + 5.0, 10.0));
    let last = s.iter().find(|s| s.path == [0, 0, 5]).unwrap();
    assert!(!last.shows(last.rect.x + 5.0, 10.0));
}

#[test]
fn a_list_is_a_row_tall_for_each_row() {
    let n = column(vec![list(vec![item("One").icon("folder"), item("Two"), item("Three")], Some(1), |_| ())]);
    let s = lay(&n, 300.0, 400.0);
    assert_eq!((s[0].rect.w, s[0].rect.h), (300.0, 3.0 * theme::sheet().row));
    assert!(n.items()[0].focusable());
    assert!(!list::<()>(vec![], None, |_| ()).focusable());
}

#[test]
fn a_pressable_drawing_is_reached_by_the_keyboard() {
    assert!(!drawing::<u8>((10.0, 10.0), "Band", |_, _| ()).focusable());
    assert!(drawing((10.0, 10.0), "Band", |_, _| ()).pressable(1u8).focusable());
}

#[test]
fn a_centred_column_puts_each_item_on_its_middle() {
    let n = column(vec![ring(0.5, 40.0), button("a", "A")]).centred();
    let s = lay(&n, 200.0, 100.0);
    assert_eq!(s.iter().map(|s| s.rect.x).collect::<Vec<_>>(), vec![80.0, 83.0]);
    let left = lay(&column(vec![ring(0.5, 40.0)]), 200.0, 100.0);
    assert_eq!(left[0].rect.x, 0.0);
}
