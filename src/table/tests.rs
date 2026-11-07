//! Tests of table.

use super::*;
use crate::input::Keysym;
use std::time::Duration;

fn table() -> Table {
    let mut t = Table::new(vec![Column::new("Name", 200.0).sortable(), Column::new("Size", 80.0).sortable().end(), Column::new("Kind", 100.0)]);
    t.set_rows(100);
    t
}

fn at(g: &mut Gesture, t: &mut Table, e: Pointer, ms: u64, base: Instant) -> Option<Touch> {
    g.pointer(t, e, base + Duration::from_millis(ms), (400.0, 400.0))
}

#[test]
fn rows_lie_under_the_heading_and_are_found_by_point() {
    let t = table();
    assert_eq!(t.row_rect(0).y, 32.0);
    assert_eq!(t.row_at(10.0, 33.0, 400.0), Some(0));
    assert_eq!(t.row_at(10.0, 32.0 + 36.0 * 2.0 + 1.0, 400.0), Some(2));
    assert_eq!(t.row_at(10.0, 10.0, 400.0), None);
    assert_eq!(t.row_at(381.0, 40.0, 400.0), None);
}

#[test]
fn the_bar_is_dragged_to_scroll_the_rows() {
    let (mut t, mut g, now) = (table(), Gesture::new(), Instant::now());
    let size = (400.0, 400.0);
    let bar = t.bar(size.0, size.1).expect("a hundred rows do not fit");
    let grab = (bar.x + bar.w / 2.0, bar.y + 8.0);
    assert_eq!(g.pointer(&mut t, Pointer::Down(grab.0, grab.1), now, size), None);
    let halfway = (400.0 - 32.0 - bar.h) / 2.0;
    assert_eq!(g.pointer(&mut t, Pointer::Move(grab.0, grab.1 + halfway), now, size), Some(Touch::Scrolling));
    assert!((t.first - 45.0).abs() < 0.01, "{}", t.first);
    g.pointer(&mut t, Pointer::Move(grab.0, 5000.0), now, size);
    assert_eq!(t.first, 90.0);
    assert_eq!(g.pointer(&mut t, Pointer::Up(grab.0, 5000.0), now, size), None);
    assert_eq!(g.pointer(&mut t, Pointer::Move(grab.0, 100.0), now, size), None);
}

#[test]
fn only_rows_in_sight_are_visible() {
    let mut t = table();
    assert_eq!(t.visible(32.0 + 36.0 * 3.0), 0..3);
    t.first = 10.5;
    assert_eq!(t.visible(32.0 + 36.0 * 3.0), 10..14);
    t.first = 98.0;
    assert_eq!(t.visible(400.0), 98..100);
}

#[test]
fn a_press_on_a_heading_sorts_and_again_flips() {
    let (mut t, mut g, now) = (table(), Gesture::new(), Instant::now());
    assert_eq!(at(&mut g, &mut t, Pointer::Down(50.0, 10.0), 0, now), None);
    assert_eq!(at(&mut g, &mut t, Pointer::Up(50.0, 10.0), 10, now), Some(Touch::Sorted(0, true)));
    at(&mut g, &mut t, Pointer::Down(50.0, 10.0), 1000, now);
    assert_eq!(at(&mut g, &mut t, Pointer::Up(50.0, 10.0), 1010, now), Some(Touch::Sorted(0, false)));
    at(&mut g, &mut t, Pointer::Down(250.0, 10.0), 2000, now);
    assert_eq!(at(&mut g, &mut t, Pointer::Up(250.0, 10.0), 2010, now), Some(Touch::Sorted(1, true)));
}

#[test]
fn a_heading_that_cannot_be_sorted_stays_as_it_is() {
    let (mut t, mut g, now) = (table(), Gesture::new(), Instant::now());
    at(&mut g, &mut t, Pointer::Down(350.0, 10.0), 0, now);
    assert_eq!(at(&mut g, &mut t, Pointer::Up(350.0, 10.0), 10, now), None);
    assert_eq!(t.sort, None);
}

#[test]
fn dragging_an_edge_resizes_and_never_below_the_least() {
    let (mut t, mut g, now) = (table(), Gesture::new(), Instant::now());
    at(&mut g, &mut t, Pointer::Down(201.0, 10.0), 0, now);
    assert_eq!(at(&mut g, &mut t, Pointer::Move(261.0, 10.0), 10, now), Some(Touch::Resizing(0)));
    assert_eq!(t.columns[0].width, 260.0);
    at(&mut g, &mut t, Pointer::Move(5.0, 10.0), 20, now);
    assert_eq!(t.columns[0].width, 48.0);
    assert_eq!(at(&mut g, &mut t, Pointer::Up(5.0, 10.0), 30, now), Some(Touch::Resized(0)));
    assert_eq!(t.sort, None);
}

#[test]
fn a_double_click_on_an_edge_asks_to_fit() {
    let (mut t, mut g, now) = (table(), Gesture::new(), Instant::now());
    at(&mut g, &mut t, Pointer::Down(200.0, 10.0), 0, now);
    at(&mut g, &mut t, Pointer::Up(200.0, 10.0), 5, now);
    assert_eq!(at(&mut g, &mut t, Pointer::Down(200.0, 10.0), 100, now), Some(Touch::Fit(0)));
    t.fit(0, 120.0);
    assert_eq!(t.columns[0].width, 136.0);
}

#[test]
fn rows_are_chosen_opened_toggled_and_extended() {
    let (mut t, mut g, now) = (table(), Gesture::new(), Instant::now());
    assert_eq!(at(&mut g, &mut t, Pointer::Down(10.0, 70.0), 0, now), Some(Touch::Chose(1)));
    assert_eq!(t.chosen, Some(1));
    at(&mut g, &mut t, Pointer::Up(10.0, 70.0), 5, now);
    assert_eq!(at(&mut g, &mut t, Pointer::Down(10.0, 70.0), 100, now), Some(Touch::Opened(1)));
    at(&mut g, &mut t, Pointer::Up(10.0, 70.0), 105, now);
    let ctrl = Mods { ctrl: true, ..Mods::default() };
    assert_eq!(g.pointer_mods(&mut t, Pointer::Down(10.0, 110.0), now + Duration::from_secs(5), ctrl, (400.0, 400.0)), Some(Touch::Toggled(2)));
    let shift = Mods { shift: true, ..Mods::default() };
    assert_eq!(g.pointer_mods(&mut t, Pointer::Down(10.0, 150.0), now + Duration::from_secs(9), shift, (400.0, 400.0)), Some(Touch::Extended(3)));
    assert_eq!(t.chosen, Some(1));
}

#[test]
fn the_keys_move_the_chosen_row_and_keep_it_in_sight() {
    let mut t = table();
    let tall = 32.0 + 36.0 * 5.0;
    assert_eq!(t.step(1, tall), Some(0));
    assert_eq!(t.step(1, tall), Some(1));
    for _ in 0..6 {
        t.step(1, tall);
    }
    assert_eq!(t.chosen, Some(7));
    assert!(t.first >= 3.0 && t.first <= 7.0);
    assert_eq!(t.end(tall), Some(99));
    assert_eq!(t.first, 95.0);
    assert_eq!(t.home(tall), Some(0));
    assert_eq!(t.first, 0.0);
    assert_eq!(t.step(-1, tall), Some(0));
    t.chosen = None;
    assert_eq!(t.end(tall), Some(99));
}

#[test]
fn scrolling_stays_within_the_rows() {
    let mut t = table();
    t.scroll(-5.0);
    assert_eq!(t.first, 0.0);
    t.scroll(1000.0);
    assert_eq!(t.first, 99.0);
    t.set_rows(10);
    assert_eq!(t.first, 9.0);
}

#[test]
fn fewer_rows_drop_a_chosen_row_that_is_gone() {
    let mut t = table();
    t.chosen = Some(50);
    t.set_rows(10);
    assert_eq!(t.chosen, None);
}

#[test]
fn the_cursor_shows_a_column_pull_over_an_edge() {
    let (mut t, g) = (table(), Gesture::new());
    assert_eq!(g.cursor(&t, 200.0, 10.0), Cursor::ColResize);
    assert_eq!(g.cursor(&t, 100.0, 10.0), Cursor::Hand);
    assert_eq!(g.cursor(&t, 100.0, 100.0), Cursor::Arrow);
    t.columns[0].sortable = false;
    assert_eq!(g.cursor(&t, 100.0, 10.0), Cursor::Arrow);
}

#[test]
fn what_a_reader_is_told_is_the_rows_in_sight() {
    let mut t = table();
    t.first = 20.0;
    t.chosen = Some(21);
    t.sort = Some((1, false));
    let said = t.said(32.0 + 36.0 * 3.0, |r, c| format!("{r}.{c}"));
    assert_eq!(said.total, 100);
    assert_eq!(said.headings, ["Name", "Size", "Kind"]);
    assert_eq!(said.rows.len(), 3);
    assert_eq!(said.rows[0].index, 20);
    assert_eq!(said.rows[1].cells, ["21.0", "21.1", "21.2"]);
    assert!(said.rows[1].chosen && !said.rows[0].chosen);
    assert_eq!(said.sort, Some((1, false)));
}

#[test]
fn a_double_click_on_an_edge_leaves_no_drag_behind() {
    let (mut t, mut g, now) = (table(), Gesture::new(), Instant::now());
    at(&mut g, &mut t, Pointer::Down(200.0, 10.0), 0, now);
    at(&mut g, &mut t, Pointer::Up(200.0, 10.0), 5, now);
    assert_eq!(at(&mut g, &mut t, Pointer::Down(200.0, 10.0), 100, now), Some(Touch::Fit(0)));
    at(&mut g, &mut t, Pointer::Move(100.0, 10.0), 110, now);
    assert_eq!(t.columns[0].width, 200.0);
}

#[test]
fn a_row_height_of_nothing_divides_nothing_by_zero() {
    let mut t = table();
    t.row_height = 0.0;
    assert!(t.rows_in(400.0) <= 400);
    assert!(t.visible(400.0).len() <= 400);
    assert!(t.row_at(10.0, 40.0, 400.0).is_some());
    assert!(t.bar(400.0, 10.0).is_none_or(|b| b.h >= 0.0));
}

#[test]
fn group_headings_take_a_line_each_and_rows_move_down() {
    let mut t = table();
    t.set_groups(vec![(0, "Today".into()), (3, "Earlier".into()), (500, "Gone".into())]);
    assert_eq!(t.groups.len(), 2);
    assert_eq!(t.lines(), 102);
    assert_eq!(t.at_line(0), Line::Group(0));
    assert_eq!(t.at_line(1), Line::Row(0));
    assert_eq!(t.at_line(4), Line::Group(1));
    assert_eq!(t.at_line(5), Line::Row(3));
    assert_eq!(t.line_of(3), 5);
    assert_eq!(t.row_rect(0).y, 32.0 + 36.0);
    assert_eq!(t.row_at(10.0, 33.0, 400.0), None, "a group's heading is not a row");
    assert_eq!(t.row_at(10.0, 32.0 + 36.0 + 1.0, 400.0), Some(0));
    assert_eq!(t.visible(32.0 + 36.0 * 6.0), 0..4);
    assert_eq!(t.group_of(2), Some(0));
    assert_eq!(t.group_of(50), Some(1));
}

#[test]
fn revealing_a_group_s_first_row_brings_its_heading() {
    let mut t = table();
    t.set_groups(vec![(0, "A".into()), (50, "B".into())]);
    t.first = 80.0;
    t.reveal(50, 400.0);
    assert_eq!(t.first, 51.0, "the heading of B is on line 51");
    t.scroll(1000.0);
    assert_eq!(t.first, 101.0);
}

#[test]
fn a_reader_hears_the_group_of_each_row() {
    let mut t = table();
    t.set_groups(vec![(1, "Later".into())]);
    let said = t.said(32.0 + 36.0 * 3.0, |r, c| format!("{r}.{c}"));
    assert_eq!(said.rows.iter().map(|r| r.index).collect::<Vec<_>>(), [0, 1]);
    assert_eq!(said.rows[0].group, None);
    assert_eq!(said.rows[1].group.as_deref(), Some("Later"));
}

#[test]
fn a_heading_dragged_along_moves_its_column_and_keeps_the_sort() {
    let (mut t, mut g, now) = (table(), Gesture::new(), Instant::now());
    t.sort = Some((0, true));
    at(&mut g, &mut t, Pointer::Down(100.0, 10.0), 0, now);
    assert_eq!(at(&mut g, &mut t, Pointer::Move(103.0, 10.0), 5, now), None, "a small slip is still a press");
    assert_eq!(at(&mut g, &mut t, Pointer::Move(300.0, 10.0), 10, now), Some(Touch::Moving(0, 1)), "past the middle of Size");
    assert_eq!(at(&mut g, &mut t, Pointer::Move(350.0, 10.0), 15, now), Some(Touch::Moving(0, 2)), "past the middle of Kind");
    assert_eq!(g.moving(), Some((0, 2)));
    assert_eq!(at(&mut g, &mut t, Pointer::Up(350.0, 10.0), 20, now), Some(Touch::Moved(0, 2)));
    assert_eq!(t.columns.iter().map(|c| c.id).collect::<Vec<_>>(), [1, 2, 0]);
    assert_eq!(t.sort, Some((2, true)), "the sort stays on Name");
    assert_eq!(g.moving(), None);
}

#[test]
fn a_heading_let_go_where_it_was_neither_moves_nor_sorts() {
    let (mut t, mut g, now) = (table(), Gesture::new(), Instant::now());
    at(&mut g, &mut t, Pointer::Down(100.0, 10.0), 0, now);
    at(&mut g, &mut t, Pointer::Move(150.0, 10.0), 5, now);
    assert_eq!(at(&mut g, &mut t, Pointer::Up(150.0, 10.0), 10, now), None);
    assert_eq!(t.sort, None);
    assert_eq!(t.columns[0].id, 0);
}

fn editable() -> Table {
    let mut t = Table::new(vec![Column::new("Name", 200.0).editable(), Column::new("Size", 80.0)]);
    t.set_rows(10);
    t
}

fn key(t: &mut Table, sym: Keysym, text: Option<&str>) -> Option<Edited> {
    t.edit_key(sym, text, Mods::default())
}

#[test]
fn a_double_click_on_an_editable_cell_asks_to_type_in_it() {
    let (mut t, mut g, now) = (editable(), Gesture::new(), Instant::now());
    assert_eq!(at(&mut g, &mut t, Pointer::Down(10.0, 70.0), 0, now), Some(Touch::Chose(1)));
    at(&mut g, &mut t, Pointer::Up(10.0, 70.0), 5, now);
    assert_eq!(at(&mut g, &mut t, Pointer::Down(10.0, 70.0), 100, now), Some(Touch::Edit(1, 0)));
    at(&mut g, &mut t, Pointer::Up(10.0, 70.0), 105, now);
    at(&mut g, &mut t, Pointer::Down(250.0, 70.0), 2000, now);
    at(&mut g, &mut t, Pointer::Up(250.0, 70.0), 2005, now);
    assert_eq!(at(&mut g, &mut t, Pointer::Down(250.0, 70.0), 2100, now), Some(Touch::Opened(1)), "Size cannot be typed in");
    assert!(!t.begin_edit(1, 1, "4 kB"));
    assert_eq!(t.first_editable(), Some(0));
}

#[test]
fn typing_replaces_the_words_and_enter_keeps_them() {
    let mut t = editable();
    assert!(t.begin_edit(2, 0, "notes.txt"));
    assert_eq!(key(&mut t, Keysym::a, Some("a")), Some(Edited::Typing), "all was selected, so typing replaces it");
    key(&mut t, Keysym::b, Some("b"));
    key(&mut t, Keysym::Left, None);
    key(&mut t, Keysym::BackSpace, None);
    key(&mut t, Keysym::End, None);
    key(&mut t, Keysym::c, Some("c"));
    assert_eq!(t.editing.as_ref().map(|e| (e.text(), e.caret())), Some(("bc", 2)));
    assert_eq!(key(&mut t, Keysym::Return, None), Some(Edited::Kept { row: 2, column: 0, text: "bc".into() }));
    assert_eq!(t.editing, None);
    assert_eq!(key(&mut t, Keysym::Down, None), None, "with no cell typed in the key is the program's");
}

#[test]
fn a_cell_typed_in_undoes_and_redoes() {
    let mut t = editable();
    t.begin_edit(0, 0, "x");
    key(&mut t, Keysym::a, Some("a"));
    let ctrl = |t: &mut Table, sym, shift| t.edit_key(sym, None, Mods { ctrl: true, shift, ..Mods::default() });
    ctrl(&mut t, Keysym::z, false);
    assert_eq!(t.editing.as_ref().map(|e| e.text()), Some("x"));
    ctrl(&mut t, Keysym::y, false);
    assert_eq!(t.editing.as_ref().map(|e| e.text()), Some("a"));
    ctrl(&mut t, Keysym::Z, false);
    assert_eq!(t.editing.as_ref().map(|e| e.text()), Some("x"), "Ctrl+Z with Caps Lock still undoes");
    ctrl(&mut t, Keysym::Z, true);
    assert_eq!(t.editing.as_ref().map(|e| e.text()), Some("a"), "Ctrl+Shift+Z redoes");
}

#[test]
fn a_cell_copies_cuts_and_pastes_through_the_clipboard() {
    let mut t = editable();
    t.begin_edit(0, 0, "one two");
    let with = |t: &mut Table, sym, ctrl, shift| t.edit_key(sym, None, Mods { ctrl, shift, ..Mods::default() });
    assert_eq!(with(&mut t, Keysym::c, true, false), Some(Edited::Copy("one two".into())), "all was selected on starting");
    key(&mut t, Keysym::End, None);
    assert_eq!(with(&mut t, Keysym::c, true, false), Some(Edited::Typing), "nothing selected, nothing copied");
    assert_eq!(with(&mut t, Keysym::x, true, false), Some(Edited::Typing));
    key(&mut t, Keysym::Left, None);
    with(&mut t, Keysym::Left, true, true);
    assert_eq!(with(&mut t, Keysym::X, true, false), Some(Edited::Copy("tw".into())));
    assert_eq!(t.editing.as_ref().map(|e| e.text()), Some("one o"));
    assert_eq!(with(&mut t, Keysym::v, true, false), Some(Edited::Paste));
    assert_eq!(with(&mut t, Keysym::Insert, false, true), Some(Edited::Paste));
    assert!(t.paste("tw\no"));
    assert_eq!(t.editing.as_ref().map(|e| (e.text(), e.caret())), Some(("one tw oo", 8)));
    with(&mut t, Keysym::z, true, false);
    assert_eq!(t.editing.as_ref().map(|e| e.text()), Some("one o"), "a paste is one step of undo");
    with(&mut t, Keysym::z, true, false);
    assert_eq!(t.editing.as_ref().map(|e| e.text()), Some("one two"), "so is a cut");
    t.end_edit();
    assert!(!t.paste("x"), "no cell, nothing pasted");
}

#[test]
fn escape_drops_what_was_typed_and_a_press_away_says_so() {
    let (mut t, mut g, now) = (editable(), Gesture::new(), Instant::now());
    t.begin_edit(0, 0, "x");
    assert_eq!(at(&mut g, &mut t, Pointer::Down(10.0, 40.0), 0, now), None, "a press in the cell stays in it");
    assert_eq!(at(&mut g, &mut t, Pointer::Down(10.0, 200.0), 10, now), Some(Touch::LeftEdit));
    assert_eq!(g.cursor(&t, 10.0, 40.0), Cursor::Text);
    assert_eq!(key(&mut t, Keysym::Escape, None), Some(Edited::Dropped));
    assert_eq!(t.end_edit(), None);
}

#[test]
fn a_moved_column_keeps_the_cell_being_typed_in() {
    let mut t = editable();
    t.begin_edit(0, 0, "x");
    t.move_column(0, 1);
    assert_eq!(t.editing.as_ref().map(|e| e.column), Some(1));
    t.set_rows(0);
    assert_eq!(t.editing, None);
}

#[test]
fn groups_a_drop_line_and_a_typing_box_are_painted() {
    let theme = super::super::theme::Theme::new(true, "#fbc711");
    let (fonts, icons) = (Fonts::new(), super::super::icon::Icons::new());
    let pen = Pen { theme: &theme, fonts: &fonts, icons: &icons, scale: 1.0 };
    let mut t = editable();
    t.set_groups(vec![(0, "Group".into())]);
    t.begin_edit(0, 0, "a long name that is wider than the cell can show at once");
    let mut data = vec![0u8; 400 * 200 * 4];
    let mut canvas = PixmapMut::from_bytes(&mut data, 400, 200).unwrap();
    paint_table(&mut canvas, &pen, &t, None, |_, _| "x".into());
    paint_column_drop(&mut canvas, &pen, &t, Some((1, 0)));
    let is = |x: u32, y: u32, c: Colour| {
        let p = canvas.as_ref().pixel(x, y).unwrap().demultiply();
        [(p.red(), c.r), (p.green(), c.g), (p.blue(), c.b)].iter().all(|&(v, w)| (v as f32 - w * 255.0).abs() <= 2.0)
    };
    assert!(is(395, 32 + 18, theme.band), "the group's heading is on the band");
    assert!(is(196, 32 + 36 + 18, theme.field), "the cell is a typing box");
    assert!(is(0, 10, theme.accent), "the drop line");
}

#[test]
fn the_keys_choose_a_column_move_it_and_type_in_it() {
    let mut t = editable();
    assert_eq!(t.shift_column(1), None, "no column is on the keys yet");
    assert_eq!(t.edit_column(), Some(0));
    assert_eq!(t.step_column(-1), Some(0));
    assert_eq!(t.step_column(5), Some(1));
    t.sort = Some((0, true));
    assert_eq!(t.shift_column(-1), Some((1, 0)));
    assert_eq!((t.column, t.columns[0].id, t.sort), (Some(0), 1, Some((1, true))), "the keys and the sort stay on their columns");
    assert_eq!(t.shift_column(-1), None);
    assert_eq!(t.edit_column(), Some(1), "the keys' column cannot be typed in, so the first that can");
    t.move_column(1, 0);
    assert_eq!(t.column, Some(1), "a drag keeps the keys on their column");
    assert_eq!(Table::new(Vec::new()).step_column(1), None);
}

#[test]
fn a_press_moves_the_keys_once_they_are_in_use() {
    let (mut t, mut g, now) = (table(), Gesture::new(), Instant::now());
    at(&mut g, &mut t, Pointer::Down(250.0, 40.0), 0, now);
    assert_eq!(t.column, None);
    t.step_column(1);
    at(&mut g, &mut t, Pointer::Down(300.0, 40.0), 1000, now);
    assert_eq!(t.column, Some(2));
}

#[test]
fn the_keys_column_has_the_keyboard_ring() {
    let theme = super::super::theme::Theme::new(true, "#fbc711");
    let (fonts, icons) = (Fonts::new(), super::super::icon::Icons::new());
    let pen = Pen { theme: &theme, fonts: &fonts, icons: &icons, scale: 1.0 };
    let mut t = table();
    let mut data = vec![0u8; 400 * 200 * 4];
    let mut ring = |t: &Table, x: u32, y: u32| {
        data.fill(0);
        let mut canvas = PixmapMut::from_bytes(&mut data, 400, 200).unwrap();
        paint_table(&mut canvas, &pen, t, None, |_, _| String::new());
        let p = canvas.as_ref().pixel(x, y).unwrap().demultiply();
        let c = theme.accent;
        [(p.red(), c.r), (p.green(), c.g), (p.blue(), c.b)].iter().all(|&(v, w)| (v as f32 - w * 255.0).abs() <= 2.0)
    };
    assert!(!ring(&t, 200, 33));
    t.step_column(1);
    t.step_column(1);
    assert!(ring(&t, 240, 0), "on the heading while no row is chosen");
    t.chosen = Some(0);
    assert!(ring(&t, 240, 32) && ring(&t, 200, 50) && !ring(&t, 240, 0), "on the chosen row's cell");
}

#[test]
fn a_million_rows_in_groups_cost_no_more_than_the_rows_in_sight() {
    let theme = super::super::theme::Theme::new(true, "#fbc711");
    let (fonts, icons) = (Fonts::new(), super::super::icon::Icons::new());
    let pen = Pen { theme: &theme, fonts: &fonts, icons: &icons, scale: 1.0 };
    let mut t = table();
    t.set_rows(1_000_000);
    t.set_groups((0..1000).map(|g| (g * 1000, format!("Group {g}"))).collect());
    let asked = std::cell::Cell::new(0);
    let cell = |r: usize, c: usize| {
        asked.set(asked.get() + 1);
        format!("{r}.{c}")
    };
    let mut data = vec![0u8; 400 * 400 * 4];
    let began = Instant::now();
    t.end(400.0);
    t.first = 500_499.5;
    paint_table(&mut PixmapMut::from_bytes(&mut data, 400, 400).unwrap(), &pen, &t, None, cell);
    let said = t.said(400.0, cell);
    let took = began.elapsed();
    assert_eq!(t.lines(), 1_001_000);
    assert!(said.rows.len() <= 11);
    assert!(asked.get() <= 2 * 11 * 3, "only cells in sight are asked for: {}", asked.get());
    assert!(took < Duration::from_millis(500), "{took:?}");
}

#[test]
fn a_column_fits_its_words_and_the_table_is_painted() {
    let theme = super::super::theme::Theme::new(true, "#fbc711");
    let (fonts, icons) = (Fonts::new(), super::super::icon::Icons::new());
    let pen = Pen { theme: &theme, fonts: &fonts, icons: &icons, scale: 1.0 };
    let mut t = table();
    t.set_rows(3);
    let words = |r: usize, _| if r == 1 { "a much longer cell than the heading".to_string() } else { "x".to_string() };
    let before = t.columns[2].width;
    t.fit_to(2, &fonts, 1.0, words);
    assert!(t.columns[2].width > before, "the long cell widens the column");
    let mut data = vec![0u8; 400 * 200 * 4];
    let mut canvas = PixmapMut::from_bytes(&mut data, 400, 200).unwrap();
    paint_table(&mut canvas, &pen, &t, None, words);
    assert!(canvas.pixels_mut().iter().any(|p| p.alpha() > 0), "something is painted");
}
