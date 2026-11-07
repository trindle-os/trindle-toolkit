//! Tests of rows that open under a parent and of several rows chosen at once, with a band.

use std::time::{Duration, Instant};

use tiny_skia::PixmapMut;

use super::*;
use crate::input::Keysym;
use crate::theme::Theme;

const NONE: Mods = Mods { shift: false, ctrl: false, alt: false, logo: false };
const CTRL: Mods = Mods { ctrl: true, ..NONE };
const SHIFT: Mods = Mods { shift: true, ..NONE };
const SIZE: (f32, f32) = (400.0, 400.0);

fn columns() -> Vec<Column> {
    vec![Column::new("Name", 200.0), Column::new("Size", 80.0).end()]
}

fn tree() -> Table {
    let mut t = Table::new(columns());
    t.set_tree(vec![Branch::closed(0), Branch::leaf(1), Branch::closed(2)]);
    t
}

fn many(rows: usize) -> Table {
    let mut t = Table::new(columns());
    t.set_rows(rows);
    t.multiple = true;
    t
}

fn y_of(r: usize) -> f32 {
    32.0 + 36.0 * r as f32 + 18.0
}

fn press(g: &mut Gesture, t: &mut Table, e: Pointer, ms: u64, mods: Mods, base: Instant) -> Option<Touch> {
    g.pointer_mods(t, e, base + Duration::from_millis(ms), mods, SIZE)
}

fn click(g: &mut Gesture, t: &mut Table, (x, y): (f32, f32), ms: u64, mods: Mods, base: Instant) -> (Option<Touch>, Option<Touch>) {
    let down = press(g, t, Pointer::Down(x, y), ms, mods, base);
    (down, press(g, t, Pointer::Up(x, y), ms + 5, mods, base))
}

fn paint_all(t: &Table, band: Option<Rect>, cell: &impl Fn(usize, usize) -> String) -> Vec<u8> {
    let theme = Theme::new(true, "#fbc711");
    let (fonts, icons) = (Fonts::new(), super::super::icon::Icons::new());
    let pen = Pen { theme: &theme, fonts: &fonts, icons: &icons, scale: 1.0 };
    let mut data = vec![0u8; 400 * 400 * 4];
    let mut canvas = PixmapMut::from_bytes(&mut data, 400, 400).unwrap();
    paint_table(&mut canvas, &pen, t, None, cell);
    paint_band(&mut canvas, &pen, band);
    data
}

fn painted(t: &Table, band: Option<Rect>) -> Vec<u8> {
    paint_all(t, band, &|_, _| String::new())
}

#[test]
fn a_branch_opens_with_its_children_under_it_and_the_rows_below_move_down() {
    let mut t = tree();
    t.chosen = Some(2);
    t.multiple = true;
    t.set_picked([1, 2]);
    assert!(t.open_row(0, vec![Branch::leaf(10), Branch::closed(11)]));
    assert_eq!((t.rows, t.chosen, t.is_open(0)), (5, Some(4), Some(true)));
    assert_eq!(t.picked(), &[3, 4]);
    assert_eq!(t.branch(1).map(|b| (b.id, b.depth)), Some((10, 1)), "children sit one level in");
    assert!(!t.open_row(0, vec![Branch::leaf(12)]), "an open branch is not opened again");
    assert!(!t.open_row(1, vec![Branch::leaf(12)]), "a leaf has nothing to open");
    assert!(t.open_row(2, vec![Branch::leaf(20)]));
    assert_eq!(t.branch(3).map(|b| b.depth), Some(2));
    assert_eq!(t.parent_of(3), Some(2));
    assert_eq!(t.parent_of(2), Some(0));
    assert_eq!(t.parent_of(0), None);
}

#[test]
fn closing_a_branch_forgets_everything_under_it() {
    let mut t = tree();
    t.multiple = true;
    t.open_row(0, vec![Branch::leaf(10), Branch::closed(11)]);
    t.open_row(2, vec![Branch::leaf(20)]);
    t.chosen = Some(3);
    t.set_picked([3, 5]);
    assert!(t.close_row(0));
    assert_eq!((t.rows, t.is_open(0), t.chosen), (3, Some(false), Some(0)), "the chosen row that is gone falls to the branch");
    assert_eq!(t.picked(), &[2], "the picked row that was below moves up");
    assert!(!t.close_row(0));
    t.open_row(0, vec![Branch::leaf(10)]);
    assert_eq!(t.rows, 4, "reopened it holds only what the program gave again");
}

#[test]
fn a_branch_with_no_children_becomes_a_plain_row() {
    let mut t = tree();
    assert!(t.open_row(0, Vec::new()));
    assert_eq!((t.rows, t.is_open(0)), (3, None));
    assert_eq!(t.control_rect(0), None);
}

#[test]
fn groups_move_with_the_rows_opened_above_them() {
    let mut t = tree();
    t.set_groups(vec![(1, "Files".into())]);
    t.open_row(0, vec![Branch::leaf(10), Branch::leaf(11)]);
    assert_eq!(t.groups, vec![(3, "Files".to_string())]);
    t.close_row(0);
    assert_eq!(t.groups, vec![(1, "Files".to_string())]);
}

#[test]
fn set_rows_ends_a_tree() {
    let mut t = tree();
    t.set_rows(5);
    assert!(!t.is_tree() && t.control_rect(0).is_none());
}

#[test]
fn rows_are_set_in_by_depth_and_the_control_is_a_target_of_24_points() {
    let mut t = tree();
    t.row_height = 10.0;
    t.set_tree(vec![Branch::opened(0), Branch::closed(1).depth(1), Branch::leaf(2).depth(2)]);
    assert!(t.row_height >= 24.0);
    assert_eq!(t.lead(0), 24.0);
    let step = crate::theme::sheet().tree_indent;
    assert_eq!(t.lead(2) - t.lead(1), step);
    let c = t.control_rect(1).unwrap();
    assert!(c.w >= 24.0 && c.h >= 24.0, "{c:?}");
    assert!(t.control_rect(2).is_none());
    let mut flat = t.clone();
    flat.set_tree(vec![Branch::leaf(0), Branch::leaf(1).depth(1), Branch::leaf(2).depth(2)]);
    assert_ne!(painted(&t, None), painted(&flat, None), "the arrows are painted");
}

#[test]
fn the_keys_open_close_and_walk_a_tree() {
    let mut t = tree();
    assert_eq!(t.key(Keysym::Down, NONE, 400.0), Some(Touch::Chose(0)));
    assert_eq!(t.key(Keysym::Right, NONE, 400.0), Some(Touch::Open(0)), "a closed branch asks for its children");
    t.open_row(0, vec![Branch::leaf(10)]);
    assert_eq!(t.key(Keysym::Right, NONE, 400.0), Some(Touch::Chose(1)), "an open one goes to its first child");
    assert_eq!(t.key(Keysym::Right, NONE, 400.0), None, "a leaf has nowhere");
    assert_eq!(t.key(Keysym::Left, NONE, 400.0), Some(Touch::Chose(0)), "Left goes to the parent");
    assert_eq!(t.key(Keysym::Left, NONE, 400.0), Some(Touch::Closed(0)));
    assert_eq!(t.rows, 3);
    assert_eq!(t.key(Keysym::Left, NONE, 400.0), None, "the top has no parent");
    assert_eq!(t.key(Keysym::Return, NONE, 400.0), Some(Touch::Open(0)));
    t.open_row(0, vec![Branch::leaf(10)]);
    assert_eq!(t.key(Keysym::Return, NONE, 400.0), Some(Touch::Closed(0)));
    t.chosen = Some(1);
    assert_eq!(t.key(Keysym::Return, NONE, 400.0), Some(Touch::Opened(1)), "Enter on a leaf opens it");
    assert_eq!(t.key(Keysym::space, NONE, 400.0), Some(Touch::Opened(1)));
}

#[test]
fn enter_opens_a_row_of_a_plain_table_and_space_does_nothing() {
    let mut t = Table::new(columns());
    t.set_rows(3);
    t.chosen = Some(1);
    assert_eq!(t.key(Keysym::Return, NONE, 400.0), Some(Touch::Opened(1)));
    assert_eq!(t.key(Keysym::space, NONE, 400.0), None);
    assert_eq!(t.key(Keysym::Right, NONE, 400.0), None);
}

#[test]
fn the_control_the_row_and_a_double_click_each_do_their_part() {
    let (mut t, mut g, now) = (tree(), Gesture::new(), Instant::now());
    let c = t.control_rect(0).unwrap();
    for (n, x) in [c.x + 0.5, c.x + c.w - 0.5].into_iter().enumerate() {
        let (down, _) = click(&mut g, &mut t, (x, y_of(0)), 1000 * n as u64, NONE, now);
        assert_eq!(down, Some(Touch::Open(0)), "all of its 24 points are the control");
    }
    assert_eq!(t.chosen, None, "pressing the control does not choose the row");
    let beside = (c.x + c.w + 20.0, y_of(0));
    let (down, _) = click(&mut g, &mut t, beside, 5000, NONE, now);
    assert_eq!(down, Some(Touch::Chose(0)));
    let (down, _) = click(&mut g, &mut t, beside, 5100, NONE, now);
    assert_eq!(down, Some(Touch::Open(0)), "a double click on a closed branch opens it");
    t.open_row(0, vec![Branch::leaf(10)]);
    click(&mut g, &mut t, beside, 9000, NONE, now);
    let (down, _) = click(&mut g, &mut t, beside, 9100, NONE, now);
    assert_eq!((down, t.rows), (Some(Touch::Closed(0)), 3), "and closes an open one");
    click(&mut g, &mut t, (60.0, y_of(1)), 12000, NONE, now);
    let (down, _) = click(&mut g, &mut t, (60.0, y_of(1)), 12100, NONE, now);
    assert_eq!(down, Some(Touch::Opened(1)), "a leaf is opened");
}

#[test]
fn a_reader_is_told_each_row_s_level_openness_and_place_among_its_siblings() {
    let mut t = tree();
    t.open_row(0, vec![Branch::leaf(10), Branch::closed(11)]);
    let said = t.said(400.0, |_, _| String::new());
    assert!(said.tree && !said.many);
    let at = |i: usize| said.rows[i].branch.map(|b| (b.level, b.open, b.place, b.of));
    assert_eq!(at(0), Some((1, Some(true), 0, 3)));
    assert_eq!(at(1), Some((2, None, 0, 2)));
    assert_eq!(at(2), Some((2, Some(false), 1, 2)));
    assert_eq!(at(3), Some((1, None, 1, 3)));
    assert_eq!(at(4), Some((1, Some(false), 2, 3)));
}

#[test]
fn a_million_rows_with_branches_open_cost_no_more_than_the_rows_in_sight() {
    let mut t = Table::new(columns());
    t.set_tree((0..1_000_000).map(|i| if i % 1000 == 0 { Branch::closed(i) } else { Branch::leaf(i) }).collect());
    let began = Instant::now();
    assert!(t.open_row(500_000, (0..200).map(|i| if i % 50 == 0 { Branch::closed(i) } else { Branch::leaf(i) }).collect()));
    assert!(t.open_row(500_001 + 50, vec![Branch::leaf(1)]));
    let opening = began.elapsed();
    t.first = 500_040.0;
    t.chosen = Some(500_060);
    let asked = std::cell::Cell::new(0);
    let cell = |r: usize, c: usize| {
        asked.set(asked.get() + 1);
        format!("{r}.{c}")
    };
    let began = Instant::now();
    paint_all(&t, None, &cell);
    let said = t.said(400.0, cell);
    let took = began.elapsed();
    assert_eq!(t.rows, 1_000_201);
    assert!(said.rows.len() <= 11 && said.rows.iter().all(|r| r.branch.is_some()));
    assert!(asked.get() <= 2 * 11 * 2, "only cells in sight are asked for: {}", asked.get());
    assert!(took < Duration::from_millis(500), "painting {took:?}");
    assert!(opening < Duration::from_millis(500), "opening {opening:?}");
}

#[test]
fn several_rows_are_chosen_by_ctrl_and_shift_clicks_only_when_asked() {
    let (mut t, mut g, now) = (many(20), Gesture::new(), Instant::now());
    let (down, _) = click(&mut g, &mut t, (60.0, y_of(2)), 0, NONE, now);
    assert_eq!((down, t.picked()), (Some(Touch::Chose(2)), &[2][..]));
    let (down, up) = click(&mut g, &mut t, (60.0, y_of(5)), 1000, CTRL, now);
    assert_eq!((down, up), (None, Some(Touch::Toggled(5))), "Ctrl may begin a band, so the row toggles on letting go");
    assert_eq!(t.picked(), &[2, 5]);
    let (down, _) = click(&mut g, &mut t, (60.0, y_of(2)), 2000, CTRL, now);
    assert_eq!(down, None);
    assert_eq!(t.picked(), &[5]);
    let (down, _) = click(&mut g, &mut t, (60.0, y_of(8)), 3000, SHIFT, now);
    assert_eq!(down, Some(Touch::Extended(8)));
    assert_eq!(t.picked(), &[2, 3, 4, 5, 6, 7, 8], "a run from the last row clicked");
    click(&mut g, &mut t, (60.0, y_of(4)), 4000, NONE, now);
    assert_eq!(t.picked(), &[4], "a plain click leaves one");
    assert!(t.is_chosen(4) && !t.is_chosen(5));

    let (mut one, mut g) = (many(20), Gesture::new());
    one.multiple = false;
    let (down, _) = click(&mut g, &mut one, (60.0, y_of(5)), 0, CTRL, now);
    assert_eq!((down, one.picked()), (Some(Touch::Toggled(5)), &[][..]), "off, the program keeps the rows as before");
    one.pick_all();
    assert!(!one.is_chosen(3), "off, only the chosen row counts");
}

#[test]
fn shift_and_the_arrows_extend_ctrl_a_chooses_all_and_space_toggles() {
    let mut t = many(20);
    assert_eq!(t.key(Keysym::Down, NONE, 400.0), Some(Touch::Chose(0)));
    t.key(Keysym::Down, NONE, 400.0);
    t.key(Keysym::Down, NONE, 400.0);
    assert_eq!(t.picked(), &[2]);
    assert_eq!(t.key(Keysym::Down, SHIFT, 400.0), Some(Touch::Extended(3)));
    t.key(Keysym::Down, SHIFT, 400.0);
    assert_eq!(t.picked(), &[2, 3, 4]);
    t.key(Keysym::Up, SHIFT, 400.0);
    assert_eq!(t.picked(), &[2, 3], "Shift and Up takes the last back");
    t.key(Keysym::Home, SHIFT, 400.0);
    assert_eq!(t.picked(), &[0, 1, 2]);
    assert_eq!(t.key(Keysym::Down, NONE, 400.0), Some(Touch::Chose(1)));
    assert_eq!(t.picked(), &[1], "an arrow alone leaves one");
    assert_eq!(t.key(Keysym::a, CTRL, 400.0), Some(Touch::Picked));
    assert_eq!(t.picked().len(), 20);
    assert_eq!(t.key(Keysym::space, NONE, 400.0), Some(Touch::Toggled(1)));
    assert!(!t.is_chosen(1) && t.is_chosen(0));
    let mut single = Table::new(columns());
    single.set_rows(5);
    assert_eq!(single.key(Keysym::a, CTRL, 400.0), None, "off, Ctrl+A does nothing");
    assert_eq!(single.key(Keysym::Down, SHIFT, 400.0), Some(Touch::Chose(0)), "off, Shift does not extend");
    assert!(single.picked().is_empty());
}

#[test]
fn a_band_from_the_room_below_the_rows_picks_what_it_crosses() {
    let (mut t, mut g, now) = (many(5), Gesture::new(), Instant::now());
    t.set_picked([0]);
    assert_eq!(press(&mut g, &mut t, Pointer::Down(50.0, 300.0), 0, NONE, now), None);
    assert_eq!(g.band(&t, SIZE), None, "not a band until it moves");
    assert_eq!(press(&mut g, &mut t, Pointer::Move(52.0, 298.0), 1, NONE, now), None);
    assert_eq!(press(&mut g, &mut t, Pointer::Move(50.0, y_of(2)), 2, NONE, now), Some(Touch::Banded));
    assert_eq!(t.picked(), &[2, 3, 4], "what was picked goes: a band without Ctrl picks afresh");
    assert_eq!(press(&mut g, &mut t, Pointer::Move(90.0, y_of(2) + 1.0), 3, NONE, now), None, "nothing new, nothing said");
    let band = g.band(&t, SIZE).expect("a band");
    assert!(band.y >= 32.0 && band.y + band.h <= 400.0 && band.w > 0.0);
    assert_ne!(painted(&t, Some(band)), painted(&t, None), "the band is painted");
    assert_eq!(press(&mut g, &mut t, Pointer::Up(50.0, y_of(2)), 4, NONE, now), None);
    assert_eq!(g.band(&t, SIZE), None);
    assert_eq!(t.picked(), &[2, 3, 4]);
}

#[test]
fn a_band_begun_with_ctrl_adds_to_the_rows_picked_and_may_begin_on_a_row() {
    let (mut t, mut g, now) = (many(10), Gesture::new(), Instant::now());
    t.set_picked([8]);
    press(&mut g, &mut t, Pointer::Down(50.0, y_of(0)), 0, CTRL, now);
    assert_eq!(press(&mut g, &mut t, Pointer::Move(50.0, y_of(2)), 1, CTRL, now), Some(Touch::Banded));
    assert_eq!(t.picked(), &[0, 1, 2, 8]);
    assert_eq!(press(&mut g, &mut t, Pointer::Up(50.0, y_of(2)), 2, CTRL, now), None, "a band let go is not also a click");
    let (mut t, mut g) = (many(10), Gesture::new());
    press(&mut g, &mut t, Pointer::Down(50.0, y_of(0)), 0, NONE, now);
    assert_eq!(press(&mut g, &mut t, Pointer::Move(50.0, y_of(4)), 1, NONE, now), None, "a plain press on a row begins no band");
    assert!(t.picked().len() == 1 && g.band(&t, SIZE).is_none());
}

#[test]
fn no_band_without_several_row_choice_and_none_from_the_heading() {
    let (mut t, mut g, now) = (many(5), Gesture::new(), Instant::now());
    t.multiple = false;
    press(&mut g, &mut t, Pointer::Down(50.0, 300.0), 0, NONE, now);
    assert_eq!(press(&mut g, &mut t, Pointer::Move(50.0, y_of(2)), 1, NONE, now), None);
    assert!(t.picked().is_empty());
    t.multiple = true;
    let mut g = Gesture::new();
    press(&mut g, &mut t, Pointer::Down(50.0, 10.0), 0, NONE, now);
    press(&mut g, &mut t, Pointer::Move(50.0, y_of(2)), 1, NONE, now);
    assert!(t.picked().is_empty(), "the heading is not room for a band");
}

#[test]
fn a_band_counts_rows_not_group_headings_and_follows_a_scroll() {
    let (mut t, mut g, now) = (many(12), Gesture::new(), Instant::now());
    t.set_groups(vec![(2, "Later".into())]);
    press(&mut g, &mut t, Pointer::Down(50.0, y_of(0)), 0, CTRL, now);
    press(&mut g, &mut t, Pointer::Move(50.0, y_of(3)), 1, CTRL, now);
    assert_eq!(t.picked(), &[0, 1, 2]);
    let (mut t, mut g) = (many(30), Gesture::new());
    press(&mut g, &mut t, Pointer::Down(50.0, y_of(4)), 0, CTRL, now);
    t.scroll(10.0);
    press(&mut g, &mut t, Pointer::Move(50.0, y_of(4) + 80.0), 1, CTRL, now);
    assert_eq!(t.picked(), &(4..=16).collect::<Vec<_>>()[..], "the band stays where it began in the rows");
}

#[test]
fn a_reader_is_told_which_rows_are_chosen_and_that_there_can_be_several() {
    let mut t = many(10);
    t.set_picked([1, 3]);
    t.chosen = Some(3);
    let said = t.said(400.0, |_, _| String::new());
    assert!(said.many && !said.tree);
    assert_eq!(said.cursor, Some(3));
    let chosen: Vec<usize> = said.rows.iter().filter(|r| r.chosen).map(|r| r.index).collect();
    assert_eq!(chosen, vec![1, 3]);
}

#[test]
fn a_million_rows_all_chosen_cost_no_more_than_the_rows_in_sight() {
    let mut t = many(1_000_000);
    t.pick_all();
    t.chosen = Some(500_000);
    t.first = 499_990.0;
    let asked = std::cell::Cell::new(0);
    let cell = |_: usize, _: usize| {
        asked.set(asked.get() + 1);
        String::new()
    };
    let began = Instant::now();
    paint_all(&t, None, &cell);
    let said = t.said(400.0, cell);
    let took = began.elapsed();
    assert!(said.rows.len() <= 11 && said.rows.iter().all(|r| r.chosen));
    assert!(asked.get() <= 2 * 11 * 2);
    assert!(took < Duration::from_millis(500), "{took:?}");
}
